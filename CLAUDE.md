# CLAUDE.md

This repository is an ESP32 firmware starter project, not a finished appliance.
Treat it as a reusable framework with deliberately minimal application logic.

## Primary goal

Preserve and improve the reusable infrastructure:

- WiFi station mode
- AP fallback mode
- factory reset flow
- static IP support
- ping watchdog
- HTTP config / OTA UI
- JSON API examples
- MQTT examples
- optional DS18B20 support
- optional display helper modules
- multi-target build support

Avoid turning it back into a project-specific app.

## Current architecture

- `src/bin/esp32example.rs` owns startup, pin setup, task orchestration, reset button handling and ping watchdog
- `src/apiserver.rs` serves the Askama UI and JSON endpoints
- `src/config.rs` stores runtime config in NVS using postcard + CRC32
- `src/mqtt.rs` provides a minimal but practical MQTT example
- `src/wifi.rs` manages station mode, AP mode and reconnect behavior
- `src/measure.rs` handles DS18B20 scan and polling
- `src/rmt_ow.rs` is the local wrapper around the ESP-IDF `onewire_bus` component
- `src/display.rs`, `src/font.rs`, `src/ws2812.rs` are optional helpers behind feature flags

## Project rules

- Keep the example logic small and generic
- Prefer extending the scaffold in reusable ways instead of adding narrow product behavior
- Keep board-specific differences explicit and localized
- Do not remove WPA2 Enterprise, AP mode, or static IP support unless explicitly requested
- Do not replace the RMT-backed DS18B20 implementation with the older bit-banged one
- Keep helper script names short: `flash_c3`, `flash_wroom32`, `make_ota_image_c3`, `make_ota_image_wroom32`

## Build expectations

Default target is ESP32-C3.

Useful commands:

```bash
cargo fmt
cargo check
./flash_c3
./flash_wroom32
./make_ota_image_c3
./make_ota_image_wroom32
```

Xtensa builds use:

```bash
MCU=esp32 cargo +esp ...
```

## When editing

- Update `README.md` if public behavior, endpoints, features or build steps change
- Keep `CLAUDE.md` and `AGENTS.md` aligned with repo conventions if workflow guidance changes
- Prefer feature-gating optional subsystems rather than branching the default runtime
- Preserve the repo as a good clean example for future project clones
