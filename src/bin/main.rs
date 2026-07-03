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

    let mut last_button_low = false;
    let mut debouncing = false;
    let mut debounce_deadline = Instant::now();
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

        let pressed = button.is_low();
        if pressed != last_button_low {
            last_button_low = pressed;
            if pressed {
                debouncing = true;
                debounce_deadline =
                    Instant::now() + Duration::from_millis(pins::BUTTON_DEBOUNCE_MS as u64);
            }
        }

        if debouncing && Instant::now() >= debounce_deadline && pressed {
            debouncing = false;
            log::info!("Physical button -> relay pulse");
            pulse_relay(gpio);
            notify_physical_button();
        }
    }
}

fn pulse_relay(gpio: &GpioState) {
    if let (Ok(mut relay), Ok(mut debug)) = (gpio.relay.lock(), gpio.debug.lock()) {
        let _ = relay.set_high();
        let _ = debug.set_high();
        FreeRtos::delay_ms(pins::RELAY_PULSE_MS);
        let _ = relay.set_low();
        let _ = debug.set_low();
    }
}

unsafe extern "C" fn relay_pulse() {
    if let Some(gpio) = *GPIO.lock().unwrap() {
        pulse_relay(gpio);
    }
}
