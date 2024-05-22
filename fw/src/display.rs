use heapless::String;
use core::fmt::Write;
use stm32f3xx_hal::timer;
use crate::board;
use crate::delay::TimDelay;

pub struct LcdDisplay<TIM: timer::Instance> {
    lcd: board::LCD,
    delay: TimDelay<TIM>,
}

impl<TIM: timer::Instance> LcdDisplay<TIM> {
    pub fn new(mut lcd: board::LCD, mut delay: TimDelay<TIM>) -> Self {
        lcd.clear(&mut delay).unwrap_or_default();
        LcdDisplay { lcd, delay }
    }

    pub fn error_message(&mut self, msg: &str) {
        self.lcd.clear(&mut self.delay).unwrap_or_default();
        self.lcd.set_cursor_pos(0, &mut self.delay).unwrap_or_default();
        self.lcd.write_str(msg, &mut self.delay).unwrap_or_default();
    }

    pub fn message(&mut self, msg: &str) {
        self.lcd.set_cursor_pos(0, &mut self.delay).unwrap_or_default();
        self.lcd.write_str(msg, &mut self.delay).unwrap_or_default();
    }

    pub fn state(&mut self, time: u16, temp_actual: u16, temp_requested: u16) {
        let mut temp_string: String<3> = String::new();
        if temp_actual < 50 {
            write!(temp_string, "{}", "---").unwrap_or_default();
        } else {
            write!(temp_string, "{:03}", temp_actual).unwrap_or_default();
        }
        let mut output: String<16> = String::new();
        write!(output, "T{:02}:{:02} {}/{:03}", time / 60, time % 60, temp_string, temp_requested).unwrap_or_default();
        self.lcd.set_cursor_pos(40, &mut self.delay).unwrap_or_default();
        self.lcd.write_str(&output, &mut self.delay).unwrap_or_default();
        self.lcd.write_byte(0xDFu8, &mut self.delay).unwrap_or_default();
        self.lcd.write_str("t", &mut self.delay).unwrap_or_default();
    }
}