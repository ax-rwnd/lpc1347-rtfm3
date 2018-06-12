#![allow(dead_code)]

extern crate lpc1347;

use lpc1347::Interrupt::{CT16B0, CT16B1};

/// Timer selection
#[derive(Copy, Clone)]
pub enum Timer16 {
    /// First timer
    Timer0,
    /// Second timer
    Timer1,
}

/// Specify a match register
#[derive(Copy, Clone)]
pub enum MatchReg {
    /// Specifies match register 0
    Reg0,
    /// Specifies match register 1
    Reg1,
    /// Specifies match register 2
    Reg2,
    /// Specifies match register 3
    Reg3,
}

/// Initialize 16-bit timers
///
/// # Arguments
/// * `timer` - Selects one of the two 16-bit timers
///
/// # Example
/// ```
/// // Setup timer0 to generate interrupts
/// timers::reset(&p.CT16B0, Timer16::Timer0);
/// timers::init(&p.SYSCON, &p.NVIC, Timer16::Timer0);
/// timers::set_interrupt(&p.CT16B0, &p.CT16B1, Timer16::Timer0, MatchReg::Reg0, true);
/// timers::set_enabled(&p.CT16B0, &p.CT16B1, Timer16::Timer0, true);
/// unsafe { timers::set_match(&p.CT16B0, &p.CT16B1, Timer16::Timer0, MatchReg::Reg0, 2u16); }
/// ```
pub fn init(syscon: &lpc1347::SYSCON, nvic: &mut lpc1347::NVIC, timer: Timer16) {
    match timer {
        Timer16::Timer0 => {
            syscon.sysahbclkctrl.modify(|_, w| w.ct16b0().bit(true));
            nvic.enable(CT16B0);
        }
        Timer16::Timer1 => {
            syscon.sysahbclkctrl.modify(|_, w| w.ct16b1().bit(true));
            nvic.enable(CT16B1);
        }
    }
}

/// Enable or disable interrupts for a timer
pub fn set_interrupt(
    ct16b0: &lpc1347::CT16B0,
    ct16b1: &lpc1347::CT16B1,
    timer: Timer16,
    mr: MatchReg,
    enabled: bool,
) {
    match timer {
        Timer16::Timer0 => match mr {
            MatchReg::Reg0 => {
                ct16b0.mcr.modify(|_, w| w.mr0i().bit(enabled));
                ct16b0.mcr.modify(|_, w| w.mr0r().bit(enabled));
            }
            MatchReg::Reg1 => {
                ct16b0.mcr.modify(|_, w| w.mr1i().bit(enabled));
            }
            MatchReg::Reg2 => {
                ct16b0.mcr.modify(|_, w| w.mr2i().bit(enabled));
            }
            MatchReg::Reg3 => {
                ct16b0.mcr.modify(|_, w| w.mr3i().bit(enabled));
            }
        },

        Timer16::Timer1 => match mr {
            MatchReg::Reg0 => {
                ct16b1.mcr.modify(|_, w| w.mr0i().bit(enabled));
            }
            MatchReg::Reg1 => {
                ct16b1.mcr.modify(|_, w| w.mr1i().bit(enabled));
            }
            MatchReg::Reg2 => {
                ct16b1.mcr.modify(|_, w| w.mr2i().bit(enabled));
            }
            MatchReg::Reg3 => {
                ct16b1.mcr.modify(|_, w| w.mr3i().bit(enabled));
            }
        },
    }
}

/// Enable or disable 16-bit timers
pub fn set_enabled(
    ct16b0: &lpc1347::CT16B0,
    ct16b1: &lpc1347::CT16B1,
    timer: Timer16,
    enabled: bool,
) {
    match timer {
        Timer16::Timer0 => {
            ct16b0.tcr.modify(|_, w| w.cen().bit(enabled));
        }
        Timer16::Timer1 => {
            ct16b1.tcr.modify(|_, w| w.cen().bit(enabled));
        }
    }
}

/// Reset a 16-bit timer
pub fn reset(ct16b0: &lpc1347::CT16B0, ct16b1: &lpc1347::CT16B1, timer: Timer16) {
    // TODO: these should write 0x02, but that's reserved?
    match timer {
        Timer16::Timer0 => {
            ct16b0.tcr.modify(|_, w| w.crst().bit(true));
        }
        Timer16::Timer1 => {
            ct16b1.tcr.modify(|_, w| w.crst().bit(true));
        }
    }
}

/// Cause a blocking delay for some ticks
// pub fn delay_ticks(_p: &Peripherals, _timer: Timer16, _delay: u16) {
//     panic!("not implemented");
// }

/// Set the match register
pub fn set_match(
    ct16b0: &lpc1347::CT16B0,
    ct16b1: &lpc1347::CT16B1,
    timer: Timer16,
    reg: MatchReg,
    value: u16,
) {
    unsafe {
        match timer {
            Timer16::Timer0 => match reg {
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
            },

            Timer16::Timer1 => match reg {
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
            },
        }
    }
}

