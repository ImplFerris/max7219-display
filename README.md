# MAX7219 Display Driver

A platform-agnostic, `no_std` driver for the MAX7219 LED display controller using [`embedded-hal`] traits.  
Supports both 8x8 LED matrix displays and 7-segment numeric displays.

---

## Features

- Supports multiple daisy-chained MAX7219 devices.
- Compatible with both LED matrix and 7-segment display configurations.
- Uses `embedded-hal` for SPI communication.
- `no_std` support.
- Flexible APIs for drawing characters, raw data, and controlling intensity.
- Easy-to-extend with custom fonts and display layouts.

---

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
# max7219-display = "0.1"
max7219-display = { git = "https://github.com/implferris/max7219-display" }
```

<!-- 
## Example (LED Matrix)

```rust
use max7219_display::{LedMatrix, fonts::get_char_bitmap};
use embedded_hal::spi::SpiDevice;

let matrix = LedMatrix::from_spi(spi, 1).unwrap();
matrix.draw_char(0, 'A').unwrap();
``` -->
