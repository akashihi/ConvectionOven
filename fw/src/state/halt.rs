use crate::current_sensor::CurrentSensor;
use crate::state::{Oven, OvenControl, OvenControlHardware};
use crate::temp_sensor::TempSensor;

/**
Halt state. Triggered by any other state when error is detected.

Can't set temp/time.
Can't start cooking.
 */
pub struct OvenHalt {
    hw: OvenControlHardware
}

impl OvenHalt {
    pub fn overheating(mut hw: OvenControlHardware) -> ! {
        hw.display.error_message("DEVICE OVERHEAT!");
        loop {cortex_m::asm::nop()}
    }

    pub fn temp_error(mut hw: OvenControlHardware) -> ! {
        hw.display.error_message("T SENSOR FAILURE");
        loop {cortex_m::asm::nop()}
    }

    pub fn current_error(mut hw: OvenControlHardware) -> ! {
        hw.display.error_message("C SENSOR FAILURE");
        loop {cortex_m::asm::nop()}
    }

    pub fn motor_uncontrolled(mut hw: OvenControlHardware) -> ! {
        hw.display.error_message(" MOTOR CONTROL! ");
        loop {cortex_m::asm::nop()}
    }

    pub fn motor_failed(mut hw: OvenControlHardware) -> ! {
        hw.display.error_message(" MOTOR FAILURE! ");
        loop {cortex_m::asm::nop()}
    }

    pub fn motor_overload(mut hw: OvenControlHardware) -> ! {
        hw.display.error_message(" MOTOR OVERLOAD ");
        loop {cortex_m::asm::nop()}
    }
}

impl OvenControl for OvenHalt {
    fn on_cook_btn(self) -> Oven {
        Oven::from(self)
    }

    fn on_sensors(self, _: bool, _: &TempSensor, _: &CurrentSensor) -> Oven {
        Oven::from(self)
    }

    fn on_settings(self, _temp_actual: u16, _temp_requested: u16, _time: u16) -> (Oven, u16) {
        (Oven::from(self), 0)
    }

    fn on_pid(&mut self) {}

    fn get_hw_ref(&mut self) -> &mut OvenControlHardware {
        &mut self.hw
    }
}