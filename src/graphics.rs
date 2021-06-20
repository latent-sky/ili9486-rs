use crate::ILI9486;
use core::convert::TryInto;
use display_interface::{DisplayError, WriteOnlyDataCommand};
use embedded_graphics::pixelcolor::{
    raw::{RawData, RawU16},
    Rgb565,
};
use embedded_graphics::prelude::*;
use embedded_hal::digital::v2::OutputPin;

impl<CSX, RSX, IF> DrawTarget for ILI9486<CSX, RSX, IF>
where
    CSX: OutputPin,
    RSX: OutputPin,
    IF: WriteOnlyDataCommand,
{
    type Color = Rgb565;
    type Error = DisplayError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if let Ok((x @ 0..=320u32, y @ 0..=480u32)) = coord.try_into() {
                self.set_window(x as u16, y as u16, x as u16, y as u16)?;
                let mut pixel = [RawU16::from(color).into_inner()];
                self.write_pixels(&mut pixel)?;
            }
        }

        Ok(())
    }
}

impl<CSX, RSX, IF> OriginDimensions for ILI9486<CSX, RSX, IF>
where
    CSX: OutputPin,
    RSX: OutputPin,
    IF: WriteOnlyDataCommand,
{
    fn size(&self) -> Size {
        Size::new(320, 480)
    }
}
