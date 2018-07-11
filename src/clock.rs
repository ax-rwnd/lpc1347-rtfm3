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
pub fn pll_init(syscon: &lpc1347::SYSCON, mval: u8, pval: u8) {
    if pval > 0x3 {
        panic!("the PLL does not admit p-values higher than 0x3");
    }

    // Make sure the system oscillator is powered on, otherwise no system signal
    syscon.pdawakecfg.modify(|_, w| w.sysosc_pd().powered());
    syscon.pdruncfg.modify(|_, w| w.sysosc_pd().powered());

    // Configure the PLL
    syscon.pdruncfg.modify(|_, w| w.syspll_pd().powered_down());
    syscon
        .syspllclksel
        .modify(|_, w| w.sel().crystal_oscillator_());
    unsafe {
        syscon.syspllctrl.modify(|_, w| w.msel().bits(mval));
        syscon.syspllctrl.modify(|_, w| w.psel().bits(pval));
    }
    syscon.pdruncfg.modify(|_, w| w.syspll_pd().powered());

    // Before setting the clocks, wait for the PLL to lock
    while !syscon.syspllstat.read().lock().bit() {}
    syscon.mainclksel.modify(|_, w| w.sel().pll_output());
    syscon.clkoutsel.modify(|_, w| w.sel().main_clock());
}

/// Set the System clock divider
///
/// # Arguments
/// * `divider` -  divider value for Fclkana expressed as a value between 0 and 255.
///
/// A divider value of `0` disables the system clock completely
///
/// Default value: 1
///
///
/// # Example
/// ```
/// // Half normal clock speed, typically 12 MHz
/// clock::sysclock_set_divider(
///     &p.device.SYSCON,
///     2,
/// );
/// ```
/// ```
/// // Disable system clock
/// clock::sysclock_set_divider(
///     &p.device.SYSCON,
///     0,
/// );
/// ```
///
pub fn sysclock_set_divider(syscon: &lpc1347::SYSCON, divider: u8) {
    unsafe {
        syscon.sysahbclkdiv.modify(|_, w| w.div().bits(divider));
    }
}

/// Initializes the watchdog
///
/// # Arguments
/// * `wd_int_enable` - Enable the watchdog timer interrupts * `wwdt_clk_src` - Select the clock source, OSC or watchdog * `frequency` - Select the Watchdog Fclkana frequency
/// * `wwdt_clk_src` - The source for the WWDT clock
/// * `frequency` - Select the WWDT frequency
/// * `divider` -  Divider value for Fclkana expressed as the actual divider, (2, 4, 6, 8, ... 62, 64) all even numbers up to 64.
///
/// Divider is calculated as follows:
/// DIVSEL is the actual register value
///
/// ```
/// wdt_osc_clk = Fclkana/ (2 * (1 + DIVSEL))
/// 00000: 2 * (1 + DIVSEL) = 2
/// 00001: 2 * (1 + DIVSEL) = 4
/// to
/// 11111: 2 * (1 + DIVSEL) = 64
/// ```
///
/// # Example
/// ```
/// clock::wwdt_init(
///     &p.device.SYSCON,
///     &p.device.WWDT,
///     true,
///     lpc1347::wwdt::clksel::CLKSELW::WATCHDOG_OSCILLATOR_,
///     lpc1347::syscon::wdtoscctrl::FREQSELW::_0_6_MHZ,
///     64
/// );
/// ```
///
pub fn wwdt_init(
    nvic: &mut lpc1347::NVIC,
    syscon: &lpc1347::SYSCON,
    wwdt: &lpc1347::WWDT,
    wd_int_enable: bool,
    wwdt_clk_src: lpc1347::wwdt::clksel::CLKSELW,
    frequency: lpc1347::syscon::wdtoscctrl::FREQSELW,
    divider: u8,
) {
    if divider < 2 {
        panic!("Too small divider! Minimum = 2");
    } else if divider > 64 {
        panic!("Too large divider! Maximum = 64");
    }

    syscon.sysahbclkctrl.modify(|_, w| w.wwdt().enable());
    syscon.pdruncfg.modify(|_, w| w.wdtosc_pd().powered());
    syscon.pdsleepcfg.modify(|_, w| w.wdtosc_pd().powered());
    syscon.pdawakecfg.modify(|_, w| w.wdtosc_pd().powered());

    // Set the clocksource
    wwdt.clksel.modify(|_, w| w.clksel().variant(wwdt_clk_src));
    syscon
        .wdtoscctrl
        .modify(|_, w| w.freqsel().variant(frequency));
    unsafe {
        syscon
            .wdtoscctrl
            .modify(|_, w| w.divsel().bits(divider / 2 - 1));
    }
    if wd_int_enable {
        // Enable WWDT Interrupt
        nvic.enable(lpc1347::Interrupt::WWDT);

        // Configure WWDT to act as wakeup interrupt
        syscon.starterp1.write(|w| w.wwdtint().enabled());
    }
}

