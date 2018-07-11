#![allow(dead_code)]
extern crate lpc1347;

/// Maps ADC channels to pins
#[derive(Copy, Clone)]
pub enum PinPos {
    /// Channel 0 => pio0_11
    Pin0 = 11,
    /// Channel 1 => pio0_12
    Pin1 = 12,
    /// Channel 2 => pio0_13
    Pin2 = 13,
    /// Channel 3 => pio0_15
    Pin3 = 14,
    /// Channel 4 => pio0_15
    Pin4 = 15,
    /// Channel 5 => pio0_16
    Pin5 = 16,
    /// Channel 6 => pio0_22
    Pin6 = 22,
    /// Channel 7 => pio0_23
    Pin7 = 23,
}

/// Specify event for capture triggers
#[derive(Copy, Clone)]
pub enum Capture {
    /// Capture on rising edge
    Rising,
    /// Capture on falling edge
    Falling,
}

/// Initialize the ADC
///
/// # Arguments
/// * `pinnum` - The AD-pin to use (0-7)
/// * `system_core_clock` - Clock frequency for the main clock
/// * `low_power` - Use less power
/// * `mode10bit` - Limit sampling to 10bit to allow 31MHz sampling
/// * `edge` - Set interrupts on rising or falling edges
///
/// # Example
/// ```
/// // Configure ADC to read from pin 5
/// adc::init(&p.SYSCON, &p.ADC, 5u8, 24000000u32, false, false, Capture::Rising);
/// adc::set_adc_pin(&p.IOCON, adc::PinPos::Pin5);
/// ```
pub fn init(
    syscon: &lpc1347::SYSCON,
    adc: &lpc1347::ADC,
    pinnum: u8,
    system_core_clock: u32,
    low_power: bool,
    mode10bit: bool,
    edge: Capture,
) {
    // Disallow invalid pins
    if pinnum > 7 {
        panic!("invalid pin number initialized");
    }

    // Power up ADC module
    syscon.pdruncfg.modify(|_, w| w.adc_pd().bit(false));
    syscon.sysahbclkctrl.modify(|_, w| w.adc().bit(true));

    // Stop the ADC
    adc.cr.modify(|_, w| w.start().no_start_this_value());

    unsafe {
        // Select channels
        adc.cr
            .modify(|r, w| w.sel().bits(r.sel().bits() | 1 << pinnum));

        // Read the system clock divider
        // to be able to calculate the clockdiv for ADC
        // The quicker the better, requires about 31 cycles to complete
        // Maximum frequency for 12 bit, 15.5MHz, 10 bit 31MHz

        let clkdiv = syscon.sysahbclkdiv.read().div().bits() as u32;

        // Set ADC clock divider
        // ADC is driven on the APB bus
        adc.cr.modify(|_, w| {
            w.clkdiv()
                .bits(((system_core_clock / (system_core_clock / clkdiv)) - 1) as u8)
        });
    }

    // Set software control
    adc.cr.modify(|_, w| w.burst().software_controlled_());

    // Set lowpower mode, if requested
    adc.cr.modify(|_, w| w.lpwrmode().bit(low_power));

    // Set 10-bit conversion mode
    adc.cr.modify(|_, w| w.mode10bit().bit(mode10bit));

    // Set rising/falling edge
    match edge {
        Capture::Rising => {
            adc.cr.modify(|_, w| w.edge().rising());
        }
        Capture::Falling => {
            adc.cr.modify(|_, w| w.edge().falling());
        }
    }
}

/// Configure the board to read from pin ADn
///
/// # Arguments
/// * `pin` - The AD-pin to use (0-7)
pub fn set_adc_pin(iocon: &lpc1347::IOCON, pin: PinPos) {
    match pin {
        PinPos::Pin0 => {
            iocon
                .tdi_pio0_11
                .modify(|_, w| w.admode().analog_input_mode_());
            iocon.tdi_pio0_11.modify(|_, w| w.func().ad0_());
        }
        PinPos::Pin1 => {
            iocon
                .tms_pio0_12
                .modify(|_, w| w.admode().analog_input_mode_());
            iocon.tms_pio0_12.modify(|_, w| w.func().ad1_());
        }
        PinPos::Pin2 => {
            iocon
                .tdo_pio0_13
                .modify(|_, w| w.admode().analog_input_mode_());
            iocon.tdo_pio0_13.modify(|_, w| w.func().ad2_());
        }
        PinPos::Pin3 => {
            iocon
                .trst_pio0_14
                .modify(|_, w| w.admode().analog_input_mode_());
            iocon.trst_pio0_14.modify(|_, w| w.func().ad3_());
        }
        PinPos::Pin4 => {
            iocon
                .swdio_pio0_15
                .modify(|_, w| w.admode().analog_input_mode_());
            iocon.swdio_pio0_15.modify(|_, w| w.func().ad4_());
        }
        PinPos::Pin5 => {
            iocon.pio0_16.modify(|_, w| w.admode().analog_input_mode_());
            iocon.pio0_16.modify(|_, w| w.func().ad5_());
        }
        PinPos::Pin6 => {
            iocon.pio0_22.modify(|_, w| w.admode().analog_input_mode_());
            iocon.pio0_22.modify(|_, w| w.func().ad6_());
        }
        PinPos::Pin7 => {
            iocon.pio0_23.modify(|_, w| w.admode().analog_input_mode_());
            iocon.pio0_23.modify(|_, w| w.func().ad7_());
        }
    }
}

/// Read from the ADC at some channel
///
/// # Arguments
/// * `channel` - A/D channel to sample
///
/// # Example
/// ```
/// {
///     let mut stdout = hio::hstdout().unwrap();
///     let _ = writeln!(stdout, "ADC ({})", adc::read(r.ADC, 5));
/// }
/// ```
pub fn read(adc: &lpc1347::ADC, channel: u8) -> u16 {
    if channel > 7 {
        panic!("invalid channel selected")
    }

    // Start read on channel
    unsafe {
        adc.cr.modify(|_, w| w.sel().bits(1 << channel));
    }
    adc.cr.modify(|_, w| w.start().start_conversion_now());

    // Read data
    let mut register_value;
    loop {
        match channel {
            0 => register_value = adc.dr[0].read(),
            1 => register_value = adc.dr[1].read(),
            2 => register_value = adc.dr[2].read(),
            3 => register_value = adc.dr[3].read(),
            4 => register_value = adc.dr[4].read(),
            5 => register_value = adc.dr[5].read(),
            6 => register_value = adc.dr[6].read(),
            7 => register_value = adc.dr[7].read(),
            _ => panic!("invalid channel selected!"),
        }

        if register_value.done().bit_is_set() {
            break;
        }
    }

    // Stop conversion
    adc.cr.modify(|_, w| w.start().no_start_this_value());

    // Return value, depends on 10-bit mode
    if adc.cr.read().mode10bit().is_enable_the_10_bit_co() {
        // In 10 bit mode, the two LSB bits are forced to 0, thus shift 2 steps
        return (register_value.v_vref().bits() >> 2) & 0x3FF;
    } else {
        return register_value.v_vref().bits() & 0xFFF;
    }
}
