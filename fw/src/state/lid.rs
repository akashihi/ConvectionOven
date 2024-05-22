use crate::current_sensor::CurrentSensor;
use crate::state::{Oven, OvenControl, OvenControlHardware};
use crate::state::OvenHalt;
use crate::state::OvenReady;
use crate::temp_sensor::TempSensor;

/**
 LidOpen state. Triggered by any other state when lid is open (detected by lid sensor).

 Can set temp/time.
 Can't start cooking.
*/
pub struct LidOpen {
    hw: OvenControlHardware
}

impl LidOpen {
    pub fn new(mut hw: OvenControlHardware) -> Self {
        hw.display.message("Please close lid");
        LidOpen{hw}
    }
}

impl OvenControl for LidOpen{
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
        } else if lid {
            Oven::from(OvenReady::new(self.hw))
        } else {
            Oven::from(self)
        }
    }

    fn on_settings(self, _temp_actual: u16, _temp_requested: u16, time: u16) -> (Oven, u16) {
        (Oven::from(self), time)
    }

    fn on_pid(&mut self) {}

    fn get_hw_ref(&mut self) -> &mut OvenControlHardware {
        &mut self.hw
    }

}