/// Configure and setup the watchdog
///
/// # Arguments
/// * `wd_reset` - Enable if watchdog should reset
/// * `wd_lock` - Prevent disruption of the WDCLKSRC
/// * `wd_enable` - Enable the watchdog timer
/// * `wd_timerconstant` - Specify the time-out value, decrementing counter
/// * `wd_warnint` - Specify the timer warning interrupt value, at which value should interrupt be
/// triggered
///
/// # Example
/// ```
/// clock::wwdt_configure(
///     &p.device.WWDT,
///     true,
///     true,
///     true,
///     0xFFFF,
///     0xF
/// );
/// ```
pub fn wwdt_configure(
    wwdt: &lpc1347::WWDT,
    wd_reset: bool,
    wd_lock: bool,
    wd_enable: bool,
    wd_timerconstant: u32,
    wd_warnint: u16,
) {
    // Enable watchdog reset
    wwdt.mod_.modify(|_, w| w.wdreset().bit(wd_reset));

    // Prevent disabling/powerdown of watchdog clock source
    wwdt.mod_.modify(|_, w| w.lock().bit(wd_lock));

    // Enable the watchdog
    wwdt.mod_.modify(|_, w| w.wden().bit(wd_enable));

    unsafe {
        // Configure the Timer Constant, the time-out value
        // 24-bits max
        if wd_timerconstant > 0xFF_FF_FF {
            panic!("Invalid timer constant!");
        }
        wwdt.tc.modify(|_, w| w.count().bits(wd_timerconstant));

        // Set the warning
        if wd_warnint > 0x3_FF {
            panic!("Invalid warnint constant!");
        }
        wwdt.warnint.modify(|_, w| w.warnint().bits(wd_warnint));

        // Required for starting the watchdog, see 15.8.3 in UM10524
        wwdt.feed.write(|w| w.feed().bits(0xAA));
        wwdt.feed.write(|w| w.feed().bits(0x55));
    }
}

/// Feed the Watchdog
///
/// This resets the Watchdog counter to the value contained in WDTC
///
/// Usage (depending on mode): Use this to periodically feed the Watchdog
/// to prevent a system reset
///
/// # Example
/// ```
/// clock::wwdt_feed(
///     &p.device.WWDT
/// );
/// ```
pub fn wwdt_feed(wwdt: &lpc1347::WWDT) {
    unsafe {
        // Resets the watchdog counter back to WDTC
        wwdt.feed.write(|w| w.feed().bits(0xAA));
        wwdt.feed.write(|w| w.feed().bits(0x55));
    }
}

/// Clear WDINT interrupt register
///
/// This writes a 1 to WDINT, clearing the interrupt
///
/// Usage (depending on mode): Use this to prime the WDINT
///
/// Datasheet incorrectly states that a 0 should be written to clear it
///
/// # Example
/// ```
/// clock::wwdt_intclear(
///     &p.device.WWDT
/// );
/// ```
pub fn wwdt_intclear(wwdt: &lpc1347::WWDT) {
    wwdt.mod_.write(|w| w.wdint().set_bit());
}

/// Set WWDT as the system clock source
///
/// Change the main system clock to be the wwdt oscillator
///
/// Attention! Need to start WWDT first
///
///
/// # Example
/// ```
/// clock::wwdt_as_mainclk(
///     &p.device.SYSCON
/// );
/// ```
pub fn wwdt_as_mainclk(syscon: &lpc1347::SYSCON) {
    // WWDT on always, extra precaution
    syscon.pdawakecfg.modify(|_, w| w.wdtosc_pd().powered());
    syscon.pdruncfg.modify(|_, w| w.wdtosc_pd().powered());
    syscon.pdsleepcfg.modify(|_, w| w.wdtosc_pd().powered());

    // Select as main clocksource
    syscon
        .mainclksel
        .modify(|_, w| w.sel().watchdog_oscillator());

    // Disable IRC clock on wakeup
    syscon
        .pdawakecfg
        .modify(|_, w| w.ircout_pd().powered_down());
    syscon.pdawakecfg.modify(|_, w| w.irc_pd().powered_down());

    // Power down the IRC oscillator output
    syscon.pdruncfg.modify(|_, w| w.ircout_pd().powered_down());

    // Power down the IRC oscillator
    syscon.pdruncfg.modify(|_, w| w.irc_pd().powered_down());
}
