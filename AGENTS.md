# AGENTS.md — hap-pc-btn

Handoff guide for AI agents and contributors continuing work on this repo.

## What this project is

Rust firmware for **ESP32-S2** that exposes a **HomeKit Switch** accessory to pulse a PC power button relay (500 ms), matching the original Arduino/HomeSpan sketch in `_ref/arduino/`.

- **HomeKit stack**: [Espressif esp-homekit-sdk](https://github.com/espressif/esp-homekit-sdk) (C), not the Rust `hap` crate
- **Rust stack**: `esp-idf-svc` + `esp-idf-hal` (std, FreeRTOS), **not** `esp-hal` no_std
- **Pairing**: setup code `111-22-334`, setup id `ES32` (same as Arduino reference)

## Architecture

```
src/bin/main.rs     GPIO loop (button debounce, LED blink, relay pulse)
src/ffi.rs          Rust ↔ C trampoline for relay callback
components/pc_homekit/pc_homekit.c   HomeKit Switch + WiFi + HAP init
third-party/esp-homekit-sdk/         Vendored Espressif SDK (git clone)
```

**Control flow**

1. HomeKit Switch `On=true` → C `switch_write()` → Rust `relay_pulse()` → 500 ms relay HIGH → Switch forced back to `Off`
2. Physical button (GPIO18, active low, debounced) → same relay pulse + `pc_homekit_physical_button()` to sync HomeKit state

WiFi and HAP run in a FreeRTOS task started by `pc_homekit_start()`; Rust `main()` owns GPIO polling.

## Hardware (ESP32-S2)

Pin map matches `_ref/arduino/homekit_pc_switch.ino`.

| Signal      | GPIO | Notes |
|-------------|------|-------|
| Status LED  | 15   | External LED, active high |
| Power button| 18   | External input, pull-up, active low |
| Relay out   | 12   | 500 ms pulse |
| Debug out   | 16   | Toggles with relay |

**Do not use:** GPIO0, GPIO3, GPIO45, GPIO46 (strapping pins).

Pin constants: `src/pins.rs`.

Reference sketch: `_ref/arduino/homekit_pc_switch.ino`, `_ref/arduino/pc_pwr_button.h`

## Build prerequisites

| Requirement | Value |
|-------------|-------|
| Rust toolchain | `esp-1.90` (`rust-toolchain.toml`) |
| Target | `xtensa-esp32s2-espidf` |
| MCU | `esp32s2` (`.cargo/config.toml` → `MCU`) |
| ESP-IDF | **v5.5.3** managed via `embuild` (`ESP_IDF_VERSION` in `.cargo/config.toml`) |
| Flash runner | `espflash` (via `.cargo/config.toml` runner) |

### Third-party SDK (required before first build)

```bash
git clone --recursive https://github.com/espressif/esp-homekit-sdk.git third-party/esp-homekit-sdk
```

The directory `third-party/esp-homekit-sdk/` is not committed; clone it locally.

### WiFi credentials

Edit `sdkconfig.defaults`:

```
CONFIG_APP_WIFI_SSID="your-ssid"
CONFIG_APP_WIFI_PASSWORD="your-password"
```

Uses hardcoded WiFi (`CONFIG_APP_WIFI_USE_HARDCODED=y`), not BLE/SoftAP provisioning.

## Build and flash

**Critical**: If the shell exports `IDF_PATH` pointing to ESP-IDF **6.x**, the build fails (`wifi_provisioning` missing). esp-homekit-sdk targets IDF 5.x only.

```bash
unset IDF_PATH          # use managed v5.5.3 from .cargo/config.toml
cargo build             # dev
cargo build --release
cargo run --release       # espflash flash --no-stub --chip esp32s2
```

First build downloads IDF/tools and compiles C components (~5+ min). Subsequent builds are faster.

Verified: `cargo build` and `cargo build --release` succeed with managed IDF v5.5.3.

## HomeKit QR code

Scripts in `script/` generate the pairing URI. **Setup ID must match firmware** (`CONFIG_EXAMPLE_SETUP_ID` in `sdkconfig.defaults`, default `ES32`).

```bash
./script/generate_qr.py          # reads sdkconfig.defaults
./script/generate_qr.py -s       # URI only → X-HM://0080QW42MES32
./script/show_homekit_pc_switch_qr.sh
```

Wrong Setup ID (e.g. `HSPN` instead of `ES32`) produces a valid-looking QR that the iPhone accepts, but the ESP32 will not respond — mDNS filters on Setup ID suffix.

## Key configuration files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Deps + `[package.metadata.esp-idf-sys.extra_components]` for homekit SDK paths |
| `.cargo/config.toml` | Target, runner, `ESP_IDF_VERSION`, `MCU` |
| `build.rs` | `embuild::espidf::sysenv::output()` only |
| `sdkconfig.defaults` | WiFi, HomeKit setup code, stack/mDNS defaults |
| `components/pc_homekit/` | Custom IDF component wrapping HAP Switch |
| `components/pc_homekit/Kconfig.projbuild` | `CONFIG_EXAMPLE_SETUP_CODE` etc. |

### Linking esp-homekit-sdk into esp-idf-sys

Configured in `Cargo.toml`:

```toml
[[package.metadata.esp-idf-sys.extra_components]]
component_dirs = [
    "third-party/esp-homekit-sdk/components",
    "third-party/esp-homekit-sdk/components/homekit",
    "third-party/esp-homekit-sdk/examples/common",
    "components",
]
bindings_header = "components/pc_homekit/include/pc_homekit.h"
```

Do **not** use `ESP_IDF_EXTRA_COMPONENT_DIRS` env var; use this metadata block.

### pc_homekit C component

- `REQUIRES`: `esp_hap_core`, `esp_hap_apple_profiles`, `app_wifi`, `app_hap_setup_payload`, `esp_event`, `nvs_flash`
- **Wrong**: `REQUIRES homekit` — no such component; use `esp_hap_core` instead
- Public C API: `components/pc_homekit/include/pc_homekit.h`

## Common pitfalls (already hit)

1. **IDF 6.x + esp-homekit-sdk** → `Failed to resolve component 'wifi_provisioning'`. Fix: IDF **5.5.x**, `unset IDF_PATH`.
2. **`homekit` component name** → use `esp_hap_core` / `esp_hap_apple_profiles`.
3. **Rust `hap` crate** → requires std/Tokio; does not work on embedded. Stay on esp-homekit-sdk C stack.
4. **`esp-idf-svc` embassy features** → link error `__embassy_time_queue_item_from_waker`. Current `Cargo.toml` uses only `critical-section`.
5. **GPIO API (esp-idf-hal 0.46)** → `PinDriver::input(pin, Pull::Up)`, not `.with_pull()`.
6. **Import path** → use `esp_idf_svc::hal::...`, not bare `esp_idf_hal` (hal is not a direct dependency).

## What is done vs not done

**Done**

- esp-idf-std project scaffold for ESP32-S2
- esp-homekit-sdk integration via C component + FFI
- Switch accessory with relay pulse semantics matching Arduino
- Physical button + status LED loop in Rust
- dev/release builds pass

**Not done / follow-ups**

- On-device flash + HomeKit pairing test (needs hardware + real WiFi SSID)
- `sdkconfig.defaults` still has placeholder WiFi
- No CI; no submodule/gitignore entry for `third-party/` (clone is manual)
- MFi: `hap_enable_mfi_auth(HAP_MFI_AUTH_HW)` in `pc_homekit.c` — public SDK uses dummy MFi; verify behavior on real device
- `_ref/` is reference only (gitignored in `.gitignore`)

## Modifying behavior

| Change | Where |
|--------|-------|
| Relay pulse duration | `src/pins.rs` → `RELAY_PULSE_MS` |
| GPIO pins | `src/pins.rs` + `src/bin/main.rs` peripheral assignment |
| HomeKit name/model/setup code | `components/pc_homekit/pc_homekit.c`, `sdkconfig.defaults`, `Kconfig.projbuild` |
| WiFi mode (provisioning vs hardcoded) | `sdkconfig.defaults`, esp-homekit-sdk `app_wifi` Kconfig |
| New HomeKit services/chars | Extend `pc_homekit.c` using esp-homekit-sdk examples (`examples/smart_outlet`, `examples/fan`) |

## Useful references

- esp-homekit-sdk examples: `third-party/esp-homekit-sdk/examples/smart_outlet/`
- Rust + homekit example (older stack): [schphil/rust-esp-homekit-sdk-smart-outlet](https://github.com/schphil/rust-esp-homekit-sdk-smart-outlet)
- esp-idf Rust book: https://docs.esp-rs.org/esp-idf-template/

## Agent workflow tips

- Read `_ref/arduino/homekit_pc_switch.ino` before changing accessory semantics.
- After editing C or `sdkconfig.defaults`, run a full `cargo build` (esp-idf-sys regenerates sdkconfig).
- Keep changes minimal: Rust for GPIO/timing, C for HomeKit/WiFi unless replacing `app_wifi` with esp-idf-svc WiFi (larger refactor).
- Do not commit secrets in `sdkconfig.defaults`.
- Do not switch to IDF 6.x without porting esp-homekit-sdk / replacing `app_wifi`.
