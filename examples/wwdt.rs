#![deny(overflowing_literals)]
#![feature(proc_macro, proc_macro_gen, lang_items)]
#![no_std]
// For custom start
#![feature(start)]

extern crate panic_abort;

extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate cortex_m_semihosting;
extern crate lpc1347_rtfm3 as lpc;

#[macro_use(exception)]
extern crate cortex_m_rt as rt;
use rt::ExceptionFrame;

use core::fmt::Write;
use cortex_m_semihosting::hio;
use lpc::lpc1347;
use lpc::lpc1347::CT16B0 as CT16B0_RES;
use lpc::lpc1347::WWDT as WWDT_RES;
use lpc::lpc1347::{GPIO_PORT, NVIC, PMU, SCB, SYSCON};
use rtfm::{app, wfi, Threshold};

use lpc::clock;
use lpc::gpio;
use lpc::gpio::Port::Port0;
use lpc::power;
use lpc::timers16;
use lpc::timers16::{MatchReg, Timer};

// Manual start lang item
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
        static PMU: PMU;
        static WWDT_RES: WWDT_RES;
        static SYSCON: SYSCON;
        static SCB: SCB;
        static NVIC: NVIC;
        static CT16B0_RES: CT16B0_RES;
    },

    tasks: {
        CT16B0: {
            path: clock0_tick,
            priority: 1,
            resources: [GPIO_PORT, CT16B0_RES],
        },
        WWDT: {
            path: wwdt_wakeup,
            priority: 1,
            resources: [GPIO_PORT, CT16B0_RES, WWDT_RES, NVIC, PMU, SYSCON, SCB],
        },
    }
}

fn init(mut p: init::Peripherals) -> init::LateResources {
    {
        let mut stdout = hio::hstdout().unwrap();
        writeln!(stdout, "Initializing...").unwrap();
    }

    // Enable clock for the I/O configuration block
    p.device
        .SYSCON
        .sysahbclkctrl
        .modify(|_, w| w.iocon().enable());

    // Initialize GPIO and set pio0_3 to output
    gpio::init(&p.device.SYSCON, false, false);
    gpio::set_dir(&p.device.GPIO_PORT, Port0, 3, true);
    gpio::set_dir(&p.device.GPIO_PORT, Port0, 4, true);
    gpio::set_dir(&p.device.GPIO_PORT, Port0, 5, true);

    gpio::set_pin_value(&p.device.GPIO_PORT, Port0, 5, true);

    // Setup the WWDT (Window Watch Dog Timer)
    // 600 kHz / 2 * (63 + 1) = 9,375kHz
    clock::wwdt_init(
        &mut p.core.NVIC,
        &p.device.SYSCON,
        &p.device.WWDT,
        true,
        lpc1347::wwdt::clksel::CLKSELW::WATCHDOG_OSCILLATOR_,
        lpc1347::syscon::wdtoscctrl::FREQSELW::_0_6_MHZ,
        64,
    );

    clock::wwdt_configure(
        &p.device.WWDT,
        true,
        true,
        true,
        9375, // 9375+1023, should give 1 second
        1023,
    );

    // Clock 0 setup
    // 24MHz systemclock, prescale 24000 and count to 1000
    // 1Hz blink
    timers16::reset_t0(&p.device.CT16B0);
    timers16::init(&p.device.SYSCON, &mut p.core.NVIC, Timer::Timer0);
    timers16::set_interrupt_t0(&p.device.CT16B0, MatchReg::Reg0, true, true, false);
    timers16::set_prescaler_t0(&p.device.CT16B0, 24_000);
    timers16::set_enabled_t0(&p.device.CT16B0, true);
    timers16::set_match_t0(&p.device.CT16B0, MatchReg::Reg0, 1000u16);

    {
        let mut stdout = hio::hstdout().unwrap();
        writeln!(stdout, "Done").unwrap();
    }

    // Setup how a wfi should behave
    power::sleep(&p.device.PMU, &mut p.core.SCB);

    init::LateResources {
        GPIO_PORT: p.device.GPIO_PORT,
        WWDT_RES: p.device.WWDT,
        PMU: p.device.PMU,
        SYSCON: p.device.SYSCON,
        SCB: p.core.SCB,
        CT16B0_RES: p.device.CT16B0,
        NVIC: p.core.NVIC,
    }
}

fn idle() -> ! {
    loop {
        wfi();
    }
}

fn wwdt_wakeup(_t: &mut Threshold, r: WWDT::Resources) {
    // Clear the WWDT Interrupt flag
    clock::wwdt_intclear(&r.WWDT_RES);

    // Feed the watchdog
    clock::wwdt_feed(&r.WWDT_RES);

    gpio::toggle_pin_value(&r.GPIO_PORT, Port0, 3);
    gpio::toggle_pin_value(&r.GPIO_PORT, Port0, 5);

    //let mut stdout = hio::hstdout().unwrap();
    //writeln!(stdout, "WWDT wakeup").unwrap();
}

fn clock0_tick(_t: &mut Threshold, r: CT16B0::Resources) {
    timers16::clear_interrupt_t0(&r.CT16B0_RES, MatchReg::Reg0);

    //let mut stdout = hio::hstdout().unwrap();
    //writeln!(stdout, "Clock 0!").unwrap();

    gpio::toggle_pin_value(&r.GPIO_PORT, Port0, 4);
}
