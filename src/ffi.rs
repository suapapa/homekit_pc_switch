//! FFI to the esp-homekit-sdk wrapper component.

extern "C" {
    fn pc_homekit_set_relay_pulse(fn_ptr: Option<unsafe extern "C" fn()>);
    fn pc_homekit_start();
    fn pc_homekit_physical_button();
}

static mut RELAY_PULSE_FN: Option<unsafe extern "C" fn()> = None;

unsafe extern "C" fn relay_pulse_trampoline() {
    if let Some(f) = RELAY_PULSE_FN {
        f();
    }
}

pub fn set_relay_pulse_handler(handler: unsafe extern "C" fn()) {
    unsafe {
        RELAY_PULSE_FN = Some(handler);
        pc_homekit_set_relay_pulse(Some(relay_pulse_trampoline));
    }
}

pub fn start_homekit() {
    unsafe { pc_homekit_start() };
}

pub fn notify_physical_button() {
    unsafe { pc_homekit_physical_button() };
}
