//! Minimal example with zero tasks
#![deny(unsafe_code)]
#![deny(warnings)]
// IMPORTANT always include this feature gate
#![feature(proc_macro)]
#![feature(proc_macro_gen)]
#![feature(start)]
#![no_std]

extern crate panic_abort;

extern crate cortex_m_rtfm as rtfm; // IMPORTANT always do this rename
extern crate lpc1347; // the device crate

#[macro_use(exception)]
extern crate cortex_m_rt as rt;
use rt::ExceptionFrame;

// import the procedural macro
use rtfm::app;

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

// This macro call indicates that this is a RTFM application
//
// This macro will expand to a `main` function so you don't need to supply
// `main` yourself.
app! {
    // this is the path to the device crate
    device: lpc1347,
}

// The initialization phase.
//
// This runs first and within a *global* critical section. Nothing can preempt
// this function.
fn init(p: init::Peripherals) {
    // This function has access to all the peripherals of the device
    p.device.USART;
}

// The idle loop.
//
// This runs after `init` and has a priority of 0. All tasks can preempt this
// function. This function can never return so it must contain some sort of
// endless loop.
fn idle() -> ! {
    loop {
        // This puts the processor to sleep until there's a task to service
        rtfm::wfi();
    }
}
