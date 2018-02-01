use std;
use serialport;

use std::ops::DerefMut;

use {
    CommunicationError,
    Interface,
};

impl From<serialport::Error> for CommunicationError {
    fn from(_err: serialport::Error) -> CommunicationError {
        CommunicationError::Other
    }
}


impl Interface for std::boxed::Box<serialport::SerialPort> {
    
    fn read(&mut self, data: &mut [u8]) -> Result<(), CommunicationError> {
        self.set_timeout(std::time::Duration::new(0, 1000000000))?;
        Ok(std::io::Read::read_exact(self, data)?)
    }

    fn write(&mut self, data: &[u8]) -> Result<(), CommunicationError> {
        self.set_timeout(std::time::Duration::new(0, 1000000000))?;
        Ok(std::io::Write::write_all(self, data)?)
    }
}
