pub mod parallel;
pub mod serial;

/// Sealed trait pattern
pub trait Interface: private::Interface {}

mod private {
    use crate::Command;
    pub trait Interface {
        fn write_command(&mut self, command: Command);
        fn write_data(&mut self, data: &[u8]);
        fn read_status(&mut self) -> u8;
        fn read_data(&mut self) -> u8;
    }
}
