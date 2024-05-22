use stm32f3xx_hal::pac::{TIM1, TIM3};
use paste::paste;
macro_rules! encoder_reader {
    ($timer:ident) => {
        paste! {
            pub struct [<EncoderReader $timer>] {
                enc: $timer,
                prev_value: u16,
                low_margin: u16,
                high_margin: u16
            }

            impl [<EncoderReader $timer>] {
                pub fn new(enc: $timer, low_margin: u16, high_margin: u16) -> Self {
                    //Configure timer to act as encoder interface
                    enc.smcr.modify(|_,w| w.sms().encoder_mode_1()); // Act on both channels
                    enc.ccer.modify(|_,w| w.cc1p().set_bit().cc2p().set_bit()); //On falling edge
                    enc.ccmr1_input().modify(|_, w| w.ic1f().fdts_div8_n6()); // Filter 8
                    let filter_value = enc.ccmr1_input().read().ic1f().bits();
                    enc.ccmr1_input().modify(|_, w| w.ic2f().bits(filter_value)); //Copy of filter 8
                    enc.ccmr1_input().modify(|_,w| w.cc1s().ti1().cc2s().ti2());
                    enc.psc.write(|w| w.psc().bits(0)); //No prescaler
                    enc.arr.write(|w| w.arr().bits(1024)); //1024 values
                    enc.cnt.write(|w| w.cnt().bits(512)); //Start in the middle
                    enc.cr1.modify(|_,w| w.cen().enabled()); //Start encoder interface

                    //Read the current value
                    let prev_value = enc.cnt.read().cnt().bits()>>1;
                    Self{enc, prev_value, low_margin, high_margin}
                }

                pub fn read(&mut self, current: u16) -> Option<u16> {
                    let dir = self.enc.cr1.read().dir().is_down();
                    let value = self.enc.cnt.read().cnt().bits() >> 1;
                    let steps = get_steps(dir, self.prev_value, value);
                    if steps != 0 {
                        self.prev_value = value;
                        let mut next_value = if dir {
                            current.saturating_sub(steps)
                        } else {
                            current + steps
                        };
                        // Clamping the range
                        if next_value < self.low_margin {
                            next_value = self.low_margin;
                        }
                        if next_value > self.high_margin {
                            next_value = self.high_margin;
                        }
                        Some(next_value)
                    } else {
                        None
                    }
                }
            }
        }
    }
}

encoder_reader!(TIM1);
encoder_reader!(TIM3);

fn get_steps(direction: bool, prev: u16, current: u16) -> u16 {
    if prev == current {
        return 0; //No changes
    }
    let steps = if direction { //Counting backwards
        if current > prev { //On backward count this means wrap
            512 - current + prev
        } else {
            prev - current
        }
    } else {
        if current < prev { //On forward count this means wrap
            512 - prev + current
        } else {
            current - prev
        }
    };
    if steps > 50 { //Most probably overspeed error, ignore it
        0
    } else {
        steps
    }
}