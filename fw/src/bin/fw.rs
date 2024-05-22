#![no_main]
#![no_std]

extern crate panic_halt;

use fw as _;

#[rtic::app(device = stm32f3xx_hal::pac, dispatchers = [FMC])]
mod app {
    use cortex_m::peripheral::NVIC;
    use stm32f3xx_hal::prelude::*;
    use dwt_systick_monotonic::DwtSystick;
    use stm32f3xx_hal::gpio::Edge;
    use stm32f3xx_hal::timer::{Timer, Event};
    use stm32f3xx_hal::adc;
    use fw::board::{Board, CookBtn, Lid};
    use fw::encoder::{EncoderReaderTIM1, EncoderReaderTIM3};
    use dwt_systick_monotonic::ExtU32;
    use stm32f3xx_hal::adc::{VoltageInternalReference};
    use stm32f3xx_hal::pac::{TIM2, TIM6, TIM15};
    use fw::buzzer::BuzzerManager;
    use fw::current_sensor::{CurrentReader, CurrentSensor};
    use fw::delay::TimDelay;
    use fw::display::LcdDisplay;
    use fw::state::manager::StateManager;
    use fw::state::OvenControlHardware;
    use fw::temp_sensor::TempSensor;

    #[monotonic(binds = SysTick, default = true)]
    type SysMono = DwtSystick<64_000_000>;

    #[shared]
    struct Shared {
        cook_btn_debounce: bool,
        lid_debounce: bool,
        lid: Lid,
        state: StateManager,
    }

    #[local]
    struct Local {
        cook_btn: CookBtn,
        current_timer: Timer<TIM2>,
        current_reader: CurrentReader,
        state_poll_timer: Timer<TIM6>,
        pid_timer: Timer<TIM15>
    }

    #[init]
    fn init(mut cx: init::Context) -> (Shared, Local, init::Monotonics) {
        //Set up the system clock
        let mut rcc = cx.device.RCC.constrain();
        let mut flash = cx.device.FLASH.constrain();
        let clocks = rcc.cfgr.sysclk(64.MHz()).freeze(&mut flash.acr);
        let mut syscfg = cx.device.SYSCFG.constrain(&mut rcc.apb2);
        let mut exti = cx.device.EXTI;

        //Set up monotonic timer
        cx.core.DCB.enable_trace();
        cx.core.DWT.enable_cycle_counter();
        let mono = DwtSystick::new(&mut cx.core.DCB, cx.core.DWT, cx.core.SYST, clocks.hclk().0);

        //Configure TIM7 as a delay provider
        let mut delay = TimDelay::new(cx.device.TIM7, clocks, &mut rcc.apb1);

        //Configure board
        let mut board = Board::new(cx.device.GPIOA, cx.device.GPIOB, cx.device.SPI1, &mut rcc.ahb, &mut rcc.apb2, clocks, &mut delay);

        //Setup event timers
        let mut state_poll_timer = Timer::new(cx.device.TIM6, clocks, &mut rcc.apb1);
        state_poll_timer.start(100.milliseconds());
        state_poll_timer.enable_interrupt(Event::Update);
        let mut pid_timer = Timer::new(cx.device.TIM15, clocks, &mut rcc.apb2);
        pid_timer.start(10.seconds());
        pid_timer.enable_interrupt(Event::Update);

        //Configure temperature encoder
        let tim1 = Timer::new(cx.device.TIM1, clocks, &mut rcc.apb2);
        let temp_encoder = EncoderReaderTIM1::new(tim1.free(), 10, 50);

        //Configure time encoder
        let tim3 = Timer::new(cx.device.TIM3, clocks, &mut rcc.apb1);
        let time_encoder = EncoderReaderTIM3::new(tim3.free(), 0, 180);

        //Configure drive current sensor
        let mut current_timer = Timer::new(cx.device.TIM2, clocks, &mut rcc.apb1);
        current_timer.start(10.milliseconds());
        current_timer.enable_interrupt(Event::Update);
        let mut adc_common_current = adc::CommonAdc::new(cx.device.ADC1_2, &clocks, &mut rcc.ahb);
        let mut adc_pair_current = (cx.device.ADC1, cx.device.ADC2);
        let v_in = VoltageInternalReference::new(&mut adc_common_current, &mut adc_pair_current);
        let adc_current = adc::Adc::new(adc_pair_current.0, adc::config::Config::default(), &clocks, &adc_common_current).into_oneshot();

        //Configure the interrrupts
        syscfg.select_exti_interrupt_source(&board.cook_btn);
        board.cook_btn.trigger_on_edge(&mut exti, Edge::Rising);
        board.cook_btn.enable_interrupt(&mut exti);
        syscfg.select_exti_interrupt_source(&board.lid);
        board.lid.trigger_on_edge(&mut exti, Edge::Falling);
        board.lid.enable_interrupt(&mut exti);

        unsafe {
            NVIC::unmask(board.cook_btn.interrupt());
            NVIC::unmask(board.lid.interrupt());
            NVIC::unmask(current_timer.interrupt());
            NVIC::unmask(state_poll_timer.interrupt());
            NVIC::unmask(pid_timer.interrupt());
        };

        //Configure control structs
        let display_manager = LcdDisplay::new(board.lcd, delay);
        let temp_sensor = TempSensor::new(board.tc_cs, board.tc_spi);
        let buzzer = BuzzerManager::new(board.buzzer);
        let current_reader = CurrentReader::new(adc_current, v_in, board.current);
        let current_sensor = CurrentSensor::new();
        let control_hardware = OvenControlHardware{display: display_manager, buzzer, cook_ld: board.cook_ld, heater: board.heater, motor: board.motor};
        let state_manager = StateManager::new(control_hardware, current_sensor, temp_encoder, time_encoder, temp_sensor);

        let shared = Shared {
            cook_btn_debounce: false,
            lid_debounce: false,
            lid: board.lid,
            state: state_manager
        };

        let local = Local {
            cook_btn: board.cook_btn,
            current_timer,
            current_reader,
            state_poll_timer,
            pid_timer
        };

        (shared, local, init::Monotonics(mono))
    }

