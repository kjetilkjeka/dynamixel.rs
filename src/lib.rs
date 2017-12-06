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


pub trait Servo {
    type OperatingModes;
    type Error;

    fn set_enable_torque(&mut self, enable_torque: bool) -> Result<(), Self::Error>;
    
    /// Configure the servo into a specified operating mode.
    ///
    /// This allows use of that operting as a setpoint by calling `set_setpoint(&mut self, operating_mode, f32)` afterwards.
    fn set_operating_mode(&mut self, operating_mode: Self::OperatingModes) -> Result<(), Self::Error>;

    /// Set the servo setpoint
    ///
    /// Requires that the servo is configured to the correct operating mode with `set_operating_mode` first.
    fn set_setpoint(&mut self, operating_mode: Self::OperatingModes, f32) -> Result<(), Self::Error>;
    fn get_position(&mut self) -> Result<f32, Self::Error>;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CommunicationError {
    Timeout,
    Other,
}

/// The interface for communicating with dynamixel servos.
pub trait Interface {
    /// A blocking/spinning read with timeout.
    ///
    /// This function should either:
    ///
    /// - read a number of bytes corresponding to `data.len()` into `data` and return `Ok(())`.
    /// - return `Err(_)`.
    ///
    /// If bytes are not received for a given time, a timeout should occur.
    /// A timeout is signaled by returning `Err(Error::Timeout)`.
    /// The time between bytes before a timeout occur should be 100ms or more.
    /// If the timeout is not implemented, a "dead" servo can cause the code to "freeze".
    fn read(&mut self, data: &mut [u8]) -> Result<(), CommunicationError>;

    /// A blocking/spinning write.
    ///
    /// This function should either:
    /// 
    /// - write every byte in `data` and return `Ok(())`.
    /// - return `Err(_)`.
    ///
    /// After a transmission is started the time between two consecutive bytes need to be less than 100ms.
    /// This is because the dynamixel actuator recognizes a time of more than 100ms between bytes as a communication problem.
    fn write(&mut self, data: &[u8]) -> Result<(), CommunicationError>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
