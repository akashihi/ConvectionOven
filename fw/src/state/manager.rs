use crate::current_sensor::CurrentSensor;
use crate::encoder::{EncoderReaderTIM1, EncoderReaderTIM3};
use crate::state::{Oven, OvenControlHardware, OvenControl};
use crate::state::ready::OvenReady;
use crate::temp_sensor::TempSensor;

pub struct StateManager {
    time: u16,
    temp_requested: u16,
    temp_actual: u16,
    temp_actual_raw: u16,
    temp_enc: EncoderReaderTIM1,
    time_enc: EncoderReaderTIM3,
    state: Option<Oven>,
    temp_sensor: TempSensor,
    current_sensor: CurrentSensor
}

impl StateManager {
    pub fn new(hw: OvenControlHardware, current_sensor: CurrentSensor, temp_enc: EncoderReaderTIM1, time_enc: EncoderReaderTIM3, temp_sensor: TempSensor) -> Self {
        let initial_state = Some(Oven::from(OvenReady::new(hw)));
        let mut manager = StateManager{time:0, temp_requested: 50, temp_actual: 0, temp_actual_raw: 0, temp_enc, time_enc, state: initial_state, temp_sensor, current_sensor};
        manager.state.as_mut().unwrap().get_hw_ref().display.state(manager.time, manager.temp_actual, manager.temp_requested);
        manager
    }

    pub fn adc_poll(&mut self, volts: f32) {
        self.current_sensor.add_value(volts);
    }

    pub fn enc_poll(&mut self, lid: bool) {
        //Poll sensors
        self.temp_sensor.poll_sensor();
        if let Some(o) = &mut self.state {
            o.get_hw_ref().buzzer.on_timer();
        }

        //Check lid state
        if self.state.is_some() { //Check the lid state
            let lid_value_state = self.state.take().map(|o| Oven::from(o.on_sensors(lid, &self.temp_sensor, &self.current_sensor)));
            self.state = lid_value_state;
        }

        let mut state_updated = self.temp_enc.read(self.temp_requested/5).map(|v| self.temp_requested = v * 5).is_some();
        state_updated = state_updated || self.time_enc.read(self.time).map(|v| self.time = v).is_some();

        if let Some(measured_temp) = self.temp_sensor.get_sensor() {
            self.temp_actual_raw = measured_temp as u16; //Lets feed PID with actualy temp values
            let q_value = if libm::fabsf(self.temp_requested as f32 - measured_temp) <= 5.0 {
                self.temp_requested
            } else {
                ((measured_temp/5.0) as u16) * 5 //Quantization by 5
            };
            if self.temp_actual != q_value {
                self.temp_actual = q_value;
                state_updated = true
            }
        }

        if self.state.is_some() {
            let (settings_state, time) = self.state.take().unwrap().on_settings(self.temp_actual_raw, self.temp_requested, self.time);
            if time != self.time {
                self.time = time;
                state_updated = true;
            }
            self.state = Some(settings_state);
        }

        if state_updated{
            if let Some(o) = &mut self.state {
                o.get_hw_ref().display.state(self.time, self.temp_actual, self.temp_requested);
            }
        }

    }

    pub fn pid_poll(&mut self) {
        self.state.as_mut().map(|o| o.on_pid());
    }

    pub fn on_cook_btn(&mut self) {
        let cook_value_state = self.state.take().map(|o| Oven::from(o.on_cook_btn()));
        self.state = cook_value_state;
    }
}