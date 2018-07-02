#![allow(dead_code)]

extern crate lpc1347;

use lpc1347::Interrupt::{CT16B0, CT16B1};

/// Timer selection
#[derive(Copy, Clone)]
pub enum Timer {
    /// First timer
    Timer0,
    /// Second timer
    Timer1,
}

/// Specify a match register
#[derive(Copy, Clone)]
pub enum MatchReg {
    /// Specifies match register 0
    Reg0 = 0,
    /// Specifies match register 1
    Reg1 = 1,
    /// Specifies match register 2
    Reg2 = 2,
    /// Specifies match register 3
    Reg3 = 3,
}

/// Specify PWM-mode
#[derive(Copy, Clone)]
pub enum Control {
    /// Don't do anything
    Nothing = 0x0,
    /// Clear the output
    Clear = 0x1,
    /// Set the output
    Set = 0x2,
    /// Invert the output
    Toggle = 0x3,
}

/// Initialize 16-bit timers
///
/// # Arguments
/// * `timer` - Selects one of the two 16-bit timers
///
/// # Example
/// ```
/// // Initialise the timers
/// timers::init(&p.device.SYSCON, &p.core.NVIC, Timer::Timer0);
/// timers::init(&p.device.SYSCON, &p.core.NVIC, Timer::Timer1);
/// ```
pub fn init(syscon: &lpc1347::SYSCON, nvic: &mut lpc1347::NVIC, timer: Timer) {
    match timer {
        Timer::Timer0 => {
            syscon.sysahbclkctrl.modify(|_, w| w.ct16b0().enable());
            nvic.enable(CT16B0);
        }
        Timer::Timer1 => {
            syscon.sysahbclkctrl.modify(|_, w| w.ct16b1().enable());
            nvic.enable(CT16B1);
        }
    }
}

/// Set a match-register value for the first clock
///
/// # Arguments
/// * `mr` - Selects the MatchReg
/// * `value` - Value to match on
///
/// # Example
/// ```
/// // Set the values to match
/// unsafe { timers::set_match_t0(&p.device.CT16B0, MatchReg::Reg0, 2u16); }
/// unsafe { timers::set_match_t0(&p.device.CT16B0, MatchReg::Reg3, 2u16); }
/// ```
pub fn set_match_t0(ct16b0: &lpc1347::CT16B0, mr: MatchReg, value: u16) {
    unsafe {
        match mr {
            MatchReg::Reg0 => {
                ct16b0.mr[0].modify(|_, w| w.match_().bits(value));
            }
            MatchReg::Reg1 => {
                ct16b0.mr[1].modify(|_, w| w.match_().bits(value));
            }
            MatchReg::Reg2 => {
                ct16b0.mr[2].modify(|_, w| w.match_().bits(value));
            }
            MatchReg::Reg3 => {
                ct16b0.mr[3].modify(|_, w| w.match_().bits(value));
            }
        }
    }
}

/// Set a match-register value for the second clock
///
/// # Arguments
/// * `mr` - Selects the MatchReg
/// * `value` - Value to match on
///
/// # Example
/// ```
/// // Set the values to match
/// unsafe { timers::set_match_t1(&p.device.CT16B1, MatchReg::Reg0, 2u16); }
/// unsafe { timers::set_match_t1(&p.device.CT16B1, MatchReg::Reg3, 2u16); }
/// ```
pub fn set_match_t1(ct16b1: &lpc1347::CT16B1, mr: MatchReg, value: u16) {
    unsafe {
        match mr {
            MatchReg::Reg0 => {
                ct16b1.mr[0].modify(|_, w| w.match_().bits(value));
            }
            MatchReg::Reg1 => {
                ct16b1.mr[1].modify(|_, w| w.match_().bits(value));
            }
            MatchReg::Reg2 => {
                ct16b1.mr[2].modify(|_, w| w.match_().bits(value));
            }
            MatchReg::Reg3 => {
                ct16b1.mr[3].modify(|_, w| w.match_().bits(value));
            }
        }
    }
}

