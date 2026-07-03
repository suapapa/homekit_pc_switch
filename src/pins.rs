//! GPIO pin assignments for ESP32-C3 Super Mini.
//!
//! Exposed header pins: GPIO0–10, GPIO20, GPIO21 (GPIO18 is NOT broken out).
//! Strapping pins GPIO2, GPIO8, GPIO9 must not be pulled low at reset.
//! - GPIO8: onboard blue LED (active-low); safe to drive after boot
//! - GPIO9: onboard BOOT button — do not wire external circuits here

pub const STATUS_LED: i32 = 8;
pub const PWRBTN_IN: i32 = 4;
pub const RELAY_OUT: i32 = 10;
pub const DEBUG_OUT: i32 = 3;

pub const RELAY_PULSE_MS: u32 = 500;
pub const FORCE_SHUTDOWN_MS: u32 = 5000;
pub const BUTTON_SHORT_PRESS_MAX_MS: u32 = 1000;
pub const BUTTON_LONG_PRESS_MS: u32 = 3000;
pub const BUTTON_DEBOUNCE_MS: u32 = 50;
