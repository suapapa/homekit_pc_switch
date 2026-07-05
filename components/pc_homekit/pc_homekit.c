/*
 * HomeKit Switch accessory for PC power button relay.
 * Based on esp-homekit-sdk fan/smart_outlet examples.
 */

#include <stdio.h>
#include <string.h>

#include <freertos/FreeRTOS.h>
#include <freertos/task.h>
#include <esp_event.h>
#include <esp_log.h>
#include <esp_wifi.h>

#include <hap.h>
#include <hap_apple_servs.h>
#include <hap_apple_chars.h>

#include <app_wifi.h>
#include <app_hap_setup_payload.h>

#include "pc_homekit.h"

static const char *TAG = "pc_homekit";

#define PC_HOMEKIT_TASK_NAME      "pc_homekit"
#define PC_HOMEKIT_TASK_STACKSIZE (6 * 1024)
#define PC_HOMEKIT_TASK_PRIORITY  1

#define RELAY_PULSE_TASK_NAME     "relay_pulse"
#define RELAY_PULSE_TASK_STACK    (3 * 1024)
#define RELAY_PULSE_TASK_PRIORITY 2

/* Let the write response reach the controller before relay work starts. */
#define HAP_WRITE_SETTLE_MS       150
/* After relay pulse, wait before syncing Switch Off (matches Arduino setVal timing). */
#define HAP_OFF_SYNC_DELAY_MS     400

static hap_char_t *s_switch_on_char = NULL;
static pc_relay_pulse_fn s_relay_pulse = NULL;
static volatile bool s_relay_task_running = false;

/// Max TX power + no Wi-Fi power save — helps HomeKit discovery/pairing on ESP32-S2.
static void pc_homekit_wifi_boost(void)
{
    esp_err_t err = esp_wifi_set_max_tx_power(80); /* 80 × 0.25 dBm = 20 dBm */
    if (err != ESP_OK) {
        ESP_LOGW(TAG, "esp_wifi_set_max_tx_power: %s", esp_err_to_name(err));
    }
    err = esp_wifi_set_ps(WIFI_PS_NONE);
    if (err != ESP_OK) {
        ESP_LOGW(TAG, "esp_wifi_set_ps: %s", esp_err_to_name(err));
    }
    ESP_LOGI(TAG, "WiFi boost: TX 20 dBm, power save off");
}

void pc_homekit_set_relay_pulse(pc_relay_pulse_fn fn)
{
    s_relay_pulse = fn;
}

static void sync_switch_off(void)
{
    if (!s_switch_on_char) {
        return;
    }

    const hap_val_t *cur = hap_char_get_val(s_switch_on_char);
    if (cur && cur->b) {
        hap_val_t off = { .b = false };
        hap_char_update_val(s_switch_on_char, &off);
    }
}

static void relay_pulse_task(void *arg)
{
    (void)arg;

    vTaskDelay(pdMS_TO_TICKS(HAP_WRITE_SETTLE_MS));
    if (s_relay_pulse) {
        s_relay_pulse();
    }
    vTaskDelay(pdMS_TO_TICKS(HAP_OFF_SYNC_DELAY_MS));
    sync_switch_off();
    s_relay_task_running = false;
    vTaskDelete(NULL);
}

static bool schedule_relay_pulse(void)
{
    if (s_relay_task_running) {
        ESP_LOGW(TAG, "Relay pulse already in progress, ignoring");
        return false;
    }

    s_relay_task_running = true;
    if (xTaskCreate(relay_pulse_task, RELAY_PULSE_TASK_NAME, RELAY_PULSE_TASK_STACK,
                NULL, RELAY_PULSE_TASK_PRIORITY, NULL) != pdPASS) {
        s_relay_task_running = false;
        ESP_LOGE(TAG, "Failed to start relay pulse task");
        return false;
    }
    return true;
}

static int switch_identify(hap_acc_t *ha)
{
    ESP_LOGI(TAG, "Accessory identified");
    return HAP_SUCCESS;
}

static int switch_write(hap_write_data_t write_data[], int count,
        void *serv_priv, void *write_priv)
{
    int i, ret = HAP_SUCCESS;
    hap_val_t off = { .b = false };

    for (i = 0; i < count; i++) {
        hap_write_data_t *write = &write_data[i];
        if (!strcmp(hap_char_get_type_uuid(write->hc), HAP_CHAR_UUID_ON)) {
            ESP_LOGI(TAG, "Switch write: On=%d", write->val.b);
            if (write->val.b) {
                /* Momentary ATX button: return immediately (write response stays Off),
                 * pulse relay in a worker task, then sync Off after relay + settle time.
                 * Do not push On here — that double-updates the phone and causes UI stutter. */
                schedule_relay_pulse();
            } else {
                hap_char_update_val(write->hc, &off);
            }
            *(write->status) = HAP_STATUS_SUCCESS;
        } else {
            *(write->status) = HAP_STATUS_RES_ABSENT;
        }
    }
    return ret;
}

void pc_homekit_physical_button(void)
{
    schedule_relay_pulse();
}

static void pc_homekit_thread(void *arg)
{
    hap_acc_t *accessory;
    hap_serv_t *service;

    hap_init(HAP_TRANSPORT_WIFI);

    hap_acc_cfg_t cfg = {
        .name = "PC Power Button",
        .manufacturer = "Espressif",
        .model = "HAP-PC-BTN",
        .serial_num = "001122334455",
        .fw_rev = "0.1.0",
        .hw_rev = "0.1.0",
        .pv = "1.1.0",
        .identify_routine = switch_identify,
        .cid = HAP_CID_SWITCH,
    };

    accessory = hap_acc_create(&cfg);

    uint8_t product_data[] = {'E','S','P','3','2','H','A','P'};
    hap_acc_add_product_data(accessory, product_data, sizeof(product_data));
    hap_acc_add_wifi_transport_service(accessory, 0);

    service = hap_serv_switch_create(false);
    hap_serv_add_char(service, hap_char_name_create("PC Power Button"));
    hap_serv_set_write_cb(service, switch_write);
    s_switch_on_char = hap_serv_get_char_by_uuid(service, HAP_CHAR_UUID_ON);

    hap_acc_add_serv(accessory, service);
    hap_add_accessory(accessory);

#ifdef CONFIG_EXAMPLE_USE_HARDCODED_SETUP_CODE
    hap_set_setup_code(CONFIG_EXAMPLE_SETUP_CODE);
    hap_set_setup_id(CONFIG_EXAMPLE_SETUP_ID);
    app_hap_setup_payload(CONFIG_EXAMPLE_SETUP_CODE, CONFIG_EXAMPLE_SETUP_ID, false, cfg.cid);
#endif

    hap_enable_mfi_auth(HAP_MFI_AUTH_HW);

    app_wifi_init();
    hap_start();
    app_wifi_start(portMAX_DELAY);
    pc_homekit_wifi_boost();

    ESP_LOGI(TAG, "HomeKit started (setup code %s, setup id %s)", CONFIG_EXAMPLE_SETUP_CODE, CONFIG_EXAMPLE_SETUP_ID);
    vTaskDelete(NULL);
}

void pc_homekit_start(void)
{
    xTaskCreate(pc_homekit_thread, PC_HOMEKIT_TASK_NAME, PC_HOMEKIT_TASK_STACKSIZE,
                NULL, PC_HOMEKIT_TASK_PRIORITY, NULL);
}
