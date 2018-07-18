#![deny(overflowing_literals)]
#![feature(proc_macro, proc_macro_gen, lang_items)]
#![no_std]
// For custom start
#![feature(start)]

// Value where the potentiometer value indicates state-change
const ADC_LIMIT: u16 = 800;

// BIAS Pins
const POT_BIAS_PIN: u32 = 8;

// ADC PIN 0
const FROM_POT_PIN: u8 = 0;

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
use lpc::lpc1347::GPIO_PORT;
use rtfm::{app, wfi, Threshold};

// This is the resource, not the type
use lpc::lpc1347::CT16B0 as CT16B0_RES;
use lpc::lpc1347::ADC;

use lpc::adc;
use lpc::gpio;
use lpc::gpio::Port::Port0;
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
        static CT16B0_RES: CT16B0_RES;
        static ADC: ADC;
    },

    tasks: {
        CT16B0: {
            path: clock0_tick,
            priority: 1,
            resources: [ADC, GPIO_PORT, CT16B0_RES],
        },
    }
}

fn init(mut p: init::Peripherals) -> init::LateResources {
    {
        let mut stdout = hio::hstdout().unwrap();
        writeln!(stdout, "Initializing...").unwrap();
    }

    p.device
        .SYSCON
        .sysahbclkctrl
        .modify(|_, w| w.iocon().enable());

    // Configure ADC to use pio0_11 for power on/off
    adc::init(
        &p.device.SYSCON,
        &p.device.ADC,
        FROM_POT_PIN,
        24000000u32,
        true,
        false,
        adc::Capture::Rising,
    );
    // POT
    adc::set_adc_pin(&p.device.IOCON, adc::PinPos::Pin0);

    gpio::init(&p.device.SYSCON, false, false);
    p.device.IOCON.pio0_3.modify(|_, w| w.func().pio0_3_());
    // Special i2c-pins
    p.device
        .IOCON
        .pio0_4
        .modify(|_, w| w.i2cmode().standard_io_functio());
    p.device
        .IOCON
        .pio0_5
        .modify(|_, w| w.i2cmode().standard_io_functio());

    // Initialize GPIO and set pio0_3 to output
    gpio::set_dir(&p.device.GPIO_PORT, Port0, 2, true);
    gpio::set_dir(&p.device.GPIO_PORT, Port0, 3, true);
    gpio::set_dir(&p.device.GPIO_PORT, Port0, 4, true);
    gpio::set_dir(&p.device.GPIO_PORT, Port0, 5, true);
    gpio::set_dir(&p.device.GPIO_PORT, Port0, 6, true);
    gpio::set_dir(&p.device.GPIO_PORT, Port0, 9, true);

    // POT_BIAS (pin 8)
    gpio::set_dir(&p.device.GPIO_PORT, Port0, POT_BIAS_PIN, true);
    gpio::set_pin_value(&p.device.GPIO_PORT, Port0, POT_BIAS_PIN, false);

    // LEDS
    gpio::set_pin_value(&p.device.GPIO_PORT, Port0, 3, true);
    gpio::set_pin_value(&p.device.GPIO_PORT, Port0, 4, true);
    gpio::set_pin_value(&p.device.GPIO_PORT, Port0, 5, true);

    gpio::set_pin_value(&p.device.GPIO_PORT, Port0, 6, true);
    gpio::set_pin_value(&p.device.GPIO_PORT, Port0, 8, false);
    gpio::set_pin_value(&p.device.GPIO_PORT, Port0, 9, true);

    // Clock 0 setup
    // 24MHz systemclock, prescale 24000 and count to 500
    // 2Hz
    timers16::reset_t0(&p.device.CT16B0);
    timers16::init(&p.device.SYSCON, &mut p.core.NVIC, Timer::Timer0);
    timers16::set_interrupt_t0(&p.device.CT16B0, MatchReg::Reg0, true, true, false);
    timers16::set_prescaler_t0(&p.device.CT16B0, 24_000);
    timers16::set_enabled_t0(&p.device.CT16B0, true);
    timers16::set_match_t0(&p.device.CT16B0, MatchReg::Reg0, 500u16);

    {
        let mut stdout = hio::hstdout().unwrap();
        writeln!(stdout, "Done").unwrap();
    }

    init::LateResources {
        GPIO_PORT: p.device.GPIO_PORT,
        CT16B0_RES: p.device.CT16B0,
        ADC: p.device.ADC,
    }
}

fn idle() -> ! {
    loop {
        wfi();
    }
}

/// Example where potentiometer was connected to AD1
///
/// Control a LED with potentiometer

fn clock0_tick(_t: &mut Threshold, r: CT16B0::Resources) {
    timers16::clear_interrupt_t0(&r.CT16B0_RES, MatchReg::Reg0);

    let mut stdout = hio::hstdout().unwrap();
    writeln!(stdout, "Clock 0!").unwrap();

    // Activate BIAS!
    // Check if the potentiometer is turned on

    // Activate POT_BIAS before
    gpio::set_pin_value(&r.GPIO_PORT, Port0, POT_BIAS_PIN, true);

    // Check if the POT is in OFF position
    if adc::read(&r.ADC, FROM_POT_PIN) <= ADC_LIMIT {
        gpio::set_pin_value(&r.GPIO_PORT, Port0, 4, false);
    } else {
        gpio::set_pin_value(&r.GPIO_PORT, Port0, 4, true);
    }

    // Deactivate POT_BIAS
    gpio::set_pin_value(&r.GPIO_PORT, Port0, POT_BIAS_PIN, false);
}
