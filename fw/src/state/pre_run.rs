use crate::current_sensor::CurrentSensor;
use crate::state::{Oven, OvenControl, OvenControlHardware};
use crate::state::cooking::Cooking;
use crate::state::halt::OvenHalt;
use crate::state::lid::LidOpen;
use crate::state::ready::OvenReady;
use crate::temp_sensor::TempSensor;

pub struct OvenPreRun {
    hw: OvenControlHardware
}

impl OvenPreRun {
    pub fn new(mut hw: OvenControlHardware) -> Self {
        hw.display.message("    Press RUN   ");
        OvenPreRun{hw}
    }

}

impl OvenControl for OvenPreRun {
    fn on_cook_btn(self) -> Oven {
        Oven::from(Cooking::new(self.hw))
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
            (Oven::from(OvenReady::new(self.hw)), time)
        } else {
            (Oven::from(self), time)
        }
    }

    fn on_pid(&mut self) {}

    fn get_hw_ref(&mut self) -> &mut OvenControlHardware {
        &mut self.hw
    }

}