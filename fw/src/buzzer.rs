use stm32f3xx_hal::hal::digital::v2::OutputPin;

pub struct BuzzerManager<T: OutputPin> {
    buzzer: T,
    periods: u32
}

impl<T:OutputPin> BuzzerManager<T> {
    pub fn new(buzzer: T) -> Self {
        Self{buzzer, periods: 0}
    }

    pub fn on_timer(&mut self) {
        if self.periods > 0 {
            self.periods -= 1;
            if self.periods == 0 {
                self.buzzer.set_high().ok();
            }
        }
    }

    fn beep(&mut self, duration: u32) {
        if self.periods == 0 {
            self.periods = duration;
            self.buzzer.set_low().ok();
        }
    }

    /// Short beep on `run` button call
    pub fn run_beep(&mut self) {
        self.beep(3)
    }

    /// pre-cook beep, on minute to done
    pub fn pre_beep(&mut self) {
        self.beep(10)
    }

    /// Cooking is done
    pub fn done_beep(&mut self) {
        self.beep(25)
    }
}