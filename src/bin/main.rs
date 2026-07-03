use std::sync::Mutex;
use std::time::{Duration, Instant};

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;

use hap_pc_btn::ffi::{notify_physical_button, set_relay_pulse_handler, start_homekit};
use hap_pc_btn::pins;

struct GpioState {
    relay: Mutex<PinDriver<'static, esp_idf_svc::hal::gpio::Output>>,
    debug: Mutex<PinDriver<'static, esp_idf_svc::hal::gpio::Output>>,
    status_led: Mutex<PinDriver<'static, esp_idf_svc::hal::gpio::Output>>,
}

static GPIO: Mutex<Option<&'static GpioState>> = Mutex::new(None);

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("hap-pc-btn starting (esp-homekit-sdk)");

    let peripherals = Peripherals::take()?;
    let pins_hw = peripherals.pins;

    let status_led = PinDriver::output(pins_hw.gpio8)?;
    let button = PinDriver::input(pins_hw.gpio4, Pull::Up)?;
    let relay = PinDriver::output(pins_hw.gpio10)?;
    let debug = PinDriver::output(pins_hw.gpio3)?;

    let gpio: &'static GpioState = Box::leak(Box::new(GpioState {
        relay: Mutex::new(relay),
        debug: Mutex::new(debug),
        status_led: Mutex::new(status_led),
    }));

    {
        let mut relay = gpio.relay.lock().unwrap();
        let mut debug = gpio.debug.lock().unwrap();
        let mut led = gpio.status_led.lock().unwrap();
        relay.set_low()?;
        debug.set_low()?;
        led.set_high()?; // active-low LED off
    }

    *GPIO.lock().unwrap() = Some(gpio);

    set_relay_pulse_handler(relay_pulse);
    start_homekit();

    log::info!(
        "GPIO status={} btn={} relay={} debug={}",
        pins::STATUS_LED,
        pins::PWRBTN_IN,
        pins::RELAY_OUT,
        pins::DEBUG_OUT
    );
    log::info!("HomeKit pairing code: 111-22-334");

    let mut last_raw_pressed = button.is_low();
    let mut stable_pressed = false;
    let mut debouncing = false;
    let mut debounce_deadline = Instant::now();
    let mut press_start: Option<Instant> = None;
    let mut long_press_triggered = false;
    let mut led_on = false;
    let mut last_blink = Instant::now();

    loop {
        FreeRtos::delay_ms(10);

        if last_blink.elapsed() >= Duration::from_millis(500) {
            last_blink = Instant::now();
            led_on = !led_on;
            if let Ok(mut led) = gpio.status_led.lock() {
                if led_on {
                    let _ = led.set_low();
                } else {
                    let _ = led.set_high();
                }
            }
        }

        let raw_pressed = button.is_low();
        if raw_pressed != last_raw_pressed {
            last_raw_pressed = raw_pressed;
            debouncing = true;
            debounce_deadline =
                Instant::now() + Duration::from_millis(pins::BUTTON_DEBOUNCE_MS as u64);
        }

        if debouncing && Instant::now() >= debounce_deadline {
            debouncing = false;
            if raw_pressed != stable_pressed {
                stable_pressed = raw_pressed;
                if stable_pressed {
                    press_start = Some(Instant::now());
                    long_press_triggered = false;
                } else if let Some(start) = press_start.take() {
                    let held = start.elapsed();
                    if !long_press_triggered
                        && held < Duration::from_millis(pins::BUTTON_SHORT_PRESS_MAX_MS as u64)
                    {
                        log::info!("Short press ({} ms) -> power on", held.as_millis());
                        pulse_relay(gpio);
                        notify_physical_button();
                    } else if !long_press_triggered {
                        log::info!(
                            "Release after {} ms (not short/long) -> ignored",
                            held.as_millis()
                        );
                    }
                    long_press_triggered = false;
                }
            }
        }

        if stable_pressed && !long_press_triggered {
            if let Some(start) = press_start {
                if start.elapsed() >= Duration::from_millis(pins::BUTTON_LONG_PRESS_MS as u64) {
                    long_press_triggered = true;
                    log::info!(
                        "Long press (>= {} ms) -> force shutdown ({} ms hold)",
                        pins::BUTTON_LONG_PRESS_MS,
                        pins::FORCE_SHUTDOWN_MS
                    );
                    hold_relay(gpio, pins::FORCE_SHUTDOWN_MS);
                }
            }
        }
    }
}

fn set_relay_outputs(
    gpio: &GpioState,
    on: bool,
) -> Result<(), esp_idf_svc::hal::gpio::GpioError> {
    let mut relay = gpio.relay.lock().unwrap();
    let mut debug = gpio.debug.lock().unwrap();
    if on {
        relay.set_high()?;
        debug.set_high()?;
    } else {
        relay.set_low()?;
        debug.set_low()?;
    }
    Ok(())
}

fn pulse_relay(gpio: &GpioState) {
    let _ = set_relay_outputs(gpio, true);
    FreeRtos::delay_ms(pins::RELAY_PULSE_MS);
    let _ = set_relay_outputs(gpio, false);
}

fn hold_relay(gpio: &GpioState, duration_ms: u32) {
    let _ = set_relay_outputs(gpio, true);
    FreeRtos::delay_ms(duration_ms);
    let _ = set_relay_outputs(gpio, false);
}

unsafe extern "C" fn relay_pulse() {
    if let Some(gpio) = *GPIO.lock().unwrap() {
        pulse_relay(gpio);
    }
}
