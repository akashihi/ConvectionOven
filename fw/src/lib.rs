#![no_std]

//use core::sync::atomic::AtomicUsize;
//use core::sync::atomic::Ordering;
use defmt_rtt as _;

//use panic_probe as _;

pub mod board;
pub mod encoder;
pub mod delay;
pub mod display;
pub mod temp_sensor;
pub mod state;
pub mod buzzer;
pub mod current_sensor;

//#[defmt::panic_handler]
/*fn panic() -> ! {
    cortex_m::asm::udf();
}*/

/*static COUNT: AtomicUsize = AtomicUsize::new(0);
defmt::timestamp!("{=usize}", {
    let n = COUNT.load(Ordering::Relaxed);
    COUNT.store(n + 1, Ordering::Relaxed);
    n
    });

pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}*/