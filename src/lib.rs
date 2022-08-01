#![no_std]

use core::iter;

use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::OutputPin;

mod graphics;

pub struct ILI9486<CSX, RSX, IF> {
    interface: IF,
    chip_select: CSX,
    reset: RSX,
    config: Config,
}

pub enum PixelFormat {
    Rgb565,
    Rgb666,
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

    pub fn init<D: DelayMs<u32>>(&mut self, delay: &mut D) -> Result<(), DisplayError> {
        self.disable()?;
        // Reset as per the data sheet
        //self.deassert_reset()?;
        self.assert_reset()?;
        delay.delay_ms(2);
        //self.assert_reset()?;
        self.deassert_reset()?;
        delay.delay_ms(200);

        self.enable()?;

        for cmd_seq in INIT_SEQ.iter() {
            let cmd = cmd_seq[0];
            if cmd != TFTLCD_DELAY8 {
                self.send_command_as_u8(cmd)?;
                self.send_data(&cmd_seq[1..cmd_seq.len()])?;
            } else {
                let delay_time_ms = cmd_seq[1];
                delay.delay_ms(delay_time_ms as u32);
            }
        }

        Ok(())
    }

    fn send_command_as_u8(&mut self, command_as_u8: u8) -> Result<(), DisplayError> {
        //let data = [command_as_u8 as u8];
        let data = [command_as_u8];
        self.interface.send_commands(DataFormat::U8(&data))
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
        self.send_data(&[format])
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

    pub fn set_brightness(&mut self, brightness: u8) -> Result<(), DisplayError> {
        self.send_command(Command::WriteDisplayBrightnessValue)?;
        self.send_data(&[brightness])
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
    NormalDisplayMode = 0x13,
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

    PowerControl2 = 0xC1,
    PowerControl3 = 0xC2,
    VCOMControl = 0xC5,
    PositiveGammaControl = 0xE0,
    NegativeGammaControl = 0xE1,

    WriteDisplayBrightnessValue = 0x51,
}

// Initialization sequence adapted from an Arduino ESP32  library https://github.com/schreibfaul1/ESP32-TFT-Library-ILI9486/blob/master/src/ili9486.cpp
const TFTLCD_DELAY8: u8 = 0x7F;
#[rustfmt::skip]
const INIT_SEQ: &[&[u8]] = &[
        &[0x01], // Soft reset
        &[TFTLCD_DELAY8, 120],// Required delay after soft reset (as per datasheet). FIXME this seems a hack. Find a better solution
        &[0x11], // Sleep out
        &[TFTLCD_DELAY8, 120],
        &[0x3A, 0x55],// Interface Pixel Format
        &[0xC2, 0x44],// Power Control 3 (For Normal Mode)
        &[0xC5, 0x00, 0x00, 0x00, 0x00],// VCOM Control
        &[0x13], // Normal Mode On 
        &[0xE0, 0x0F,0x1F, 0x1C, 0x0C, 0x0F, 0x08, 0x48, 0x98, 0x37, 0x0A, 0x13, 0x04, 0x11, 0x0D, 0x00],// PGAMCTRL(Positive Gamma Control)
        &[0xE1, 0x0F, 0x32, 0x2E, 0x0B, 0x0D, 0x05, 0x47, 0x75, 0x37, 0x06, 0x10, 0x03, 0x24, 0x20, 0x00],// NGAMCTRL (Negative Gamma Correction)
        &[0x20], // Display Inversion OFF   RPi LCD (A)
        // &[0x21], // Display Inversion ON    RPi LCD (B)
        &[0x36, 0x48], // Memory Access Control
        &[0x38],  //Idle Mode Off  
        &[0x29], // Display On 
        &[TFTLCD_DELAY8, 150],  // Delay after display on as per datasheet. FIXME this seems a hack. Find a better solution
];