/// Configure an interrupt for the first timer
///
/// # Arguments
/// * `mr` - Selects the MatchReg
/// * `interrupt` - Enable interrupt when MatchReg matches
/// * `reset` - Reset the TC counter when MatchReg matches
/// * `stop` - Stop the counter and disable the timer when MatchReg matches
///
/// # Example
/// ```
/// // Setup timer1 to generate interrupts and to reset the counter on match
/// timers::set_interrupt_t1(&p.device.CT16B1, MatchReg::Reg0, true, true, false);
/// ```
pub fn set_interrupt_t0(
    ct16b0: &lpc1347::CT16B0,
    mr: MatchReg,
    interrupt: bool,
    reset: bool,
    stop: bool,
) {
    match mr {
        MatchReg::Reg0 => {
            ct16b0.mcr.modify(|_, w| w.mr0i().bit(interrupt));
            ct16b0.mcr.modify(|_, w| w.mr0r().bit(reset));
            ct16b0.mcr.modify(|_, w| w.mr0s().bit(stop));
        }
        MatchReg::Reg1 => {
            ct16b0.mcr.modify(|_, w| w.mr1i().bit(interrupt));
            ct16b0.mcr.modify(|_, w| w.mr1r().bit(reset));
            ct16b0.mcr.modify(|_, w| w.mr1s().bit(stop));
        }
        MatchReg::Reg2 => {
            ct16b0.mcr.modify(|_, w| w.mr2i().bit(interrupt));
            ct16b0.mcr.modify(|_, w| w.mr2r().bit(reset));
            ct16b0.mcr.modify(|_, w| w.mr2s().bit(stop));
        }
        MatchReg::Reg3 => {
            ct16b0.mcr.modify(|_, w| w.mr3i().bit(interrupt));
            ct16b0.mcr.modify(|_, w| w.mr3r().bit(reset));
            ct16b0.mcr.modify(|_, w| w.mr3s().bit(stop));
        }
    }
}


/// Configure an interrupt for the second timer
///
/// # Arguments
/// * `mr` - Selects the MatchReg
/// * `interrupt` - Enable interrupt when MatchReg matches
/// * `reset` - Reset the TC counter when MatchReg matches
/// * `stop` - Stop the counter and disable the timer when MatchReg matches
///
/// # Example
/// ```
/// // Setup timer1 to generate interrupts and to reset the counter on match
/// timers::set_interrupt_t1(&p.device.CT16B1, MatchReg::Reg0, true, true, false);
/// ```
pub fn set_interrupt_t1(
    ct16b1: &lpc1347::CT16B1,
    mr: MatchReg,
    interrupt: bool,
    reset: bool,
    stop: bool,
) {
    match mr {
        MatchReg::Reg0 => {
            ct16b1.mcr.modify(|_, w| w.mr0i().bit(interrupt));
            ct16b1.mcr.modify(|_, w| w.mr0r().bit(reset));
            ct16b1.mcr.modify(|_, w| w.mr0s().bit(stop));
        }
        MatchReg::Reg1 => {
            ct16b1.mcr.modify(|_, w| w.mr1i().bit(interrupt));
            ct16b1.mcr.modify(|_, w| w.mr1r().bit(reset));
            ct16b1.mcr.modify(|_, w| w.mr1s().bit(stop));
        }
        MatchReg::Reg2 => {
            ct16b1.mcr.modify(|_, w| w.mr2i().bit(interrupt));
            ct16b1.mcr.modify(|_, w| w.mr2r().bit(reset));
            ct16b1.mcr.modify(|_, w| w.mr2s().bit(stop));
        }
        MatchReg::Reg3 => {
            ct16b1.mcr.modify(|_, w| w.mr3i().bit(interrupt));
            ct16b1.mcr.modify(|_, w| w.mr3r().bit(reset));
            ct16b1.mcr.modify(|_, w| w.mr3s().bit(stop));
        }
    }
}

/// Clear interrupt
///
/// # Arguments
/// * `mr` - Selects the MatchReg
///
/// # Example
/// ```
/// // Clear the timer interrupt
/// timers::clear_interrupt_t1(&p.device.CT16B1, MatchReg::Reg0);
/// ```
pub fn clear_interrupt_t0(
    ct16b0: &lpc1347::CT16B0,
    mr: MatchReg,
) {
    unsafe {
        ct16b0.ir.write(|w| w.bits(1 << mr as u32));
    }
}

/// Clear interrupt
///
/// # Arguments
/// * `mr` - Selects the MatchReg
///
/// # Example
/// ```
/// // Clear the timer interrupt
/// timers::clear_interrupt_t1(&p.device.CT16B1, MatchReg::Reg0);
/// ```
pub fn clear_interrupt_t1(
    ct16b1: &lpc1347::CT16B1,
    mr: MatchReg,
) {
    unsafe {
        ct16b1.ir.write(|w| w.bits(1 << mr as u32));
    }
}

