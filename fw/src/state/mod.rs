use enum_dispatch::enum_dispatch;
use stm32f3xx_hal::pac::TIM7;
use crate::board::{Buzzer, CookLd, HeaterEnable, MotorEnable};
use crate::buzzer::BuzzerManager;
use crate::current_sensor::CurrentSensor;
use crate::display::LcdDisplay;
use crate::state::cooking::Cooking;

pub mod halt;
pub mod lid;
pub mod ready;
pub mod cooking;
pub mod pre_run;
pub mod manager;

use crate::state::halt::OvenHalt;
use crate::state::lid::LidOpen;
use crate::state::pre_run::OvenPreRun;
use crate::state::ready::OvenReady;
use crate::temp_sensor::TempSensor;

pub struct OvenControlHardware {
    pub display: LcdDisplay<TIM7>,
    pub buzzer: BuzzerManager<Buzzer>,
    pub cook_ld: CookLd,
    pub heater: HeaterEnable,
    pub motor: MotorEnable
}

#[enum_dispatch]
trait OvenControl {
    fn on_cook_btn(self) -> Oven;
    fn on_sensors(self, lid: bool, temp_sensor: &TempSensor, current_sensor: &CurrentSensor) -> Oven;
    fn on_settings(self, temp_actual: u16, temp_requested: u16, time: u16) -> (Oven, u16);
    fn on_pid(&mut self);
    fn get_hw_ref(&mut self) -> &mut OvenControlHardware;
}

#[enum_dispatch(OvenControl)]
enum Oven {
    OvenHalt,
    LidOpen,
    OvenReady,
    OvenPreRun,
    Cooking
}