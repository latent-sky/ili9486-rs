#![no_std]

use core::fmt::Debug;
use embedded_hal::digital::v2::OutputPin;
use interface::Interface;

pub mod interface;

pub struct ILI9486<CSX, RSX, IF> {
    interface: IF,
    chip_select: CSX,
    reset: RSX,
}

impl<CSX, RSX, IF> ILI9486<CSX, RSX, IF>
where
    IF: Interface,
    CSX: OutputPin,
    RSX: OutputPin,
    CSX::Error: Debug,
    RSX::Error: Debug,
{
    pub fn new(interface: IF, mut chip_select: CSX, mut reset: RSX) -> ILI9486<CSX, RSX, IF> {
        chip_select.set_high().unwrap();
        reset.set_high().unwrap();

        ILI9486 {
            interface,
            chip_select,
            reset,
        }
    }

    pub fn assert_reset(&mut self) {
        self.reset.set_low().unwrap();
    }

    pub fn deassert_reset(&mut self) {
        self.reset.set_high().unwrap();
    }

    pub fn enable(&mut self) {
        self.chip_select.set_low().unwrap();
    }

    pub fn disable(&mut self) {
        self.chip_select.set_high().unwrap();
    }

    pub fn send_command(&mut self, command: Command) {
        self.interface.write_command(command);
    }

    pub fn send_data(&mut self, data: &[u8]) {
        self.interface.write_data(data);
    }

    pub fn read_status(&mut self) -> u8 {
        self.interface.read_status()
    }

    pub fn read_data(&mut self) -> u8 {
        self.interface.read_data()
    }

    pub fn set_address(&mut self) {
        self.interface.write_command(Command::ColumnAddressSet);
        let column_data = [0x00, 0x00, 0x01, 0x40];
        self.interface.write_data(&column_data);

        self.interface.write_command(Command::PageAddressSet);
        let page_data = [0x00, 0x00, 0x01, 0xE0];
        self.interface.write_data(&page_data);
    }

    pub fn set_pixel_format(&mut self) {
        self.interface.write_command(Command::InterfacePixelFormat);
        let format = 0x55;
        self.interface.write_data(&[format]);
    }

    pub fn set_orientation(&mut self) {
        self.interface.write_command(Command::MemoryAccessControl);
        let data = 0x48;
        self.interface.write_data(&[data]);
    }

    pub fn fill(&mut self, color: u16) {
        self.interface.write_command(Command::MemoryWrite);
        for _ in 0..320 {
            for _ in 0..480 {
                let data = [(color >> 8) as u8, color as u8];
                self.interface.write_data(&data);
            }
        }
        self.interface.write_command(Command::Nop);
    }
}

#[repr(u8)]
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
}