/// Set a prescale-register value for the first clock
///
/// # Example
/// ```
/// // Set timer1 prescale register to no-prescaler
/// timers::set_prescaler_t0(&p.device.CT16B0, 0);
/// ```
pub fn set_prescaler_t0(ct16b0: &lpc1347::CT16B0, value: u16) {
    unsafe {
            ct16b0.pr.modify(|_, w| w.pcval().bits(value));
    }
}

/// Set a prescale-register value for the second clock
///
/// # Example
/// ```
/// // Set timer1 prescale register to no-prescaler
/// timers::set_prescaler_t1(&p.device.CT16B1, 0);
/// ```
pub fn set_prescaler_t1(ct16b1: &lpc1347::CT16B1, value: u16) {
    unsafe {
            ct16b1.pr.modify(|_, w| w.pcval().bits(value));
    }
}

/// Turn timer one on or off
///
/// # Arguments
/// * `enabled` - true enables
///
/// # Example
/// ```
/// // Enable first timer
/// timers::set_enabled_t0(&p.device.CT16B0, true);
/// ```
pub fn set_enabled_t0(ct16b0: &lpc1347::CT16B0, enabled: bool) {
    ct16b0.tcr.modify(|_, w| w.cen().bit(enabled));
}

/// Turn timer two on or off
///
/// # Arguments
/// * `enabled` - true enables
///
/// # Example
/// ```
/// // Enable second timer
/// timers::set_enabled_t1(&p.device.CT16B1, true);
/// ```
pub fn set_enabled_t1(ct16b1: &lpc1347::CT16B1, enabled: bool) {
    ct16b1.tcr.modify(|_, w| w.cen().bit(enabled));
}

/// Reset timer one
///
/// # Example
/// ```
/// // Reset the first timer
/// timers::reset_t0(&p.device.CT16B0);
/// ```
pub fn reset_t0(ct16b0: &lpc1347::CT16B0) {
    ct16b0.tcr.modify(|_, w| w.crst().bit(true));
}

/// Reset timer two
///
/// # Example
/// ```
/// // Reset the second timer
/// timers::reset_t1(&p.device.CT16B1);
/// ```
pub fn reset_t1(ct16b1: &lpc1347::CT16B1) {
    ct16b1.tcr.modify(|_, w| w.crst().bit(true));
}

/// Configure pins to use for PWM output
///
/// # Arguments
/// * `timer` - Selects one of the two 16-bit timers
/// * `mr` - Match register to activate GPIO pins for
///
/// # Example
/// ```
/// // Enable Matchreg 2 GPIO pins for the first timer
/// unsafe { timers::set_pwm_output_pin(&p.device.IOCON, Timer::Timer0, MatchReg::Reg1); }
/// ```
pub unsafe fn set_pwm_output_pin(iocon: &lpc1347::IOCON, timer: Timer, mr: MatchReg) {
    match timer {
        Timer::Timer0 => match mr {
            MatchReg::Reg0 => {
                iocon.pio0_18.modify(|_, w| w.func().bits(0x2));
                iocon.pio0_18.modify(|_, w| w.mode().bits(0x2));
            }
            MatchReg::Reg1 => {
                iocon.pio0_19.modify(|_, w| w.func().bits(0x2));
                iocon.pio0_19.modify(|_, w| w.mode().bits(0x2));
            }
            MatchReg::Reg2 => {
                iocon.pio0_1.modify(|_, w| w.func().bits(0x2));
                iocon.pio0_1.modify(|_, w| w.mode().bits(0x2));
            }
            MatchReg::Reg3 => {
                iocon.tdi_pio0_11.modify(|_, w| w.func().bits(0x3));
                iocon.tdi_pio0_11.modify(|_, w| w.mode().bits(0x2));
            }
        },
        Timer::Timer1 => match mr {
            MatchReg::Reg0 => {
                iocon.tdo_pio0_13.modify(|_, w| w.func().bits(0x3));
                iocon.tdo_pio0_13.modify(|_, w| w.mode().bits(0x2));
            }
            MatchReg::Reg1 => {
                iocon.trst_pio0_14.modify(|_, w| w.func().bits(0x3));
                iocon.trst_pio0_14.modify(|_, w| w.mode().bits(0x2));
            }
            MatchReg::Reg2 => {
                iocon.swdio_pio0_15.modify(|_, w| w.func().bits(0x3));
                iocon.swdio_pio0_15.modify(|_, w| w.mode().bits(0x2));
            }
            MatchReg::Reg3 => {
                iocon.pio0_16.modify(|_, w| w.func().bits(0x2));
                iocon.pio0_16.modify(|_, w| w.mode().bits(0x2));
            }
        },
    }
}

