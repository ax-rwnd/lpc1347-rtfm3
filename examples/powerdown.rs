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
        WWDT: {
            path: wwdt_wakeup,
            priority: 1,
            resources: [GPIO_PORT, CT16B0_RES, WWDT_RES, NVIC, PMU, SYSCON, SCB],
        },
    }
}

fn init(mut p: init::Peripherals) -> init::LateResources {
    // Enable clock for the I/O configuration block
    p.device
        .SYSCON
        .sysahbclkctrl
        .modify(|_, w| w.iocon().enable());

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
        0xFF_FF_FF, // 9375+1023, should give 1 second
        1023,
    );

    // Disable i2c clock
    p.device
        .SYSCON
        .sysahbclkctrl
        .modify(|_, w| w.i2c().disable());

    macro_rules! set_pulldown {
        ($field:ident) => {{
            p.device
                .IOCON
                .$field
                .modify(|_, w| w.mode().pull_down_resistor_e());
        }};
    }
    macro_rules! set_pullup {
        ($field:expr) => {{
            p.device
                .IOCON
                .pio0_$field
                .modify(|_, w| w.mode().pull_up_resistor_ena());
        }};
    }
    macro_rules! set_nopull {
        ($field:ident) => {{
            p.device
                .IOCON
                .$field
                .modify(|_, w| w.mode().inactive_no_pull_do());
        }};
    }
    macro_rules! set_gpio_output {
        ($field:ident) => {{
            gpio::set_dir(&p.device.GPIO_PORT, Port0, $field, true);
        }};
    }
    macro_rules! set_gpio_input {
        ($field:ident) => {{
            gpio::set_dir(&p.device.GPIO_PORT, Port0, $field, false);
        }};
    }
    macro_rules! set_gpio_low {
        ($field:ident) => {{
            gpio::set_pin_value(&p.device.GPIO_PORT, Port0, $field, false);
        }};
    }

    //Setup how a wfi should behave
    //power::sleep(&p.device.PMU, &mut p.core.SCB);

    /*
    power::deep_sleep(
        &p.device.PMU,
        &p.device.SYSCON,
        &mut p.core.SCB,
        false,
        true,
    );
    */
    power::power_down(
        &p.device.PMU,
        &p.device.SYSCON,
        &mut p.core.SCB,
        false,
        true,
    );

    // Set the mainclksel to WWDT
    p.device
        .SYSCON
        .mainclksel
        .modify(|_, w| w.sel().watchdog_oscillator());

    p.device
        .SYSCON
        .pdawakecfg
        .modify(|_, w| w.irc_pd().powered_down());
    p.device
        .SYSCON
        .pdawakecfg
        .modify(|_, w| w.ircout_pd().powered_down());

    gpio::init(&p.device.SYSCON, false, false);

    // Set inputs according to product datasheet footnote on page 42
    //   "[5]
    //   IDD measurements were performed with all pins configured as GPIO outputs driven LOW and
    //   pull-up resistors disabled."
    set_nopull!(reset_pio0_0);
    set_nopull!(pio0_1);
    set_nopull!(pio0_2);
    set_nopull!(pio0_3);
    // 4 and 5 special i2c pins
    set_nopull!(pio0_6);
    set_nopull!(pio0_7);
    set_nopull!(pio0_8);
    set_nopull!(pio0_9);
    set_nopull!(swclk_pio0_10);
    set_nopull!(tdi_pio0_11);
    set_nopull!(tms_pio0_12);
    set_nopull!(tdo_pio0_13);
    set_nopull!(trst_pio0_14);
    set_nopull!(swdio_pio0_15);
    set_nopull!(pio0_16);
    set_nopull!(pio0_17);
    set_nopull!(pio0_18);
    set_nopull!(pio0_19);
    set_nopull!(pio0_20);
    set_nopull!(pio0_21);
    set_nopull!(pio0_22);
    set_nopull!(pio0_23);

    for x in 0..23 {
        set_gpio_output!(x);
        set_gpio_low!(x);
    }

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

    //gpio::toggle_pin_value(&r.GPIO_PORT, Port0, 3);
    //gpio::toggle_pin_value(&r.GPIO_PORT, Port0, 5);

    //let mut stdout = hio::hstdout().unwrap();
    //writeln!(stdout, "WWDT wakeup").unwrap();
}

/*
fn clock0_tick(_t: &mut Threshold, r: CT16B0::Resources) {
    // test
    timers16::clear_interrupt_t0(&r.CT16B0_RES, MatchReg::Reg0);

    //let mut stdout = hio::hstdout().unwrap();
    //writeln!(stdout, "Clock 0!").unwrap();

    //gpio::toggle_pin_value(&r.GPIO_PORT, Port0, 4);
}
*/
