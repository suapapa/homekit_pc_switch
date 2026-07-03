# hap-pc-btn

Rust firmware for **ESP32-C3** that exposes a **HomeKit Switch** accessory to pulse a PC power-button relay (500 ms). Behavior matches the original Arduino/HomeSpan reference in `_ref/arduino/`.

- **HomeKit**: [Espressif esp-homekit-sdk](https://github.com/espressif/esp-homekit-sdk) (C)
- **Rust**: `esp-idf-svc` + `esp-idf-hal` (std, FreeRTOS)
- **Pairing**: setup code `111-22-334`, setup ID `ES32`

## Features

- HomeKit Switch toggles a 500 ms relay pulse (then returns to Off)
- Physical button input with debounce triggers the same pulse
- Onboard status LED blinks while running
- Debug GPIO toggles with the relay

## Hardware

Target board: **ESP32-C3 Super Mini**

| Signal       | GPIO | Notes                                      |
|--------------|------|--------------------------------------------|
| Status LED   | 8    | Onboard, active-low; strapping pin         |
| Power button | 4    | External input, pull-up, active low        |
| Relay out    | 10   | 500 ms pulse to PC front-panel header      |
| Debug out    | 3    | Toggles with relay                         |

**Avoid:** GPIO9 (BOOT), GPIO2 (strapping). GPIO18 is not broken out on this board.

Pin constants live in `src/pins.rs`.

## Prerequisites

| Requirement   | Value                          |
|---------------|--------------------------------|
| Rust          | `esp-1.90` (`rust-toolchain.toml`) |
| Target        | `riscv32imc-esp-espidf`        |
| MCU           | `esp32c3`                      |
| ESP-IDF       | **v5.5.3** (managed by embuild) |
| Flash tool    | `espflash`                     |

Install `espflash` if needed:

```bash
cargo install espflash
```

### Third-party SDK (git submodule)

esp-homekit-sdk is vendored as a **git submodule** at `third-party/esp-homekit-sdk`.

**Fresh clone** — initialize submodules after cloning this repo:

```bash
git clone <this-repo-url> hap-pc-btn
cd hap-pc-btn
git submodule update --init --recursive
```

**Existing checkout** — if `third-party/esp-homekit-sdk` is empty or missing:

```bash
git submodule update --init --recursive
```

**Troubleshooting** — if `git submodule add` fails with *already exists in the index* (stale gitlink without `.gitmodules`):

```bash
git rm --cached third-party/esp-homekit-sdk
git submodule add https://github.com/espressif/esp-homekit-sdk.git third-party/esp-homekit-sdk
git submodule update --init --recursive
```

Do **not** manually `git clone` into `third-party/esp-homekit-sdk`; that conflicts with the submodule entry. After updating the submodule, run `cargo build` as usual.

## Wi-Fi credentials

Edit `sdkconfig.defaults`, or set environment variables at build time (written to `sdkconfig.defaults.env` by `build.rs`):

```bash
export WIFI_SSID="your-ssid"
export WIFI_PASS="your-password"
```

`run.sh` is a local convenience wrapper that sets these and runs `cargo run --release`.

## Build and flash

**Important:** If your shell exports `IDF_PATH` pointing to ESP-IDF **6.x**, the build fails. esp-homekit-sdk requires IDF 5.x.

```bash
unset IDF_PATH
cargo build              # dev
cargo build --release
cargo run --release      # flash + serial monitor (esp32c3)
```

The first build downloads ESP-IDF and tools (~5+ minutes). Later builds are much faster.

## HomeKit pairing

Default pairing code: **111-22-334**

Generate a QR code URI (setup ID must match `CONFIG_EXAMPLE_SETUP_ID` in `sdkconfig.defaults`, default `ES32`):

```bash
./script/generate_qr.py
./script/generate_qr.py -s          # URI only
./script/show_homekit_pc_switch_qr.sh
```

Wrong setup ID produces a QR that the iPhone accepts, but the device will not respond — mDNS filters on the setup ID suffix.

## How it works

```
HomeKit Switch On  →  C switch_write()  →  Rust relay_pulse()  →  500 ms HIGH  →  Switch Off
Physical button    →  Rust GPIO loop    →  same pulse + pc_homekit_physical_button()
```

WiFi and HAP run in a FreeRTOS task (`pc_homekit_start()`). Rust `main()` owns GPIO polling (button, LED, relay).

```
src/bin/main.rs              GPIO loop (button, LED, relay)
src/ffi.rs                   Rust ↔ C trampoline
components/pc_homekit/       HomeKit Switch + WiFi + HAP init
third-party/esp-homekit-sdk/ Espressif SDK (local clone)
```

## Configuration

| Change                    | Where                                              |
|---------------------------|----------------------------------------------------|
| Relay pulse duration      | `src/pins.rs` → `RELAY_PULSE_MS`                 |
| GPIO pins                 | `src/pins.rs`, `src/bin/main.rs`                   |
| HomeKit name / setup code | `components/pc_homekit/pc_homekit.c`, `sdkconfig.defaults` |
| WiFi mode                 | `sdkconfig.defaults`, esp-homekit-sdk `app_wifi`   |

## Project layout

| Path                  | Purpose                                      |
|-----------------------|----------------------------------------------|
| `Cargo.toml`          | Dependencies + esp-homekit-sdk component paths |
| `.cargo/config.toml`  | Target, runner, ESP-IDF version, MCU         |
| `sdkconfig.defaults`  | WiFi, HomeKit, stack/mDNS defaults           |
| `components/pc_homekit/` | Custom IDF component (HAP Switch)         |
| `script/`             | HomeKit QR code generators                   |
| `AGENTS.md`           | Contributor / AI agent handoff notes         |

## References

- [esp-homekit-sdk](https://github.com/espressif/esp-homekit-sdk) — smart_outlet example
- [esp-idf Rust book](https://docs.esp-rs.org/esp-idf-template/)
- Arduino reference: `_ref/arduino/homekit_pc_switch.ino`

## License

See repository license file if present. esp-homekit-sdk has its own license in `third-party/esp-homekit-sdk/`.
