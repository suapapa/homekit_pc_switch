idf_component_register(
    SRCS "pc_homekit.c"
    INCLUDE_DIRS "include"
    REQUIRES esp_hap_core esp_hap_apple_profiles app_wifi app_hap_setup_payload esp_event nvs_flash esp_wifi
)
