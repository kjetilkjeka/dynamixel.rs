#![cfg_attr(not(feature = "std"), no_std)]

mod lib {
    #[cfg(feature="std")]
    pub use std::*;
    #[cfg(not(feature="std"))]
    pub use core::*;
}

#[macro_use]
extern crate log;

extern crate embedded_types;
extern crate bit_field;

#[cfg(feature="serialport")]
extern crate serialport;


#[macro_use]
pub mod protocol1;
#[macro_use]
pub mod protocol2;
pub mod pro;
pub mod dynamixel;

#[cfg(feature="serialport")]
mod serial_impl;

/// The generic servo trait
///
/// Allow using servos without knowing the exact type.
pub trait Servo<I: Interface> {

    /// Enable/Disable torque on the servo.
    fn set_enable_torque(&mut self, interface: &mut I, enable_torque: bool) -> Result<(), ::Error>;

    /// Set the goal position. Some servos will be put into "position control mode" when this method is called.
    ///
    /// value is in unit: radians
    fn set_position(&mut self, interface: &mut I, value: f32) -> Result<(), ::Error>;
    
    /// Get the current position.
    ///
    /// The result is returned in unit: radians 
    fn get_position(&mut self, interface: &mut I) -> Result<f32, ::Error>;
}


/// An Error originating from an `Interface`
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CommunicationError {
    TimedOut,
    UnsupportedBaud(BaudRate),
    Other,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    Unfinished,
    
    /// An Error originating from the `Interface`
    Communication(CommunicationError),
    Format,
    Processing,
}

/// Baud rates the interface should support
///
/// May be extended and must not be matched against exhaustively.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BaudRate {
    /// Baud rate of 9600
    Baud9600,
    
    /// Baudaud rate of 19 200
    Baud19200,
    
    /// Baudaud rate of 57 600
    Baud57600,
    
    /// Baudaud rate of 115 200
    Baud115200,
    
    /// Baudaud rate of 200 000
    Baud200000,
    
    /// Baudaud rate of 250 000
    Baud250000,
    
    /// Baudaud rate of 400 000
    Baud400000,
    
    /// Baudaud rate of 500 000
    Baud500000,
    
    /// Baudaud rate of 1 000 000
    Baud1000000,
    
    /// Baudaud rate of 2 000 000
    Baud2000000,
    
    /// Baudaud rate of 3 000 000
    Baud3000000,
    
    /// Baudaud rate of 4 000 000
    Baud4000000,
    
    /// Baudaud rate of 4 500 000
    Baud4500000,
    
    /// Baudaud rate of 10 500 000
    Baud10500000,       
}

impl From<BaudRate> for u32 {
    fn from(b: BaudRate) -> u32 {
        match b {
            BaudRate::Baud9600 => 9600,
            BaudRate::Baud19200 => 19200,
            BaudRate::Baud57600 => 57600,
            BaudRate::Baud115200 => 115200,
            BaudRate::Baud200000 => 200_000,
            BaudRate::Baud250000 => 250_000,
            BaudRate::Baud400000 => 400_000,
            BaudRate::Baud500000 => 500_000,
            BaudRate::Baud1000000 => 1_000_000,
            BaudRate::Baud2000000 => 2_000_000,
            BaudRate::Baud3000000 => 3_000_000,
            BaudRate::Baud4000000 => 4_000_000,
            BaudRate::Baud4500000 => 4_500_000,
            BaudRate::Baud10500000 => 10_500_000,       
        }
    }
}

impl BaudRate {
    fn variants() -> &'static [Self] {
        &[BaudRate::Baud9600,
          BaudRate::Baud19200,
          BaudRate::Baud57600,
          BaudRate::Baud115200,
          BaudRate::Baud200000,
          BaudRate::Baud250000,
          BaudRate::Baud400000,
          BaudRate::Baud500000,
          BaudRate::Baud1000000,
          BaudRate::Baud2000000,
          BaudRate::Baud3000000,
          BaudRate::Baud4000000,
          BaudRate::Baud4500000,
          BaudRate::Baud10500000,
        ]
    }
}

/// The interface for communicating with dynamixel servos.
pub trait Interface {
    /// Set the baud rate of the interface
    ///
    /// `BaudRate` must not be matched against exhaustively.
    fn set_baud_rate(&mut self, b: BaudRate) -> Result<(), CommunicationError>;

    /// Flush out the read buffer
    ///
    /// Whenever a new transmission is started, old data from the read buffer needs to be flushed out first.
    fn flush(&mut self);
    
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

#[cfg(feature="std")]
impl From<std::io::Error> for CommunicationError {
    fn from(e: std::io::Error) -> CommunicationError {
        match e.kind() {
            std::io::ErrorKind::TimedOut => CommunicationError::TimedOut,
            _ => CommunicationError::Other,
        }
    }
}

/// All information needed to connect to a servo
#[derive(Debug, Clone)]
pub enum ServoInfo {
    Protocol1(protocol1::ServoInfo),
    Protocol2(protocol2::ServoInfo),
}

/// Enumerate all servos connected to the interface
///
/// This functions returns a Vec and thus requires the `std` feature.
#[cfg(feature="std")]
pub fn enumerate<I: ::Interface>(interface: &mut I) -> Result<Vec<ServoInfo>, CommunicationError> {
    let mut servos = Vec::new();

    let servos_protocol1 = protocol1::enumerate(interface)?;
    let servos_protocol2 = protocol2::enumerate(interface)?;

    servos.append(&mut servos_protocol1.into_iter().map(|x| ServoInfo::Protocol1(x)).collect());
    servos.append(&mut servos_protocol2.into_iter().map(|x| ServoInfo::Protocol2(x)).collect());

    Ok(servos)
}

/// Connect genericly to a servo
///
/// Only offers basic functionality. If you need more functionality use the connect method of the correct servo type instead.
/// This functions returns a Boxed trait and this requires the `std` feature.
#[cfg(feature="std")]
pub fn connect<I: Interface + 'static>(interface: &mut I, info: ServoInfo) -> Result<Box<Servo<I>>, CommunicationError> {
    match info {
        ServoInfo::Protocol1(si) => protocol1::connect(interface, si),
        ServoInfo::Protocol2(si) => protocol2::connect(interface, si),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
