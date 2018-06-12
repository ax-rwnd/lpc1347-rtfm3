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

use lpc::gpio;
use lpc::timers16;
use lpc::timers16::{Timer16, MatchReg};

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
    },

    tasks: {
        CT16B0: {
            path: clock0_tick,
            priority: 1,
            resources: [GPIO_PORT],
        },
    }
}

fn init(mut p: init::Peripherals) -> init::LateResources {
    {
        let mut stdout = hio::hstdout().unwrap();
        let _ = writeln!(stdout, "Initializing...");
    }

    // Initialize GPIO and set pio0_7 to output
    gpio::init(&p.device.SYSCON, false, false);
    gpio::set_dir(&p.device.GPIO_PORT, gpio::Port::Port0, 7, true);


    // Clock 0 setup
    timers16::reset(&p.device.CT16B0, &p.device.CT16B1, Timer16::Timer0);
    timers16::init(&p.device.SYSCON, &mut p.core.NVIC, Timer16::Timer0);
    timers16::set_interrupt(&p.device.CT16B0, &p.device.CT16B1, Timer16::Timer0, MatchReg::Reg0, true);
    timers16::set_enabled(&p.device.CT16B0, &p.device.CT16B1, Timer16::Timer0, true);
    timers16::set_match(&p.device.CT16B0, &p.device.CT16B1, Timer16::Timer0, MatchReg::Reg0, 2u16);

    {
        let mut stdout = hio::hstdout().unwrap();
        let _ = writeln!(stdout, "Done");
    }

    init::LateResources {
        GPIO_PORT: p.device.GPIO_PORT,
    }
}

fn idle() -> ! {
    loop {
        wfi();
    }
}

fn clock0_tick(_t: &mut Threshold, r: CT16B0::Resources) {
    let mut stdout = hio::hstdout().unwrap();
    let _ = writeln!(stdout, "Clock 0!");
    r.GPIO_PORT.not[0].write(|w| w.notp7().bit(true));
}
