#![cfg_attr(not(feature = "std"), no_std)]

mod lib {
    #[cfg(feature="std")]
    pub use std::*;
    #[cfg(not(feature="std"))]
    pub use core::*;
}

extern crate embedded_types;
extern crate bit_field;

#[macro_use]
pub mod protocol1;
#[macro_use]
pub mod protocol2;
pub mod pro;
pub mod dynamixel;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    Timeout,
}

pub trait Servo {
    type OperatingModes;

    fn set_enable_torque(&mut self, enable_torque: bool) -> Result<(), Error>;
    
    /// Configure the servo into a specified operating mode.
    ///
    /// This allows use of that operting as a setpoint by calling `set_setpoint(&mut self, operating_mode, f32)` afterwards.
    fn set_operating_mode(&mut self, operating_mode: Self::OperatingModes) -> Result<(), Error>;

    /// Set the servo setpoint
    ///
    /// Requires that the servo is configured to the correct operating mode with `set_operating_mode` first.
    fn set_setpoint(&mut self, operating_mode: Self::OperatingModes, f32) -> Result<(), Error>;
    fn get_position(&mut self) -> Result<f32, Error>;
}

pub trait Interface {
    fn read(&mut self, &mut [u8]) -> Result<(), Error>;
    fn write(&mut self, &[u8]) -> Result<(), Error>;
}

#[cfg(feature = "std")]
impl<T: ::std::io::Read + ::std::io::Write> Interface for T {
    fn read(&mut self, buf: &mut [u8]) {
        self.read_exact(buf).unwrap();
    }
    
    fn write(&mut self, data: &[u8]) {
        self.write(data).unwrap();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
