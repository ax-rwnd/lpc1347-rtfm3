#![deny(overflowing_literals)]
#![feature(proc_macro, proc_macro_gen, lang_items)]
#![no_std]

// #TODO For testing custom start
#![feature(start)]

extern crate panic_abort;

extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate cortex_m_semihosting;
extern crate lpc1347_rtfm3 as lpc;

// #TODO cortex-m-rt most likely not needed here
#[macro_use(exception)]
extern crate cortex_m_rt as rt;
use rt::ExceptionFrame;

use rtfm::{app, Threshold, wfi};
use cortex_m_semihosting::hio;
use core::fmt::Write;
use lpc::lpc1347;
use lpc::lpc1347::{GPIO_PORT};

// This is the resource, not the type
use lpc::lpc1347::CT16B0 as CT16B0_RES;

use lpc::gpio;
use lpc::timers16;
use lpc::timers16::{Timer, MatchReg};

// #TODO Manual start lang item
#[start]
fn main_start(_argc: isize, _argv: *const *const u8) -> isize {
    main();

    0
}

// define the default exception handler
exception!(*, default_handler);
fn default_handler(irqn: i16) {
    panic!("unhandled exception (IRQn={})", irqn);
}


// define the hard fault handler
exception!(HardFault, hard_fault);
fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}


app! {
    device: lpc1347,

    resources: {
        static GPIO_PORT: GPIO_PORT;
        static CT16B0_RES: CT16B0_RES;
    },

    tasks: {
        CT16B0: {
            path: clock0_tick,
            priority: 1,
            resources: [GPIO_PORT, CT16B0_RES],
        },
    }
}

fn init(mut p: init::Peripherals) -> init::LateResources {
    {
        let mut stdout = hio::hstdout().unwrap();
        writeln!(stdout, "Initializing...").unwrap();
    }

    p.device.SYSCON.sysahbclkctrl.modify(|_, w| w.iocon().enable());

    // Initialize GPIO and set pio0_3 to output
    gpio::init(&p.device.SYSCON, false, false);
    gpio::set_dir(&p.device.GPIO_PORT, gpio::Port::Port0, 3, true);
    gpio::set_dir(&p.device.GPIO_PORT, gpio::Port::Port0, 4, true);
    gpio::set_dir(&p.device.GPIO_PORT, gpio::Port::Port0, 5, true);
    gpio::set_pin_value(&p.device.GPIO_PORT, gpio::Port::Port0, 5, true);


    // Clock 0 setup
    // 24MHz systemclock, prescale 24000 and count to 1000
    // 1Hz blink
    timers16::reset_t0(&p.device.CT16B0);
    timers16::init(&p.device.SYSCON, &mut p.core.NVIC, Timer::Timer0);
    timers16::set_interrupt_t0(&p.device.CT16B0, MatchReg::Reg0, true, true, false);
    timers16::set_prescaler_t0(&p.device.CT16B0, 24_000);
    timers16::set_enabled_t0(&p.device.CT16B0, true);
    timers16::set_match_t0(&p.device.CT16B0,  MatchReg::Reg0, 1000u16);

    {
        let mut stdout = hio::hstdout().unwrap();
        writeln!(stdout, "Done").unwrap();
    }

    init::LateResources {
        GPIO_PORT: p.device.GPIO_PORT,
        CT16B0_RES: p.device.CT16B0
    }
}

fn idle() -> ! {
    loop {
        wfi();
    }
}

fn clock0_tick(_t: &mut Threshold, r: CT16B0::Resources) {

    timers16::clear_interrupt_t0(&r.CT16B0_RES, MatchReg::Reg0);

    let mut stdout = hio::hstdout().unwrap();
    writeln!(stdout, "Clock 0!").unwrap();

    r.GPIO_PORT.not[0].write(|w| w.notp3().bit(true));
    r.GPIO_PORT.not[0].write(|w| w.notp4().bit(true));
    r.GPIO_PORT.not[0].write(|w| w.notp5().bit(true));
}
