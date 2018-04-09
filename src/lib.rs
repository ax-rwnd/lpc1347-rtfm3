//! # RTFMv3 For the NXP LPC1347
//!
//! This package provides driver support and examples for the LPC1347, it
//! builds on the works of George Aparicio (RTFM, svd2rust, arm cortex crates),
//! and Per Lindgren (RTFM).
//!
//! A special thanks goes out to Kevin Townsend (microbuilder) for providing an
//! excellent set of drivers written in C. These have proven a very useful
//! reference during development and debug of this crate.
//!

#![deny(missing_docs)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

pub extern crate lpc1347;

/// Provides support for using the GPIO
pub mod gpio;

/// 16-bit timers
pub mod timers16;

/// 32-bit timers
pub mod timers32;

/// Analog-digital converter
pub mod adc;

/// Power configuration
pub mod power;

/// Clock configuration
pub mod clock;

/// Repetetive-intterupt timer
pub mod ritimer;

/// In-application programmoing NOTE: untested
pub mod iap;

/// USART driver
pub mod usart;
