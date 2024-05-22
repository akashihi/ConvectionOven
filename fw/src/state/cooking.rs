use pid::Pid;
use stm32f3xx_hal::prelude::_embedded_hal_digital_OutputPin;
use crate::state::{Oven, OvenControl, OvenControlHardware};
use crate::state::lid::LidOpen;
use crate::state::ready::OvenReady;
use libm::roundf;
use crate::current_sensor::CurrentSensor;
use crate::state::OvenHalt;
use crate::temp_sensor::TempSensor;

const MINUTE_IN_MS: u16 = 600; //State update timer runs in 100ms=0.1s ticks, thus minute is a 600 ticks
const K_P: f32 = 4.8; //K_u = 8, K_P = 0.6*8
const K_I: f32 = 0.06; //P_u = 145seconds = 0.006Hz, K_i = 1.2*K_u/P_u=
const K_D: f32 = 7.4; //K_d=0.075*K_u*P_u

pub struct Cooking {
    hw: OvenControlHardware,
    heater_percents: u8,
    heater_updates: u16,
    minute_delay: u16,
    temp_actual: u16,
    temp_intenal: u16,
    pid: Pid<f32>
}

impl Cooking {
    pub fn new(mut hw: OvenControlHardware) -> Self {
        let mut pid = Pid::new(50.0, 150.0);
        //defmt::println!("K_P: {}, K_I: {}, K_D: {}", K_P, K_I, K_D);
        pid.p(K_P, 150.0);
        pid.i(K_I, 150.0);
        pid.d(K_D, 150.0);

        hw.display.message("*****Cooking****");
        hw.motor.set_low().unwrap_or_default(); //Immediately start motor on cooking start
        hw.cook_ld.set_high().unwrap_or_default();
        hw.buzzer.run_beep();

        Cooking { hw, heater_percents: 0, heater_updates: 0, minute_delay: MINUTE_IN_MS, temp_actual: 0, temp_intenal: 0, pid}
    }

    fn shutdown(&mut self) {
        self.hw.cook_ld.set_low().unwrap_or_default();
        self.hw.heater.set_low().unwrap_or_default();
        self.hw.motor.set_high().unwrap_or_default();
    }
}

impl OvenControl for Cooking {
    fn on_cook_btn(mut self) -> Oven {
        self.shutdown();
        Oven::from(OvenReady::new(self.hw))
    }

    fn on_sensors(mut self, lid: bool, temp_sensor: &TempSensor, current_sensor: &CurrentSensor) -> Oven {
        self.temp_intenal = temp_sensor.get_internal_temperature().unwrap_or(0.0) as u16;
        if temp_sensor.is_error() {
            OvenHalt::temp_error(self.hw)
        } else if temp_sensor.is_overheating() {
            OvenHalt::overheating(self.hw)
        } else if current_sensor.is_error() {
            OvenHalt::current_error(self.hw)
        } else if !current_sensor.is_running() {
            OvenHalt::motor_failed(self.hw)
        } else if current_sensor.is_overloaded() {
            OvenHalt::motor_overload(self.hw)
        } else if !lid {
            self.shutdown();
            Oven::from(LidOpen::new(self.hw))
        } else {
            Oven::from(self)
        }
    }

    fn on_settings(mut self, temp_actual: u16, temp_requested: u16, time: u16) -> (Oven, u16) {
        self.temp_actual = temp_actual; //Saved for a PID call. It'll be outdated, but heating machines have huge inertia

        if self.pid.setpoint as u16 != temp_requested {
            self.pid.setpoint(temp_requested as f32);
        }
        self.heater_updates += 1;
        if self.heater_percents > 0 {
            self.hw.heater.set_high().unwrap_or_default();
            self.heater_percents -= 1;
        } else {
            self.hw.heater.set_low().unwrap_or_default();
        }
        self.minute_delay -= 1;
        let next_time = if self.minute_delay == 0 {
            self.minute_delay = MINUTE_IN_MS;
            let next = time - 1;
            if next == 1 { //We have to check that here, as we want .pre_beep() to be called exactly ones, when we hit the last minute
                self.hw.buzzer.pre_beep();
            }
            next
        } else {
            time
        };
        if next_time == 0 {
            self.hw.buzzer.done_beep();
            self.shutdown();
            (Oven::from(OvenReady::new(self.hw)), next_time)
        } else {
            (Oven::from(self), next_time)
        }
    }

    fn on_pid(&mut self) {
        let control =  self.pid.next_control_output(self.temp_actual as f32);
        if control.output <= 0.0 || (self.temp_actual as f32 - self.pid.setpoint) > 30.0 {
            self.heater_percents = 0;
        } else {
            self.heater_percents = roundf(control.output) as u8;
        }
        //defmt::println!("PID output: {}, actual_temp: {}, requested_temp: {}, intervals: {}, updates: {}, internal_temp: {}", control.output, self.temp_actual, self.pid.setpoint, self.heater_percents, self.heater_updates, self.temp_intenal);
        self.heater_updates = 0;
    }

    fn get_hw_ref(&mut self) -> &mut OvenControlHardware {
        &mut self.hw
    }
}