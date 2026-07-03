//! HomeKit PC power button — ESP32-C3 + esp-homekit-sdk.

pub mod ffi;
pub mod pins;

pub use ffi::{notify_physical_button, set_relay_pulse_handler, start_homekit};
