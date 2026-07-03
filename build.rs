fn main() -> anyhow::Result<()> {
    configure_wifi_from_env()?;
    embuild::espidf::sysenv::output();
    Ok(())
}

/// If WIFI_SSID + WIFI_PASS (or WIFI_PASSWORD) are set, write sdkconfig.defaults.env
/// so esp-idf-sys picks them up at build time (see Cargo.toml esp_idf_sdkconfig_defaults).
fn configure_wifi_from_env() -> anyhow::Result<()> {
    const ENV_DEFAULTS: &str = "sdkconfig.defaults.env";

    println!("cargo:rerun-if-env-changed=WIFI_SSID");
    println!("cargo:rerun-if-env-changed=WIFI_PASS");
    println!("cargo:rerun-if-env-changed=WIFI_PASSWORD");

    let ssid = std::env::var("WIFI_SSID").ok();
    let pass = std::env::var("WIFI_PASS")
        .or_else(|_| std::env::var("WIFI_PASSWORD"))
        .ok();

    match (ssid, pass) {
        (Some(ssid), Some(pass)) => {
            let content = format!(
                "CONFIG_APP_WIFI_USE_HARDCODED=y\nCONFIG_APP_WIFI_SSID=\"{}\"\nCONFIG_APP_WIFI_PASSWORD=\"{}\"\n",
                escape_kconfig_string(&ssid),
                escape_kconfig_string(&pass),
            );
            std::fs::write(ENV_DEFAULTS, content)?;
        }
        _ => {
            let _ = std::fs::remove_file(ENV_DEFAULTS);
        }
    }

    Ok(())
}

fn escape_kconfig_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
