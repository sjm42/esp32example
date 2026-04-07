# AGENTS.md

## Repo purpose

This repository is a minimal ESP32 Rust + ESP-IDF starter framework. It should
stay generic. New code should bias toward reusable scaffolding rather than
project-specific behavior.

## High-value areas to preserve

- WiFi station mode with WPA2 Personal and WPA2 Enterprise
- AP-mode fallback and manual setup flow
- short-press AP request and long-press factory reset
- DHCP and static IPv4 support
- periodic connectivity watchdog ping
- embedded Axum UI and JSON API
- OTA update support
- example MQTT publish / subscribe flow
- optional DS18B20 polling through the RMT-backed 1-wire path
- optional display helpers behind feature gates
- support for both `esp32-c3` and `esp-wroom-32`

## Code map

- `src/bin/esp32example.rs`: startup, pins, orchestration, reset button logic, watchdog
- `src/apiserver.rs`: web UI, JSON API, OTA endpoint
- `src/config.rs`: config model and NVS persistence
- `src/state.rs`: shared runtime state
- `src/wifi.rs`: WiFi client/AP setup and reconnect handling
- `src/mqtt.rs`: MQTT example behavior
- `src/measure.rs`: DS18B20 scan/poll logic
- `src/rmt_ow.rs`: RMT-backed 1-wire wrapper
- `src/display.rs`, `src/font.rs`, `src/ws2812.rs`: optional display support

## Working conventions

- Keep example endpoints simple and generic
- If adding new user-visible functionality, document it in `README.md`
- If adding project-specific sample behavior, keep it obviously replaceable
- Prefer additive changes over broad churn in startup/runtime wiring
- Do not silently drop cross-target support
- Do not silently remove feature gates for optional subsystems

## Verification

At minimum, run:

```bash
cargo fmt
cargo check --message-format=short
```

If script behavior changes, re-check:

```bash
./flash_c3
./flash_wroom32
./make_ota_image_c3
./make_ota_image_wroom32
```
