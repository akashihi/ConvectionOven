use hd44780_driver::bus::FourBitBus;
use hd44780_driver::{Cursor, HD44780};
use stm32f3xx_hal::gpio::{Alternate, Analog, GpioExt, Input, OpenDrain, Output, PA0, PA1, PA10, PA11, PA12, PA15, PA2, PA3, PA4, PA5, PA6, PA7, PA8, PA9, PB0, PB1, PB3, PB4, PB5, PB6, PB7, PushPull};
use stm32f3xx_hal::hal::blocking::delay::{DelayMs, DelayUs};
use stm32f3xx_hal::pac::{GPIOA, GPIOB, SPI1};
use stm32f3xx_hal::rcc::{AHB, APB2, Clocks};
use stm32f3xx_hal::spi::Spi;
use stm32f3xx_hal::prelude::*;

pub type CookBtn = PA0<Input>;
pub type Lid = PB6<Input>;
pub type CookLd = PA1<Output<PushPull>>;
pub type Buzzer = PA2<Output<OpenDrain>>;
pub type TempEncA = PA8<Alternate<OpenDrain, 6>>;
pub type TempEncB = PA9<Alternate<OpenDrain, 6>>;
pub type TimeEncA = PB4<Alternate<OpenDrain, 2>>;
pub type TimeEncB = PB5<Alternate<OpenDrain, 2>>;
pub type Current = PB1<Analog>;
pub type TcCs = PA4<Output<PushPull>>;
pub type TcSck = PA5<Alternate<PushPull, 5>>;
pub type TcSo = PA6<Alternate<PushPull, 5>>;
pub type TcSi = PA7<Alternate<PushPull, 5>>;
pub type MotorEnable = PA11<Output<OpenDrain>>;
pub type HeaterEnable = PA12<Output<PushPull>>;

pub type SpiBus = Spi<SPI1, (TcSck, TcSo, TcSi), u8>;

type Hd4BitBus = FourBitBus<
    PA15<Output<PushPull>>,
    PB7<Output<PushPull>>,
    PB3<Output<PushPull>>,
    PA10<Output<PushPull>>,
    PB0<Output<PushPull>>,
    PA3<Output<PushPull>>
>;

pub type LCD = HD44780<Hd4BitBus>;

pub struct Board {
    pub cook_btn: CookBtn,
    pub lid: Lid,
    pub cook_ld: CookLd,
    pub buzzer: Buzzer,
    pub temp_encoder: (TempEncA, TempEncB),
    pub time_encoder: (TimeEncA, TimeEncB),
    pub current: Current,
    pub lcd: LCD,
    pub tc_cs: TcCs,
    pub tc_spi: SpiBus,
    pub heater: HeaterEnable,
    pub motor: MotorEnable,
}

impl Board {
    pub fn new<D: DelayUs<u16> + DelayMs<u8>>(gpioa: GPIOA, gpiob: GPIOB, spi: SPI1, ahb: &mut AHB, apb2: &mut APB2, clocks: Clocks, delay: &mut D) -> Self {
        let mut port_a = gpioa.split(ahb);
        let mut port_b = gpiob.split(ahb);

        let cook_btn = port_a.pa0.into_floating_input(&mut port_a.moder, &mut port_a.pupdr);
        let lid = port_b.pb6.into_floating_input(&mut port_b.moder, &mut port_b.pupdr);
        let cook_ld = port_a.pa1.into_push_pull_output(&mut port_a.moder, &mut port_a.otyper);
        let mut buzzer = port_a.pa2.into_open_drain_output(&mut port_a.moder, &mut port_a.otyper);
        buzzer.set_high().unwrap_or_default(); //Speaker output is inverted, so high is the default state

        let temp_enc_a = port_a.pa8.into_af_open_drain::<6>(&mut port_a.moder, &mut port_a.otyper, &mut port_a.afrh);
        let temp_enc_b = port_a.pa9.into_af_open_drain::<6>(&mut port_a.moder, &mut port_a.otyper, &mut port_a.afrh);
        let time_enc_a = port_b.pb4.into_af_open_drain::<2>(&mut port_b.moder, &mut port_b.otyper, &mut port_b.afrl);
        let time_enc_b = port_b.pb5.into_af_open_drain::<2>(&mut port_b.moder, &mut port_b.otyper, &mut port_b.afrl);

        let current = port_b.pb1.into_analog(&mut port_b.moder, &mut port_b.pupdr);

        let mut heater = port_a.pa12.into_push_pull_output(&mut port_a.moder, &mut port_a.otyper);
        heater.set_low().unwrap_or_default();
        let mut motor = port_a.pa11.into_open_drain_output(&mut port_a.moder, &mut port_a.otyper);
        motor.set_high().unwrap_or_default();

        let lcd_rs = port_a.pa15.into_push_pull_output(&mut port_a.moder, &mut port_a.otyper);
        let lcd_e = port_b.pb7.into_push_pull_output(&mut port_b.moder, &mut port_b.otyper);
        let lcd_d5 = port_b.pb3.into_push_pull_output(&mut port_b.moder, &mut port_b.otyper);
        let lcd_d6 = port_a.pa10.into_push_pull_output(&mut port_a.moder, &mut port_a.otyper);
        let lcd_d7 = port_b.pb0.into_push_pull_output(&mut port_b.moder, &mut port_b.otyper);
        let lcd_d8 = port_a.pa3.into_push_pull_output(&mut port_a.moder, &mut port_a.otyper);

        let lcd = HD44780::new_4bit(lcd_rs, lcd_e, lcd_d5, lcd_d6, lcd_d7, lcd_d8, delay)
            .and_then(|mut l| {
                l.reset(delay)?;
                Result::Ok(l)
            })
            .and_then(|mut l| {
                l.clear(delay)?;
                Result::Ok(l)
            })
            .and_then(|mut l| {
                l.set_cursor_visibility(Cursor::Invisible, delay)?;
                Result::Ok(l)
            })
            .unwrap();

        let tc_cs = port_a.pa4.into_push_pull_output(&mut port_a.moder, &mut port_a.otyper);
        let tc_sck = port_a.pa5.into_af_push_pull::<5>(&mut port_a.moder, &mut port_a.otyper, &mut port_a.afrl);
        let tc_so = port_a.pa6.into_af_push_pull::<5>(&mut port_a.moder, &mut port_a.otyper, &mut port_a.afrl);
        let tc_si = port_a.pa7.into_af_push_pull::<5>(&mut port_a.moder, &mut port_a.otyper, &mut port_a.afrl);

        let tc_spi: SpiBus = Spi::new(spi, (tc_sck, tc_so, tc_si), 100.kHz(), clocks, apb2);

        Board { cook_btn, lid, cook_ld, buzzer, temp_encoder: (temp_enc_a, temp_enc_b), time_encoder: (time_enc_a, time_enc_b), current, lcd, tc_cs, tc_spi, heater, motor }
    }
}