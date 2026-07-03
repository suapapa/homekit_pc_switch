//! GPIO pin assignments for ESP32-S2 (matches Arduino HomeSpan reference).
//!
//! Strapping pins GPIO0, GPIO3, GPIO45, GPIO46 must not be pulled low at reset.

pub const STATUS_LED: i32 = 15;
pub const PWRBTN_IN: i32 = 18;
pub const RELAY_OUT: i32 = 12;
pub const DEBUG_OUT: i32 = 16;

pub const RELAY_PULSE_MS: u32 = 500;
pub const FORCE_SHUTDOWN_MS: u32 = 5000;
pub const BUTTON_SHORT_PRESS_MAX_MS: u32 = 1000;
pub const BUTTON_LONG_PRESS_MS: u32 = 3000;
pub const BUTTON_DEBOUNCE_MS: u32 = 50;
