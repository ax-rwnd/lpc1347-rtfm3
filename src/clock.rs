#![allow(dead_code)]
extern crate lpc1347;

/// Configure the phase-locked-loop
/// Some values of m and p may make the MCU crash, see page 15 and 44 of UM10524
/// for details.
///
/// # Arguments
/// * `mval` - Feedback divider value
/// * `pval` - Post-divider ratio
///
/// # Example
/// ```
/// extern crate lpc1347;
/// mod clock;
/// use clock::pll_init;
///
/// fn init(p: init::Peripherals, r: init::Resources) {
///     // See admissible values in p. 44 of UM10524
///     pll_init(&p, 0b11, 0b01);
/// }
///
/// ```
pub unsafe fn pll_init(syscon: &lpc1347::SYSCON, mval: u8, pval: u8) {
    if pval > 0x3 {
        panic!("the PLL does not admit p-values higher than 0x3");
    }

    // Make sure the system oscillator is powered on, otherwise no system signal
    syscon.pdawakecfg.modify(|_, w| w.sysosc_pd().bit(false));
    syscon.pdruncfg.modify(|_, w| w.sysosc_pd().bit(false));

    // Configure the PLL
    syscon.pdruncfg.modify(|_, w| w.syspll_pd().bit(true));
    syscon.syspllclksel.modify(|_, w| w.sel().bits(0x1));
    syscon.syspllctrl.modify(|_, w| w.msel().bits(mval));
    syscon.syspllctrl.modify(|_, w| w.psel().bits(pval));
    syscon.pdruncfg.modify(|_, w| w.syspll_pd().bit(false));

    // Before setting the clocks, wait for the PLL to lock
    while !syscon.syspllstat.read().lock().bit() {}
    syscon.mainclksel.modify(|_, w| w.sel().bits(0x3));
    syscon.clkoutsel.modify(|_, w| w.sel().bits(0x3));
}

/// Initializes the watchdog
/// NOTE: this is currently buggy
pub fn wwdt_init(syscon: &lpc1347::SYSCON, wwdt: &lpc1347::WWDT, frequency: u8) {
    if frequency > 0xF {
        panic!("invalid frequency set!");
    }

    syscon.sysahbclkctrl.modify(|_, w| w.wwdt().bit(true));
    syscon.pdruncfg.modify(|_, w| w.wdtosc_pd().bit(false));
    wwdt.clksel.modify(|_, w| w.clksel().bit(true));
    unsafe {
        syscon.wdtoscctrl.modify(|_, w| w.freqsel().bits(frequency));
    }
}
