#!/bin/bash

export WIFI_SSID="YOUR_WIFI_SSID"
export WIFI_PASS="YOUR_WIFI_PASSWORD"

unset IDF_PATH
cargo run --release
