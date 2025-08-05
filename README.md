# max7219-display: MAX7219 Display Driver

A platform-agnostic, `no_std` driver for the MAX7219 LED display controller using [`embedded-hal`](https://docs.rs/embedded-hal/latest/embedded_hal/) traits.  Supports both 8x8 LED matrix displays and 7-segment numeric displays.


## Features

- Supports multiple daisy-chained MAX7219 devices  
- Compatible with both LED matrix and 7-segment display configurations  
- Supports scrolling text, printing characters, and displaying patterns on LED matrices  
- Easy to extend with custom fonts and display layouts  
- Optional `embedded-graphics` integration via feature flag  


## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
max7219-display = {version="0.1", features=["led-matrix", "seven-segment", "graphics"]}
# max7219-display = { git = "https://github.com/implferris/max7219-display" }
```

## Examples

Comprehensive example projects are available in the separate [max7219-examples](https://github.com/implferris/max7219-examples) repository.

## License

This project is licensed under the MIT License.
