use stm32f3xx_hal::hal::blocking::delay::{DelayMs, DelayUs};
use stm32f3xx_hal::prelude::_embedded_hal_timer_CountDown;
use stm32f3xx_hal::{block};
use stm32f3xx_hal::rcc::{Clocks, RccBus};
use stm32f3xx_hal::time::duration::Milliseconds;
use stm32f3xx_hal::timer::{Instance, Timer};

pub struct TimDelay<TIM: RccBus> {
    timer: Timer<TIM>
}

impl<TIM: RccBus + Instance> TimDelay<TIM> {
    pub fn new(tim: TIM, clocks: Clocks, apb: &mut <TIM as RccBus>::Bus) -> Self {
        let timer = Timer::new(tim, clocks, apb);
        TimDelay{timer}
    }
}

impl<TIM: RccBus + Instance> DelayMs<u8> for TimDelay<TIM> {
    fn delay_ms(&mut self, ms: u8) {
        self.timer.start(Milliseconds::new(ms as u32));
        block!(self.timer.wait()).unwrap();
        self.timer.stop()
    }
}

impl<TIM: RccBus + Instance> DelayUs<u16> for TimDelay<TIM> {
    fn delay_us(&mut self, _: u16) {
        self.timer.start(Milliseconds::new(1u32)); //HD44780 needs longer breaks
        block!(self.timer.wait()).unwrap();
        self.timer.stop()
    }
}