    #[task(binds = TIM2, local = [current_timer, current_reader], shared=[state])]
    fn current_timer_handle(mut cx: current_timer_handle::Context) {
        // TODO ADC should be triggered by timer directly,
        // and use DMA to read both channels in sequence
        // but i'm lazy and it is fast enough to not to cause any issues
        cx.local.current_timer.clear_events();
        cx.shared.state.lock(|state| state.adc_poll(cx.local.current_reader.read()));
    }

    #[task(binds = EXTI0, local = [cook_btn], shared = [cook_btn_debounce,state])]
    fn cook_btn_handler(cx: cook_btn_handler::Context) {
        cx.local.cook_btn.clear_interrupt();
        let cook_btn_handler::SharedResources { mut cook_btn_debounce, mut state } = cx.shared;
        cook_btn_debounce.lock(|debounce|
            if !*debounce {
                *debounce = true;
                state.lock(|s| s.on_cook_btn());
                cook_btn_debounce::spawn_after(300.millis()).unwrap();
            }
        );
    }

    #[task(shared = [cook_btn_debounce])]
    fn cook_btn_debounce(mut cx: cook_btn_debounce::Context) {
        cx.shared.cook_btn_debounce.lock(|debounce| *debounce = false);
    }

    #[task(binds = EXTI9_5, shared = [lid, lid_debounce, state])]
    fn lid_handler(cx: lid_handler::Context) {
        let lid_handler::SharedResources { mut lid, mut lid_debounce, mut state } = cx.shared;
        lid.lock(|l| l.clear_interrupt());
        lid_debounce.lock(|debounce|
            if !*debounce {
                *debounce = true;
                state.lock(|s| s.enc_poll(false));
                lid_debounce::spawn_after(100.millis()).unwrap();
            }
        );
    }

    #[task(shared = [lid_debounce])]
    fn lid_debounce(mut cx: lid_debounce::Context) {
        cx.shared.lid_debounce.lock(|debounce| *debounce = false);
    }

    #[task(binds = TIM6_DACUNDER, local = [state_poll_timer], shared = [state, lid])]
    fn state_timer_handler(cx: state_timer_handler::Context) {
        cx.local.state_poll_timer.clear_events();
        let state_timer_handler::SharedResources { state, lid } = cx.shared;
        (state, lid).lock(|state, lid| {
            state.enc_poll(lid.is_low().unwrap_or(false));
        });
    }

    #[task(binds = TIM1_BRK_TIM15, local = [pid_timer], shared = [state])]
    fn pid_timer_handler(mut cx: pid_timer_handler::Context) {
        cx.local.pid_timer.clear_events();
        cx.shared.state.lock(|state| state.pid_poll())
    }
}