/// Set options for PWM on timer one
///
/// # Arguments
/// * `mr` - Select match-register
/// * `control` - Select behaviour when a match occurs
/// * `mat` - The initial value written into external match register `em`
/// * `enabled` - Enable or disable PWM
///
/// # Example
/// ```
/// // Setup PWM which toggles an initial value of 1
/// timers::set_pwm_opts_t0(
///     &p.device.CT16B0,
///     timers16::MatchReg::Reg0,
///     timers16::Control::Toggle,
///     true,
///     true
/// );
/// ```
pub fn set_pwm_opts_t0(
    ct16b0: &lpc1347::CT16B0,
    mr: MatchReg,
    control: Control,
    mat: bool,
    enabled: bool,
) {
    match mr {
        MatchReg::Reg0 => {
            ct16b0.emr.modify(|_, w| w.emc0().bits(control as u8));
            ct16b0.emr.modify(|_, w| w.em0().bit(mat));
            ct16b0.pwmc.modify(|_, w| w.pwmen0().bit(enabled));
        }
        MatchReg::Reg1 => {
            ct16b0.emr.modify(|_, w| w.emc1().bits(control as u8));
            ct16b0.emr.modify(|_, w| w.em1().bit(mat));
            ct16b0.pwmc.modify(|_, w| w.pwmen1().bit(enabled));
        }
        MatchReg::Reg2 => {
            ct16b0.emr.modify(|_, w| w.emc2().bits(control as u8));
            ct16b0.emr.modify(|_, w| w.em2().bit(mat));
            ct16b0.pwmc.modify(|_, w| w.pwmen2().bit(enabled));
        }
        MatchReg::Reg3 => {
            ct16b0.emr.modify(|_, w| w.emc3().bits(control as u8));
            ct16b0.emr.modify(|_, w| w.em3().bit(mat));
            ct16b0.pwmc.modify(|_, w| w.pwmen3().bit(enabled));
        }
    }
}

/// Set options for PWM on timer two
///
/// # Arguments
/// * `mr` - Select match-register
/// * `control` - Select behaviour when a match occurs
/// * `mat` - The initial value written into external match register `em`
/// * `enabled` - Enable or disable PWM
///
/// # Example
/// ```
/// // Setup PWM which toggles an initial value of 1
/// timers::set_pwm_opts_t1(
///     &p.device.CT16B1,
///     timers16::MatchReg::Reg0,
///     timers16::Control::Toggle,
///     true,
///     true
/// );
/// ```
pub fn set_pwm_opts_t1(
    ct16b1: &lpc1347::CT16B1,
    mr: MatchReg,
    control: Control,
    mat: bool,
    enabled: bool,
) {
    match mr {
        MatchReg::Reg0 => {
            ct16b1.emr.modify(|_, w| w.emc0().bits(control as u8));
            ct16b1.emr.modify(|_, w| w.em0().bit(mat));
            ct16b1.pwmc.modify(|_, w| w.pwmen0().bit(enabled));
        }
        MatchReg::Reg1 => {
            ct16b1.emr.modify(|_, w| w.emc1().bits(control as u8));
            ct16b1.emr.modify(|_, w| w.em1().bit(mat));
            ct16b1.pwmc.modify(|_, w| w.pwmen1().bit(enabled));
        }
        MatchReg::Reg2 => {
            ct16b1.emr.modify(|_, w| w.emc2().bits(control as u8));
            ct16b1.emr.modify(|_, w| w.em2().bit(mat));
            ct16b1.pwmc.modify(|_, w| w.pwmen2().bit(enabled));
        }
        MatchReg::Reg3 => {
            ct16b1.emr.modify(|_, w| w.emc3().bits(control as u8));
            ct16b1.emr.modify(|_, w| w.em3().bit(mat));
            ct16b1.pwmc.modify(|_, w| w.pwmen3().bit(enabled));
        }
    }
}
