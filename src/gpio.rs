#![allow(dead_code)]

extern crate lpc1347;
use lpc1347::Interrupt::{PIN_INT0, PIN_INT1, PIN_INT2, PIN_INT3, PIN_INT4, PIN_INT5, PIN_INT6,
                         PIN_INT7};

/// Writes to a register using value and bitpos
macro_rules! write_reg {
    ($field: expr, $bitpos: expr, $value: expr) => {{
        unsafe {
            //$field.write(|w| w.bits($value << $bitpos));
            $field.modify(|_, w| w.bits($value << $bitpos));
        }
    }};
}

/// Or into register
macro_rules! or_reg {
    ($field: expr, $value: expr) => {{
        unsafe {
            $field.modify(|r, w| w.bits(r.bits() | $value));
        }
    }};

    ($field: expr, $bitpos: expr, $value: expr) => {{
        unsafe {
            $field.modify(|r, w| w.bits(r.bits() | ($value << $bitpos)));
        }
    }};
}

/// And into register
macro_rules! neg_and_reg {
    ($field: expr, $value: expr) => {{
        unsafe {
            $field.modify(|r, w| w.bits(r.bits() & (!$value)));
        }
    }};

    ($field: expr, $bitpos: expr, $value: expr) => {{
        unsafe {
            $field.modify(|r, w| w.bits(r.bits() & (!($value << $bitpos))));
        }
    }};
}

/// Check port 0/1
#[derive(Copy, Clone)]
pub enum Port {
    /// Port 0 on the board.
    Port0,
    /// Port 1 on the board.
    Port1,
}

/// User edge or level detection
#[derive(Copy, Clone)]
pub enum Sense {
    /// Specifies edge sensitivity
    Edge,
    /// Specifies level sensitivity
    Level,
}

/// Falling/rising edge detected
#[derive(Copy, Clone)]
pub enum Event {
    /// Falling edge
    Falling,
    /// Rising edge
    Rising,
    /// Level high
    High,
    /// Level low
    Low,
}

/// Initialize the GPIO ports
///
/// # Arguments
/// * `group0` - Enable or disable group 0 interrupts
/// * `group1` - Enable or disable group 1 interrupts
pub fn init(syscon: &lpc1347::SYSCON, group0: bool, group1: bool) {
    // Start clocks
    syscon.sysahbclkctrl.modify(|_, w| w.iocon().bit(true));
    syscon.sysahbclkctrl.modify(|_, w| w.gpio().bit(true));
    syscon.sysahbclkctrl.modify(|_, w| w.pint().bit(true));

    // Enable grouped interrupts
    syscon
        .sysahbclkctrl
        .modify(|_, w| w.group0int().bit(group0));
    syscon
        .sysahbclkctrl
        .modify(|_, w| w.group1int().bit(group1));
}

