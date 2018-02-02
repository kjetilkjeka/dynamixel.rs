use std;
use serialport;

use std::ops::DerefMut;

use {
    CommunicationError,
    Interface,
    BaudRate,
};

impl From<serialport::Error> for CommunicationError {
    fn from(_err: serialport::Error) -> CommunicationError {
        CommunicationError::Other
    }
}

impl From<BaudRate> for serialport::BaudRate {
    fn from(b: BaudRate) -> serialport::BaudRate {
        match b {
            BaudRate::Baud9600 => serialport::BaudRate::Baud9600,
            BaudRate::Baud19200 => serialport::BaudRate::Baud19200,
            BaudRate::Baud57600 => serialport::BaudRate::Baud57600,
            BaudRate::Baud115200 => serialport::BaudRate::Baud115200,
            BaudRate::Baud500000 => serialport::BaudRate::Baud500000,
            BaudRate::Baud1000000 => serialport::BaudRate::Baud1000000,
            BaudRate::Baud2000000 => serialport::BaudRate::Baud2000000,
            BaudRate::Baud3000000 => serialport::BaudRate::Baud3000000,
            BaudRate::Baud4000000 => serialport::BaudRate::Baud4000000,
            b => serialport::BaudRate::BaudOther(u32::from(b)),
        }
    }
}

impl Interface for std::boxed::Box<serialport::SerialPort> {
    fn set_baud_rate(&mut self, b: BaudRate) -> Result<(), CommunicationError> {
        match serialport::SerialPort::set_baud_rate(self.deref_mut(), serialport::BaudRate::from(b)) {
            Ok(_) => Ok(()),
            Err(_) => Err(CommunicationError::UnsupportedBaud(b)),
        }
    }
    
    fn read(&mut self, data: &mut [u8]) -> Result<(), CommunicationError> {
        self.set_timeout(std::time::Duration::new(0, 1000000000))?;
        Ok(std::io::Read::read_exact(self, data)?)
    }

    fn write(&mut self, data: &[u8]) -> Result<(), CommunicationError> {
        self.set_timeout(std::time::Duration::new(0, 1000000000))?;
        Ok(std::io::Write::write_all(self, data)?)
    }
}
