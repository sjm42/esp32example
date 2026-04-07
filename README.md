# esp32example

Barebones ESP32 example firmware for Rust + ESP-IDF.

This repository is meant to be used as a starter framework for new projects. It
keeps the infrastructure that tends to be annoying to recreate every time, while
keeping the application logic deliberately minimal.

## What this project includes

- Tokio-based single-threaded async runtime on top of ESP-IDF / FreeRTOS
- WiFi station mode with WPA2 Personal and WPA2 Enterprise support
- automatic AP mode boot when WiFi is not configured
- short button press to request AP mode on next boot
- long button press for factory reset
- DHCP and static IPv4 configuration
- periodic gateway ping watchdog with reboot on connectivity loss
- Axum-based embedded web UI and JSON API
- OTA firmware update endpoint
- example MQTT publish and subscribe flow
- optional DS18B20 polling using the ESP-IDF RMT-backed 1-wire bus
- support for two hardware targets with helper flash / OTA-image scripts
- optional reusable display helper modules behind feature gates

## Hardware targets

Two targets are currently wired in:

- `esp32-c3` (default)
- `esp-wroom-32`

Target-specific details currently include:

- reset button GPIO
- activity LED GPIO
- available DS18B20 probe pin list
- build target / toolchain selection in helper scripts

If another board needs to be added, the main place to extend is
`src/bin/esp32example.rs`.

## Helper scripts

```bash
./flash_c3
./flash_wroom32
./make_ota_image_c3
./make_ota_image_wroom32
```

These are intentionally short and hardware-oriented:

- `flash_c3` builds and runs for the default ESP32-C3 target
- `flash_wroom32` builds and runs for Xtensa ESP32 / ESP-WROOM-32
- `make_ota_image_*` builds a firmware image suitable for OTA updates

## Cargo features

- `esp32-c3`
- `esp-wroom-32`
- `display-max7219`
- `display-ws2812`
- `reset_settings`

The display features currently expose reusable modules only. They are not
connected into the default example runtime, which keeps the base firmware small
for projects that do not need a display.

## Project layout

- `src/bin/esp32example.rs` firmware entry point and task orchestration
- `src/apiserver.rs` embedded web UI, JSON API, OTA endpoint
- `src/config.rs` runtime config model and NVS persistence
- `src/mqtt.rs` MQTT example publish / subscribe implementation
- `src/wifi.rs` WiFi station + AP mode setup and reconnection loop
- `src/measure.rs` DS18B20 scanning and periodic polling
- `src/rmt_ow.rs` local RMT-backed 1-wire wrapper
- `src/state.rs` shared application state
- `src/display.rs`, `src/font.rs`, `src/ws2812.rs` optional display helper modules
- `templates/index.html.ask` built-in configuration page
- `static/` embedded JavaScript, CSS and favicon

## Runtime behavior

The firmware launches these long-running tasks:

- button / uptime loop
- sensor polling loop
- MQTT loop
- HTTP API / web UI server
- WiFi management loop
- gateway ping watchdog

If WiFi credentials are missing from config, the device starts in AP mode
instead of station mode.

AP mode details:

- SSID: `esp32example`
- IP: `10.42.42.1`
- web UI: `http://10.42.42.1/`

## HTTP API

- `GET /` built-in configuration page
- `GET /state` example device state JSON
- `GET /sample` read the mutable sample payload
- `POST /sample` update the mutable sample payload
- `GET /config`
- `POST /config`
- `GET /reset_config`
- `GET /uptime`
- `GET /sensors`
- `GET /temp`
- `POST /fw`

The `/sample` endpoint is intentionally simple and is meant as a starting point
for project-specific JSON handlers.

## MQTT example topics

With:

- `mqtt_enable = true`
- `mqtt_topic = "esp32example"`

the firmware currently:

Publishes:

- `esp32example/state`
- `esp32example/uptime`
- `esp32example/sample`
- `esp32example/sensor/<sensor-id>`

Subscribes:

- `esp32example/cmd/sample`
- `esp32example/cmd/led`
- `esp32example/cmd/reboot`

## DS18B20 support

DS18B20 support uses the RMT-backed 1-wire implementation from the `esp32temp`
project rather than the older bit-banged path. Sensors are scanned at boot from
the board-specific pin list and may then be polled periodically if enabled in
config.

## Build and verification

Typical commands:

```bash
cargo fmt
cargo check
```

The current scaffold was verified with:

```bash
cargo fmt
cargo check --message-format=short
```

## Suggested next steps for derived projects

- replace the `/sample` endpoint and MQTT sample payload with real application data
- trim config fields that are not needed by the derived project
- add a board-specific pin-map module if target count grows
- decide whether display helpers should remain optional modules or become a real runtime task
