#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/// Called by esp-homekit-sdk when Switch `On` is set or a local button press is synced.
typedef void (*pc_relay_pulse_fn)(void);

/// Register the relay pulse callback (provided by Rust).
void pc_homekit_set_relay_pulse(pc_relay_pulse_fn fn);

/// Start HomeKit (FreeRTOS task): Wi-Fi, pairing, Switch accessory.
void pc_homekit_start(void);

/// Notify HomeKit of a physical button press (mirrors Arduino single-click).
void pc_homekit_physical_button(void);

#ifdef __cplusplus
}
#endif
