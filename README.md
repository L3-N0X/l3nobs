# L3NOBS Macro Keyboard

L3NOBS is a custom macro keyboard powered by an ESP32-S3, featuring a unique 13-key layout, a rotary encoder, and an 11-LED WS2812 strip with dynamic, layer-aware mirrored gradient lighting.

## Hardware Specifications

- **Microcontroller:** ESP32-S3
- **Matrix:** 4x4 physical matrix (13 keys wired)
- **Rotary Encoder:** 1 encoder on GPIO16/17
- **RGB Lighting:** 11 WS2812 LEDs on GPIO33
- **Connectivity:** USB and BLE supported

## Keyboard Layout

The physical layout follows a 4x4 grid but utilizes 13 keys mapped as follows:

```
(0,1) (0,2) (0,3)
(1,0) (1,1) (1,2) (1,3)
(2,0) (2,1) (2,2) (2,3)
      (3,1) (3,2)
```

## Layers

L3NOBS features 5 layers, each with a distinct purpose and a corresponding RGB color theme.

### Layer 0: Base (Multimeda)
- **Keys:** Mute, Play/Pause, Previous/Next Track, Rewind/Fast Forward.
- **Navigation:** Quick access to all other layers.
- **Shortcuts:** Control Panel, Calculator.
- **Encoder:** System Volume Control.

### Layer 1: Action (F-Keys)
- **Keys:** Mapped to F13 through F24 for custom macros.
- **Encoder:** Screen Brightness Control.

### Layer 2: Mouse
- **Keys:** Full mouse control (8 buttons) and 4-way scroll wheel emulation.
- **Encoder:** Mouse Wheel (Vertical).

### Layer 3: Layer 3
- **Keys:** Mapped to F13 through F24.
- **Encoder:** Mouse Wheel (Horizontal).

### Layer 4: Layer 4
- **Keys:** Mapped to F13 through F24.
- **Encoder:** Inverted Brightness Control.

## RGB Gradient Lighting

The keyboard features a custom `Ws2812LayerController` that updates the lighting based on the active layer.

### Mirrored Gradient Logic
The 11 LEDs are driven using a mirrored gradient algorithm. Instead of a linear progression, the color starts at the edges and meets in the middle, creating a symmetrical visual effect.
- **Physical Layout:** `1 2 3 4 5 6 5 4 3 2 1`
- **Algorithm:** The code calculates 6 unique colors for a half-strip and mirrors them across the center LED.

### Layer-Based Color Shifting
Each layer rotates the "Base Hue" by approximately 60 degrees (43 units on a 255-step color wheel), ensuring each layer has a distinct and recognizable color profile:
- **Layer 0:** Reds/Oranges
- **Layer 1:** Yellows/Greens
- **Layer 2:** Greens/Cyans
- **Layer 3:** Blues/Purples
- **Layer 4:** Purples/Magental

### Technical Implementation
- **Async RMT:** Uses the ESP32-S3 RMT (Remote Control) peripheral in async mode for non-blocking LED updates.
- **Gamma Correction:** A 2.8 gamma correction table is applied to all RGB values for more natural color transitions and accurate brightness.

## Development

### Prerequisites
- [Rust Toolchain for ESP32](https://docs.esp-rs.org/book/installation/index.html)
- `espflash`: `cargo install cargo-espflash espflash`

### Building and Flashing
To build and flash the firmware to your device:

```bash
cargo run --release
```

### Configuration
The keyboard behavior, pins, and layout are defined in `keyboard.toml`.
