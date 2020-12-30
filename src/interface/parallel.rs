use core::fmt::Debug;

use embedded_hal::digital::v2::OutputPin;

use crate::interface::private::Interface as PrivateInterface;
use crate::interface::Interface as PublicInterface;
use crate::Command;

pub struct Parallel8Bit<WRX, RDX, DCX, DB> {
    wrx: WRX,
    rdx: RDX,
    dcx: DCX,
    dbus: [DB; 8],
}

impl<WRX, RDX, DCX, DB> Parallel8Bit<WRX, RDX, DCX, DB>
where
    WRX: OutputPin,
    RDX: OutputPin,
    DCX: OutputPin,
    DB: OutputPin,
    WRX::Error: Debug,
    RDX::Error: Debug,
    DCX::Error: Debug,
    DB::Error: Debug,
{
    pub fn new(
        mut wrx: WRX,
        mut rdx: RDX,
        mut dcx: DCX,
        mut dbus: [DB; 8],
    ) -> Parallel8Bit<WRX, RDX, DCX, DB> {
        wrx.set_high().unwrap();
        rdx.set_high().unwrap();
        dcx.set_high().unwrap();

        for pin in &mut dbus {
            pin.set_high().unwrap();
        }

        Parallel8Bit {
            wrx,
            rdx,
            dcx,
            dbus,
        }
    }

    fn write_to_dbus(&mut self, byte: u8) {
        for i in 0..8 {
            if byte & (1 << i) == 0 {
                self.dbus[i].set_low().unwrap();
            } else {
                self.dbus[i].set_high().unwrap();
            }
        }
    }
}

impl<WRX, RDX, DCX, DB> PublicInterface for Parallel8Bit<WRX, RDX, DCX, DB>
where
    WRX: OutputPin,
    RDX: OutputPin,
    DCX: OutputPin,
    DB: OutputPin,
    WRX::Error: Debug,
    RDX::Error: Debug,
    DCX::Error: Debug,
    DB::Error: Debug,
{
}

impl<WRX, RDX, DCX, DB> PrivateInterface for Parallel8Bit<WRX, RDX, DCX, DB>
where
    WRX: OutputPin,
    RDX: OutputPin,
    DCX: OutputPin,
    DB: OutputPin,
    WRX::Error: Debug,
    RDX::Error: Debug,
    DCX::Error: Debug,
    DB::Error: Debug,
{
    fn write_command(&mut self, command: Command) {
        self.write_to_dbus(command as u8);
        self.dcx.set_low().unwrap();
        self.wrx.set_low().unwrap();
        self.wrx.set_high().unwrap();
        self.dcx.set_high().unwrap();
    }

    fn write_data(&mut self, data: &[u8]) {
        for byte in data {
            self.write_to_dbus(*byte);
            self.wrx.set_low().unwrap();
            self.wrx.set_high().unwrap();
        }
    }

    fn read_data(&mut self) -> u8 {
        0
    }

    fn read_status(&mut self) -> u8 {
        0
    }
}
