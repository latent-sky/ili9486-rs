# ili9486-rs

Rust library crate to use ILI9486-driven displays.

The library (originally forked from [here](https://github.com/andresovela/ili9486-rs)) works with version `0.7.1`of the crate [embedded-graphics](https://crates.io/crates/embedded-graphics).

## Usage

The following sketches how to use the crate:

1. First set up a `display interface` for the interface type (e.g. SPI, parallel 8080 type interface etc.). This should implement a generic
[`display_interface`](https://docs.rs/display-interface/0.4.1/display_interface/index.html), in particular the trait [`WriteOnlyDataCommand`](https://docs.rs/display-interface/0.4.1/display_interface/trait.WriteOnlyDataCommand.html).

2. Create the driver specifing the chip select (CS) and reset pin.

```rust
let mut lcd_driver = ILI9486::new(display_interface, lcd_cs_pin, lcd_reset_pin).unwrap();
```

3. Initialise the driver

```rust

lcd_driver.init(&mut delay).unwrap();
```

4. Set the orientation

```rust
lcd_driver.set_orientation(ili9486::Orientation::Landscape).unwrap();
```

5. Can now use the `embedded-graphics` functions to, for instance, draw a rectangle.

```rust
// Create a rectangle
let rect_style = PrimitiveStyleBuilder::new()
    .stroke_color(Rgb565::RED)
    .stroke_width(10)
    .fill_color(Rgb565::GREEN)
    .build();

Rectangle::new(Point::new(30, 20), Size::new(100, 70))
    .into_styled(rect_style)
    .draw(&mut lcd_driver)
    .unwrap();

```

*Instead of just using `unwrap` you may want to use sonething else to better handle errors.*

Examples are shon in the [`examples`](./examples) directory

## Status

See the [change log](CHANGELOG.md).

**Currently this seems to work, but was unable to continue testing as the display I was using went bust!**