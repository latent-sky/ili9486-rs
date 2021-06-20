#![no_std]

use core::iter;

use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_hal::digital::v2::OutputPin;


pub struct ILI9486<CSX, RSX, IF> {
    interface: IF,
    chip_select: CSX,
    reset: RSX,
    config: Config,
}

struct Config {
    width: u16,
    height: u16,
    orientation: Orientation,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            width: 320,
            height: 480,
            orientation: Orientation::Portrait,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Orientation {
    Portrait,
    Landscape,
    PortraitUpsideDown,
    LandscapeUpsideDown,
}

impl<CSX, RSX, IF> ILI9486<CSX, RSX, IF>
where
    IF: WriteOnlyDataCommand,
    CSX: OutputPin,
    RSX: OutputPin,
{
    const WIDTH: u16 = 320;
    const HEIGHT: u16 = 480;

    pub fn new(
        interface: IF,
        mut chip_select: CSX,
        mut reset: RSX,
    ) -> Result<ILI9486<CSX, RSX, IF>, DisplayError> {
        chip_select.set_high().map_err(|_| DisplayError::CSError)?;
        reset.set_high().map_err(|_| DisplayError::RSError)?;

        Ok(ILI9486 {
            interface,
            chip_select,
            reset,
            config: Config::default(),
        })
    }

    pub fn assert_reset(&mut self) -> Result<(), DisplayError> {
        self.reset.set_low().map_err(|_| DisplayError::RSError)
    }

    pub fn deassert_reset(&mut self) -> Result<(), DisplayError> {
        self.reset.set_high().map_err(|_| DisplayError::RSError)
    }

    pub fn enable(&mut self) -> Result<(), DisplayError> {
        self.chip_select
            .set_low()
            .map_err(|_| DisplayError::CSError)
    }

    pub fn disable(&mut self) -> Result<(), DisplayError> {
        self.chip_select
            .set_high()
            .map_err(|_| DisplayError::CSError)
    }

    pub fn send_command(&mut self, command: Command) -> Result<(), DisplayError> {
        let data = [command as u8];
        self.interface.send_commands(DataFormat::U8(&data))
    }

    pub fn send_data(&mut self, data: &[u8]) -> Result<(), DisplayError> {
        self.interface.send_data(DataFormat::U8(data))
    }

    pub fn write_pixels(&mut self, pixels: &mut [u16]) -> Result<(), DisplayError> {
        self.send_command(Command::MemoryWrite)?;
        self.interface.send_data(DataFormat::U16BE(pixels))
    }

    pub fn set_window(&mut self, x0: u16, y0: u16, x1: u16, y1: u16) -> Result<(), DisplayError> {
        self.send_command(Command::ColumnAddressSet)?;
        let mut column_data = [x0, x1];
        self.interface
            .send_data(DataFormat::U16BE(&mut column_data))?;

        self.send_command(Command::PageAddressSet)?;
        let mut page_data = [y0, y1];
        self.interface.send_data(DataFormat::U16BE(&mut page_data))
    }

    pub fn set_pixel_format(&mut self, format: PixelFormat) -> Result<(), DisplayError> {
        self.send_command(Command::InterfacePixelFormat)?;
        let format = match format {
            PixelFormat::Rgb565 => 0x55,
            PixelFormat::Rgb666 => 0x66,
        };
        self.send_data(&[format])?;
        Ok(())
    }

    pub fn set_orientation(&mut self, orientation: Orientation) -> Result<(), DisplayError> {
        match orientation {
            Orientation::Portrait | Orientation::PortraitUpsideDown => {
                self.config.height = Self::HEIGHT;
                self.config.width = Self::WIDTH;
            }
            Orientation::Landscape | Orientation::LandscapeUpsideDown => {
                self.config.height = Self::WIDTH;
                self.config.width = Self::HEIGHT;
            }
        };

        let data = match orientation {
            Orientation::Portrait => 0x40,
            Orientation::PortraitUpsideDown => 0x80 | 0x10,
            Orientation::Landscape => 0x20,
            Orientation::LandscapeUpsideDown => 0x80 | 0x40 | 0x20 | 0x10 | 0x04,
        };

        self.send_command(Command::MemoryAccessControl)?;
        self.send_data(&[data])
    }

    pub fn orientation(&self) -> Orientation {
        self.config.orientation
    }

    pub fn fill(&mut self, color: u16) -> Result<(), DisplayError> {
        let pixel_count: usize = self.config.width as usize * self.config.height as usize;
        let mut pixels = iter::repeat(color).take(pixel_count);
        self.set_window(0, 0, self.config.width, self.config.height)?;
        self.send_command(Command::MemoryWrite)?;
        self.interface
            .send_data(DataFormat::U16BEIter(&mut pixels))?;
        self.send_command(Command::Nop)
    }

    pub fn width(&self) -> u16 {
        self.config.width
    }

    pub fn height(&self) -> u16 {
        self.config.height
    }
}

#[repr(u8)]
#[non_exhaustive]
pub enum Command {
    Nop = 0x00,
    SoftReset = 0x01,
    SleepIn = 0x10,
    SleepOut = 0x11,
    DisplayInversionOff = 0x20,
    DisplayInversionOn = 0x21,
    DisplayOff = 0x28,
    DisplayOn = 0x29,
    ColumnAddressSet = 0x2A,
    PageAddressSet = 0x2B,
    MemoryWrite = 0x2C,
    MemoryRead = 0x2E,
    MemoryAccessControl = 0x36,
    IdleModeOff = 0x38,
    IdleModeOn = 0x39,
    InterfacePixelFormat = 0x3A,
    DisplayInversionControl = 0xB4,
    DisplayFunctionControl = 0xB6,
}
