# max7219-display: MAX7219 Display Driver

A platform-agnostic, `no_std` driver for the MAX7219 LED display controller using [`embedded-hal`](https://docs.rs/embedded-hal/latest/embedded_hal/) traits.  Supports both 8x8 LED matrix displays and 7-segment numeric displays.


## Crate Features

This driver supports multiple daisy-chained MAX7219 devices and works with both 7-segment and LED matrix configurations. 

It includes built-in support for scrolling text, displaying characters, and rendering custom patterns on matrix displays. Support for custom fonts. 

Additional features can be enabled by adding the following to your `Cargo.toml`:

- `led-matrix` - provides utility functions for working with 8x8 LED matrix displays, including text rendering, scrolling, and pattern display.
- `graphics` - integrates with the [`embedded-graphics-core`](https://docs.rs/embedded-graphics-core) crate to enable drawing text, shapes, and images on LED matrix displays.
- `seven-segment` - adds helper functions for 7-segment numeric displays, such as printing digits and supported characters.


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
