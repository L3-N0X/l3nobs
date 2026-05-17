#![no_std]
#![no_main]


use esp_hal::rmt::{Channel, PulseCode, Tx};
use esp_hal::gpio::Level;
use esp_hal::Async;
use rmk::channel::{CONTROLLER_CHANNEL, ControllerSub};
use rmk::controller::Controller;
use rmk::event::ControllerEvent;
use rmk::macros::rmk_keyboard;

const NUM_LEDS: usize = 11;
const NUM_UNIQUE: usize = (NUM_LEDS + 1) / 2;
const PULSE_COUNT: usize = NUM_LEDS * 24 + 1;

/// Gamma 2.8 correction table (standard Adafruit/FastLED table)
const GAMMA8: [u8; 256] = [
      0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
      0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  1,  1,  1,  1,
      1,  1,  1,  1,  1,  1,  1,  1,  1,  2,  2,  2,  2,  2,  2,  2,
      2,  3,  3,  3,  3,  3,  3,  3,  4,  4,  4,  4,  4,  5,  5,  5,
      5,  6,  6,  6,  6,  7,  7,  7,  7,  8,  8,  8,  9,  9,  9, 10,
     10, 10, 11, 11, 11, 12, 12, 13, 13, 13, 14, 14, 15, 15, 16, 16,
     17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22, 22, 23, 24, 24, 25,
     25, 26, 27, 27, 28, 29, 29, 30, 31, 32, 32, 33, 34, 35, 35, 36,
     37, 38, 39, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 50,
     51, 52, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 66, 67, 68,
     69, 70, 72, 73, 74, 75, 77, 78, 79, 81, 82, 83, 85, 86, 87, 89,
     90, 92, 93, 95, 96, 98, 99,101,102,104,105,107,109,110,112,114,
    115,117,119,120,122,124,126,127,129,131,133,135,137,138,140,142,
    144,146,148,150,152,154,156,158,160,162,164,167,169,171,173,175,
    177,180,182,184,186,189,191,193,196,198,200,203,205,208,210,213,
    215,218,220,223,225,228,231,233,236,239,241,244,247,249,252,255,
];

#[derive(Clone, Copy)]
struct RGB8 { r: u8, g: u8, b: u8 }

/// Hue 0-255 (full wheel), sat/val 0-255
fn hsv_to_rgb(hue: u8, sat: u8, val: u8) -> RGB8 {
    if sat == 0 {
        return RGB8 { r: val, g: val, b: val };
    }
    let region = hue / 43;
    let rem = (hue as u16 - region as u16 * 43) * 6; // 0-252

    let v = val as u16;
    let s = sat as u16;
    let p = ((v * (255 - s)) >> 8) as u8;
    let q = ((v * (255 - ((s * rem) >> 8))) >> 8) as u8;
    let t = ((v * (255 - ((s * (255 - rem)) >> 8))) >> 8) as u8;

    match region {
        0 => RGB8 { r: val, g: t,   b: p   },
        1 => RGB8 { r: q,   g: val, b: p   },
        2 => RGB8 { r: p,   g: val, b: t   },
        3 => RGB8 { r: p,   g: q,   b: val },
        4 => RGB8 { r: t,   g: p,   b: val },
        _ => RGB8 { r: val, g: p,   b: q   },
    }
}

fn gamma_correct(c: RGB8) -> RGB8 {
    RGB8 {
        r: GAMMA8[c.r as usize],
        g: GAMMA8[c.g as usize],
        b: GAMMA8[c.b as usize],
    }
}

/// Builds a mirrored gradient: physical layout 1 2 3 4 5 | 5 4 3 2 1
/// Each layer rotates the starting hue by ~60° (256/6 ≈ 43)
fn layer_gradient(layer: u8) -> [RGB8; NUM_LEDS] {
    let base_hue: u8 = layer.wrapping_mul(43);
    const HUE_STEP: u8 = 8;

    let mut colors = [RGB8 { r: 0, g: 0, b: 0 }; NUM_LEDS];

    for i in 0..NUM_UNIQUE {
        let hue = base_hue.wrapping_add(i as u8 * HUE_STEP);
        let color = gamma_correct(hsv_to_rgb(hue, 220, 180));
        colors[i] = color;                    // left half
        colors[NUM_LEDS - 1 - i] = color;    // mirrored right half
    }
    colors
}

pub struct Ws2812LayerController<'ch> {
    channel: Channel<'ch, Async, Tx>,
    sub: ControllerSub,
    pulse_buffer: [PulseCode; PULSE_COUNT],
}

impl<'ch> Ws2812LayerController<'ch> {
    pub fn new(channel: Channel<'ch, Async, Tx>) -> Self {
        Self {
            channel,
            sub: CONTROLLER_CHANNEL.subscriber().unwrap(),
            pulse_buffer: [PulseCode::end_marker(); PULSE_COUNT],
        }
    }

    fn encode_ws2812_in_place(&mut self, colors: &[RGB8; NUM_LEDS]) {
        for (led, color) in colors.iter().enumerate() {
            let bits = ((color.g as u32) << 16) | ((color.r as u32) << 8) | (color.b as u32);
            for bit in 0..24 {
                self.pulse_buffer[led * 24 + bit] = if (bits >> (23 - bit)) & 1 == 1 {
                    PulseCode::new(Level::High, 64, Level::Low, 36)
                } else {
                    PulseCode::new(Level::High, 32, Level::Low, 68)
                };
            }
        }
    }
}

impl<'ch> Controller for Ws2812LayerController<'ch> {
    type Event = ControllerEvent;

    async fn process_event(&mut self, event: Self::Event) {
        if let ControllerEvent::Layer(layer) = event {
            let colors = layer_gradient(layer as u8);

            // Encode directly into our struct's memory
            self.encode_ws2812_in_place(&colors);

            // 3. Non-blocking Async transmission! This yields back to the executor.
            self.channel.transmit(&self.pulse_buffer).await.unwrap();
        }
    }

    async fn next_message(&mut self) -> Self::Event {
        self.sub.next_message_pure().await
    }
}

#[rmk_keyboard]
mod keyboard {
    #[controller(event)]
        fn ws2812_layer_led() -> Ws2812LayerController {
            use esp_hal::rmt::{Rmt, TxChannelConfig,TxChannelCreator};
            use esp_hal::gpio::Level;
            use esp_hal::time::Rate;

            let rmt = Rmt::new(p.RMT, Rate::from_mhz(80)).unwrap().into_async();

            // 2. Because `rmt` is now async, `configure_tx` returns an Async channel
            let async_channel = rmt.channel0.configure_tx(
                p.GPIO33,
                TxChannelConfig::default()
                    .with_clk_divider(1)
                    .with_idle_output_level(Level::Low)
                    .with_idle_output(true)
                    .with_carrier_modulation(false),
            ).unwrap();

            Ws2812LayerController::new(async_channel)
        }
}