/// Set pin for an interrupt
///
/// # Arguments
/// * `channel` - The target channel for the interrupt
/// * `port` - Which port (0/1) of pins to use
/// * `bitpos` - Which pin to use
/// * `sense` - Sense on edge or level when generating interrupts
/// * `event` - Trigger on falling/rising or high/low
pub fn set_pin_interrupt(
    syscon: &lpc1347::SYSCON,
    nvic: &mut lpc1347::NVIC,
    gpio_pin_int: &lpc1347::GPIO_PIN_INT,
    channel: u8,
    port: Port,
    bitpos: u32,
    sense: Sense,
    event: Event,
) {
    // Calculate offset based on port
    let offset: u32;
    match port {
        Port::Port0 => {
            offset = 0u32;
        }
        Port::Port1 => {
            offset = 24u32;
        }
    }

    match channel {
        0 => {
            or_reg!(syscon.pintsel[0], bitpos + offset);
            nvic.enable(PIN_INT0);
        }
        1 => {
            or_reg!(syscon.pintsel[1], bitpos + offset);
            nvic.enable(PIN_INT1);
        }
        2 => {
            or_reg!(syscon.pintsel[2], bitpos + offset);
            nvic.enable(PIN_INT2);
        }
        3 => {
            or_reg!(syscon.pintsel[3], bitpos + offset);
            nvic.enable(PIN_INT3);
        }
        4 => {
            or_reg!(syscon.pintsel[4], bitpos + offset);
            nvic.enable(PIN_INT4);
        }
        5 => {
            or_reg!(syscon.pintsel[5], bitpos + offset);
            nvic.enable(PIN_INT5);
        }
        6 => {
            or_reg!(syscon.pintsel[6], bitpos + offset);
            nvic.enable(PIN_INT6);
        }
        7 => {
            or_reg!(syscon.pintsel[7], bitpos + offset);
            nvic.enable(PIN_INT7);
        }

        _ => {
            panic!("Invalid channel passed!");
        }
    }

    // Either use edge detection or level detection
    match sense {
        Sense::Edge => {
            neg_and_reg!(&gpio_pin_int.isel, channel, 1);

            match event {
                Event::Falling => {
                    or_reg!(&gpio_pin_int.ienf, channel, 1);
                }
                Event::Rising => {
                    or_reg!(&gpio_pin_int.ienr, channel, 1);
                }

                _ => {
                    panic!("invalid combination for sense and event, use Event::Falling or Event::Rising here")
                }
            }
        }

        Sense::Level => {
            or_reg!(&gpio_pin_int.isel, channel, 1);
            or_reg!(&gpio_pin_int.ienr, channel, 1);

            match event {
                Event::Low => {
                    neg_and_reg!(&gpio_pin_int.ienf, channel, 1);
                }
                Event::High => {
                    or_reg!(&gpio_pin_int.ienf, channel, 1);
                }
                _ => panic!(
                    "invalid combination for sense and event, use Event::Low or Event::High here"
                ),
            }
        }
    }
}

/// Determine if an interrupt is enabled or not
// pub fn get_status(p: &Peripherals, channel: u8) -> bool {
//     return (p.GPIO_PIN_INT.ist.read().bits() & (1 << channel)) == 1u32;
// }

/// Clear the pin interrupt status
pub fn clear_status(gpio_pin_int: &lpc1347::GPIO_PIN_INT, channel: u8) {
    if gpio_pin_int.isel.read().bits() & (1 << channel) == 0 {
        write_reg!(&gpio_pin_int.ist, channel, 1u32);
    }
}

/// Not implemented
pub fn set_grouped_interrupt() {
    panic!("not implemented");
}

/// Get current state of the pin
pub fn get_pin_value(gpio_port: &lpc1347::GPIO_PORT, port: Port, bitpos: u32) -> bool {
    match port {
        Port::Port0 => {
            return gpio_port.set[0].read().bits() & (1 << bitpos) > 0;
        }
        Port::Port1 => {
            return gpio_port.set[1].read().bits() & (1 << bitpos) > 0;
        }
    }
}

/// Set value for pin
pub fn set_pin_value(gpio_port: &lpc1347::GPIO_PORT, port: Port, bitpos: u32, value: bool) {
    match port {
        Port::Port0 => {
            write_reg!(gpio_port.set[0], bitpos, if value { 1 } else { 0 });
        }
        Port::Port1 => {
            write_reg!(gpio_port.set[1], bitpos, if value { 1 } else { 0 });
        }
    }
}

/// Set pin direction
///
/// # Arguments
/// * `port` - Which port (0/1) of pins to use
/// * `bitpos` - The pin number to use
/// * `output`- Whether it should be an output pin (true) or an input pin (false)
pub fn set_dir(gpio_port: &lpc1347::GPIO_PORT, port: Port, bitpos: u32, output: bool) {
    match port {
        Port::Port0 => {
            if output {
                or_reg!(gpio_port.dir[0], bitpos, 1);
            } else {
                neg_and_reg!(gpio_port.dir[0], bitpos, 0);
            }
        }
        Port::Port1 => {
            if output {
                or_reg!(gpio_port.dir[1], bitpos, 1);
            } else {
                neg_and_reg!(gpio_port.dir[1], bitpos, 0);
            }
        }
    }
}
