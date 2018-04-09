#![allow(dead_code)]

extern crate lpc1347;
use lpc1347::Interrupt::RIT_IRQ;

/// Initialize the timer
fn init(rit: &lpc1347::RITIMER, clear: bool, debug: bool) {
    rit.ctrl.modify(|_, w| w.ritenclr().bit(clear));
    rit.ctrl.modify(|_, w| w.ritenbr().bit(debug));
}

/// Reset the timer
fn reset(rit: &lpc1347::RITIMER) {
    unsafe {
        rit.counter.modify(|_, w| w.ricounter().bits(0u32));
        rit.counter_h.modify(|_, w| w.ricounter().bits(0u16));
    }
}

/// Turn the timer on or off
fn set_enabled(rit: &lpc1347::RITIMER, nvic: &mut lpc1347::NVIC, enable: bool) {
    rit.ctrl.modify(|_, w| w.riten().bit(enable));
    if enable {
        nvic.enable(RIT_IRQ);
    } else {
        nvic.disable(RIT_IRQ);
    }
}

/// Set the comparison bits
fn set_compare(rit: &lpc1347::RITIMER, value: u64) {
    unsafe {
        rit.compval.modify(|_, w| w.ricomp().bits((value & 0xFFFFFFFF) as u32));
        rit.compval_h.modify(|_, w| w.ricomp().bits(((value >> 16) & 0x0000FFFF) as u16));
    }
}

/// Set the mask bits
fn set_mask(rit: &lpc1347::RITIMER, value: u64) {
    unsafe {
        rit.mask.modify(|_, w| w.rimask().bits((value & 0xFFFFFFFF) as u32));
        rit.mask_h.modify(|_, w| w.rimask().bits(((value >> 16) & 0x0000FFFF) as u16));
    }
}
