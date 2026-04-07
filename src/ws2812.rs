use esp_idf_hal::{
    delay::Ets,
    gpio::OutputPin,
    rmt::{
        PinState, Symbol, TxChannelDriver,
        config::{MemoryAccess, TransmitConfig, TxChannelConfig},
        encoder::{BytesEncoder, BytesEncoderConfig},
    },
    units::Hertz,
};

use crate::*;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 8;
const TILE_WIDTH: usize = 8;
const TILE_HEIGHT: usize = 8;
const TILE_COUNT: usize = DISPLAY_WIDTH / TILE_WIDTH;

const PANEL_WIDTH: usize = 32;
const PANEL_HEIGHT: usize = 8;
const PANEL_PIXELS: usize = PANEL_WIDTH * PANEL_HEIGHT;

const PIXEL_COUNT: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;
const TX_BYTES: usize = PIXEL_COUNT * 3;

const RMT_RESOLUTION: Hertz = Hertz(10_000_000);

const T0H: Duration = Duration::from_nanos(350);
const T0L: Duration = Duration::from_nanos(800);
const T1H: Duration = Duration::from_nanos(700);
const T1L: Duration = Duration::from_nanos(600);

const RESET_US: u32 = 80;

const COLOR_R: u8 = 255;
const COLOR_G: u8 = 0;
const COLOR_B: u8 = 0;
const MAX_BRIGHTNESS: u8 = 24;
// Tweak these if the chained panels appear mirrored or ordered incorrectly.
const CHAIN_REVERSED: bool = false;
const FLIP_X: bool = true;
const FLIP_Y: bool = true;
const EVEN_COLUMN_TOP_TO_BOTTOM: bool = true;

pub struct LedMatrix<'a> {
    tx: TxChannelDriver<'a>,
    encoder: BytesEncoder,
    pixels: [bool; PIXEL_COUNT],
    tx_buf: [u8; TX_BYTES],
    brightness: u8,
    powered_on: bool,
}

impl<'a> LedMatrix<'a> {
    pub fn new(pin: impl OutputPin + 'a) -> anyhow::Result<Self> {
        let tx = TxChannelDriver::new(
            pin,
            &TxChannelConfig {
                resolution: RMT_RESOLUTION,
                memory_access: MemoryAccess::Indirect {
                    memory_block_symbols: 64,
                },
                transaction_queue_depth: 1,
                ..Default::default()
            },
        )?;

        let bit0 = Symbol::new_with(RMT_RESOLUTION, PinState::High, T0H, PinState::Low, T0L)?;
        let bit1 = Symbol::new_with(RMT_RESOLUTION, PinState::High, T1H, PinState::Low, T1L)?;
        let encoder = BytesEncoder::with_config(&BytesEncoderConfig {
            bit0,
            bit1,
            msb_first: true,
            ..Default::default()
        })?;

        Ok(Self {
            tx,
            encoder,
            pixels: [false; PIXEL_COUNT],
            tx_buf: [0; TX_BYTES],
            brightness: 0,
            powered_on: false,
        })
    }

    pub fn power_on(&mut self) -> anyhow::Result<()> {
        if !self.powered_on {
            self.powered_on = true;
            self.flush()?;
        }

        Ok(())
    }

    pub fn power_off(&mut self) -> anyhow::Result<()> {
        if self.powered_on {
            self.powered_on = false;
            self.flush()?;
        }

        Ok(())
    }

    pub fn clear_display(&mut self, display: usize) -> anyhow::Result<()> {
        if display >= TILE_COUNT {
            return Ok(());
        }

        let base_x = display * TILE_WIDTH;
        for y in 0..TILE_HEIGHT {
            for x in 0..TILE_WIDTH {
                self.set_pixel(base_x + x, y, false);
            }
        }

        Ok(())
    }

    pub fn set_intensity(&mut self, _display: usize, intensity: u8) -> anyhow::Result<()> {
        self.brightness = intensity_to_brightness(intensity);
        Ok(())
    }

    pub fn write_raw(&mut self, display: usize, rows: &[u8; 8]) -> anyhow::Result<()> {
        if display >= TILE_COUNT {
            return Ok(());
        }

        let base_x = display * TILE_WIDTH;
        for (y, row) in rows.iter().enumerate().take(TILE_HEIGHT) {
            for x in 0..TILE_WIDTH {
                let on = (*row & (0x80 >> x)) != 0;
                self.set_pixel(base_x + x, y, on);
            }
        }

        Ok(())
    }

    pub fn flush(&mut self) -> anyhow::Result<()> {
        self.rebuild_tx_buf();
        self.send_tx_buf()
    }

    fn set_pixel(&mut self, x: usize, y: usize, on: bool) {
        if let Some(index) = xy_to_index(x, y) {
            self.pixels[index] = on;
        }
    }

    fn rebuild_tx_buf(&mut self) {
        let (r, g, b) = if self.powered_on && self.brightness > 0 {
            (
                scale_channel(COLOR_R, self.brightness),
                scale_channel(COLOR_G, self.brightness),
                scale_channel(COLOR_B, self.brightness),
            )
        } else {
            (0, 0, 0)
        };

        for (src, dst) in self.pixels.iter().zip(self.tx_buf.chunks_exact_mut(3)) {
            if *src {
                dst[0] = g;
                dst[1] = r;
                dst[2] = b;
            } else {
                dst[0] = 0;
                dst[1] = 0;
                dst[2] = 0;
            }
        }
    }

    fn send_tx_buf(&mut self) -> anyhow::Result<()> {
        let config = TransmitConfig::default();
        unsafe {
            self.tx
                .start_send(&mut self.encoder, &self.tx_buf, &config)?;
        }
        self.tx.wait_all_done(None)?;
        Ets::delay_us(RESET_US);
        Ok(())
    }
}

fn scale_channel(channel: u8, brightness: u8) -> u8 {
    ((u16::from(channel) * u16::from(brightness)) / u16::from(MAX_BRIGHTNESS)) as u8
}

fn intensity_to_brightness(intensity: u8) -> u8 {
    intensity.min(15) + 1
}

fn xy_to_index(x: usize, y: usize) -> Option<usize> {
    if x >= DISPLAY_WIDTH || y >= DISPLAY_HEIGHT {
        return None;
    }

    let x = if FLIP_X { DISPLAY_WIDTH - 1 - x } else { x };
    let y = if FLIP_Y { DISPLAY_HEIGHT - 1 - y } else { y };
    let panel = x / PANEL_WIDTH;
    let local_x = x % PANEL_WIDTH;
    let column_base = local_x * PANEL_HEIGHT;
    let column_runs_top_to_bottom = if local_x % 2 == 0 {
        EVEN_COLUMN_TOP_TO_BOTTOM
    } else {
        !EVEN_COLUMN_TOP_TO_BOTTOM
    };
    let offset = if column_runs_top_to_bottom {
        y
    } else {
        PANEL_HEIGHT - 1 - y
    };

    let panel = if CHAIN_REVERSED {
        (DISPLAY_WIDTH / PANEL_WIDTH) - 1 - panel
    } else {
        panel
    };

    Some(panel * PANEL_PIXELS + column_base + offset)
}