/// Set the prescaler register
pub fn set_prescaler(
    ct16b0: &lpc1347::CT16B0,
    ct16b1: &lpc1347::CT16B1,
    timer: Timer16,
    value: u16,
) {
    unsafe {
        match timer {
            Timer16::Timer0 => {
                ct16b0.pr.modify(|_, w| w.pcval().bits(value));
            }
            Timer16::Timer1 => {
                ct16b1.pr.modify(|_, w| w.pcval().bits(value));
            }
        }
    }
}

/// Setup a clock to be used for PWM
///
/// # Arguments
/// `timer` - The timer to be used for PWM
/// `m0` - Value for match register 0
/// `m1` - Value for match register 1
/// `m2` - Value for match register 2
/// `m3` - Value for match register 3
///
/// # Example
/// ```
/// // Set PWM match registers
/// timers::set_pwm(&p.SYSCON, &p.CT16B0, &p.CT16B1, Timer16::Timer0, **r.PERIOD, **r.PERIOD - (**r.DC * (**r.PERIOD/100)), 1000, 1000);
/// unsafe {
///     p.CT16B0.pr.modify(|_, w| w.pcval().bits(9));
/// }
///
/// // Configure match properties
/// // Here, mr0 determines DC and mr1 when the output goes high
/// p.CT16B0.mcr.modify(|_, w| w.mr0r().bit(true));
/// p.CT16B0.mcr.modify(|_, w| w.mr1r().bit(false));
/// p.CT16B0.mcr.modify(|_, w| w.mr1s().bit(false));
///
/// // Enable the PWM
/// timers::set_enabled(&p.CT16B0, &p.CT16B1, Timer16::Timer0, false);
/// ```
pub fn set_pwm(
    syscon: &lpc1347::SYSCON,
    ct16b0: &lpc1347::CT16B0,
    ct16b1: &lpc1347::CT16B1,
    timer: Timer16,
    m0: u16,
    m1: u16,
    m2: u16,
    m3: u16,
) {
    match timer {
        Timer16::Timer0 => {
            set_enabled(&ct16b0, &ct16b1, timer, false);
            syscon.sysahbclkctrl.modify(|_, w| w.ct16b0().bit(true));

            ct16b0.emr.modify(|_, w| w.emc3().bits(0x1));
            ct16b0.emr.modify(|_, w| w.emc2().bits(0x1));
            ct16b0.emr.modify(|_, w| w.emc1().bits(0x1));
            ct16b0.emr.modify(|_, w| w.emc0().bits(0x1));

            ct16b0.emr.modify(|_, w| w.em3().bit(false));
            ct16b0.emr.modify(|_, w| w.em2().bit(false));
            ct16b0.emr.modify(|_, w| w.em1().bit(true));
            ct16b0.emr.modify(|_, w| w.em0().bit(true));

            ct16b0.pwmc.modify(|_, w| w.pwmen3().bit(false));
            ct16b0.pwmc.modify(|_, w| w.pwmen2().bit(false));
            ct16b0.pwmc.modify(|_, w| w.pwmen1().bit(true));
            ct16b0.pwmc.modify(|_, w| w.pwmen0().bit(true));

            set_match(&ct16b0, &ct16b1, Timer16::Timer0, MatchReg::Reg0, m0);
            set_match(&ct16b0, &ct16b1, Timer16::Timer0, MatchReg::Reg1, m1);
            set_match(&ct16b0, &ct16b1, Timer16::Timer0, MatchReg::Reg2, m2);
            set_match(&ct16b0, &ct16b1, Timer16::Timer0, MatchReg::Reg3, m3);

            // Reset on clock 0 -> period
            ct16b0.mcr.modify(|_, w| w.mr0r().bit(true));
        }

        Timer16::Timer1 => {
            set_enabled(&ct16b0, &ct16b1, timer, false);
            syscon.sysahbclkctrl.modify(|_, w| w.ct16b0().bit(true));

            ct16b1.emr.modify(|_, w| w.emc3().bits(0x1));
            ct16b1.emr.modify(|_, w| w.emc2().bits(0x1));
            ct16b1.emr.modify(|_, w| w.emc1().bits(0x1));
            ct16b1.emr.modify(|_, w| w.emc0().bits(0x1));

            ct16b1.emr.modify(|_, w| w.em3().bit(false));
            ct16b1.emr.modify(|_, w| w.em2().bit(false));
            ct16b1.emr.modify(|_, w| w.em1().bit(true));
            ct16b1.emr.modify(|_, w| w.em0().bit(true));

            ct16b1.pwmc.modify(|_, w| w.pwmen3().bit(false));
            ct16b1.pwmc.modify(|_, w| w.pwmen2().bit(false));
            ct16b1.pwmc.modify(|_, w| w.pwmen1().bit(true));
            ct16b1.pwmc.modify(|_, w| w.pwmen0().bit(true));

            set_match(&ct16b0, &ct16b1, Timer16::Timer1, MatchReg::Reg0, m0);
            set_match(&ct16b0, &ct16b1, Timer16::Timer1, MatchReg::Reg1, m1);
            set_match(&ct16b0, &ct16b1, Timer16::Timer1, MatchReg::Reg2, m2);
            set_match(&ct16b0, &ct16b1, Timer16::Timer1, MatchReg::Reg3, m3);

            // Reset on clock 0 -> period
            ct16b1.mcr.modify(|_, w| w.mr0r().bit(true));
        }
    }
}
