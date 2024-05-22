use crate::current_sensor::CurrentSensor;
use crate::state::{Oven, OvenControl, OvenControlHardware};
use crate::state::halt::OvenHalt;
use crate::state::lid::LidOpen;
use crate::state::pre_run::OvenPreRun;
use crate::temp_sensor::TempSensor;

pub struct OvenReady {
    hw: OvenControlHardware
}

impl OvenReady {
    pub fn new(mut hw: OvenControlHardware) -> Self {
        hw.display.message("     Ready      ");
        OvenReady{hw}
    }

}

impl OvenControl for OvenReady {
    fn on_cook_btn(self) -> Oven {
        Oven::from(self)
    }

    fn on_sensors(self, lid: bool, temp_sensor: &TempSensor, current_sensor: &CurrentSensor) -> Oven {
        if temp_sensor.is_error() {
            OvenHalt::temp_error(self.hw)
        } else if temp_sensor.is_overheating() {
            OvenHalt::overheating(self.hw)
        } else if current_sensor.is_error() {
            OvenHalt::current_error(self.hw)
        } else if !current_sensor.is_standby() {
            OvenHalt::motor_uncontrolled(self.hw)
        } else if !lid {
            Oven::from(LidOpen::new(self.hw))
        } else {
            Oven::from(self)
        }
    }

    fn on_settings(self, _temp_actual: u16, _temp_requested: u16, time: u16) -> (Oven, u16) {
        if time == 0 {
            (Oven::from(self), time)
        } else {
            (Oven::from(OvenPreRun::new(self.hw)), time)
        }
    }

    fn on_pid(&mut self) {}

    fn get_hw_ref(&mut self) -> &mut OvenControlHardware {
        &mut self.hw
    }

}