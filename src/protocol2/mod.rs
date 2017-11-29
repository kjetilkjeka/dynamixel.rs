pub mod instruction;
mod crc;

use Interface;

macro_rules! protocol2_servo {
    ($name:ident, $write:path, $read:path) => {
        pub struct $name<T: ::Interface> {
            interface: T,
            id: ::protocol2::ServoID,
        }

        impl<T: ::Interface> $name<T> {
            pub fn new(interface: T, id: ::protocol2::ServoID) -> Self {
                $name{
                    interface: interface,
                    id: id,
                }
            }

            fn read_response(&mut self, data: &mut [u8]) -> Result<(), ::Error> {
                // first read header
                self.interface.read(&mut data[..7])?;

                // then read rest of message depending on header length
                let length = data[5] as usize | ((data[6] as usize) << 8);
                self.interface.read(&mut data[7..7+length])?;
                Ok(())
            }
            
            pub fn ping(&mut self) -> Result<::protocol2::instruction::Pong, ::protocol2::Error> {
                let ping = ::protocol2::instruction::Ping::new(::protocol2::PacketID::from(self.id));
                self.interface.write(&::protocol2::Instruction::serialize(&ping))?;
                let mut received_data = [0u8; 14];
                self.read_response(&mut received_data)?;
                <::protocol2::instruction::Pong as ::protocol2::Status>::deserialize(received_data)
            }
            
            pub fn write<W: $write>(&mut self, register: W) -> Result<(), ::protocol2::Error> {
                let write = ::protocol2::instruction::Write::new(::protocol2::PacketID::from(self.id), register);
                self.interface.write(&::protocol2::Instruction::serialize(&write)[0..<::protocol2::instruction::Write<W> as ::protocol2::Instruction>::LENGTH as usize + 7])?;
                let mut received_data = [0u8; 11];
                self.read_response(&mut received_data)?;
                match <::protocol2::instruction::WriteResponse as ::protocol2::Status>::deserialize(received_data) {
                    Ok(::protocol2::instruction::WriteResponse{}) => Ok(()),
                    Err(e) => Err(e),
                }
            }

            pub fn read<R: $read>(&mut self) -> Result<R, ::protocol2::Error> {
                let write = ::protocol2::instruction::Read::<R>::new(::protocol2::PacketID::from(self.id));
                self.interface.write(&::protocol2::Instruction::serialize(&write))?;
                let mut received_data = [0u8; 15];
                self.read_response(&mut received_data)?;
                match <::protocol2::instruction::ReadResponse<R> as ::protocol2::Status>::deserialize(received_data) {
                    Ok(::protocol2::instruction::ReadResponse{value: v}) => Ok(v),
                    Err(e) => Err(e),
                }
            }
        }
    };
}

pub trait Register {
    const SIZE: u16;
    const ADDRESS: u16;
}
    
pub trait ReadRegister : Register {
    fn deserialize([u8; 4]) -> Self;
}

pub trait WriteRegister : Register {
    fn serialize(&self) -> [u8; 4];
}

pub trait Instruction {
    // The array type is no longer needed when const generics land
    // replace with [u8; Self::LENGTH]
    type Array;
    const LENGTH: u16;
    const INSTRUCTION_VALUE: u8;

    // Serialize can be implemented generically once const generics land
    fn serialize(&self) -> Self::Array { unimplemented!() }
}

pub trait Status {
    // The array type is no longer needed when const generics land
    // replace with [u8; Self::LENGTH]
    type Array;
    const LENGTH: u16;

    fn deserialize(data: Self::Array) -> Result<Self, Error> where Self: Sized;
}

impl From<::Error> for Error {
    fn from(e: ::Error) -> Error {
        match e {
            ::Error::Timeout => Error::Timeout
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    Timeout,
    Format(FormatError),
    Processing(ProcessingError),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FormatError {
    ID,
    Header,
    CRC,
    Length,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ProcessingError {
    ResultFail = 0x01,
    InstructionError = 0x02,
    CRCError = 0x03,
    DataRangeError = 0x04,
    DataLengthError = 0x05,
    DataLimitError = 0x06,
    AccessError = 0x07,
}

impl ProcessingError {
    fn decode(e: u8) -> Option<ProcessingError> {
        match e {
            0x01 => Some(ProcessingError::ResultFail),
            0x02 => Some(ProcessingError::InstructionError),
            0x03 => Some(ProcessingError::CRCError),
            0x04 => Some(ProcessingError::DataRangeError),
            0x05 => Some(ProcessingError::DataLengthError),
            0x06 => Some(ProcessingError::DataLimitError),
            0x07 => Some(ProcessingError::AccessError),
            _ => None,
        }
    }
}

impl From<ProcessingError> for u8 {
    fn from(e: ProcessingError) -> u8 {
        e as u8
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct ServoID(u8);

impl ServoID {
    pub fn new(id: u8) -> ServoID {
        assert!(id <= 252);
        ServoID(id)
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum PacketID {
    Unicast(ServoID),
    Broadcast,
}

impl PacketID {
    pub fn unicast(id: u8) -> PacketID {
        assert!(id <= 252);
        PacketID::Unicast(ServoID::new(id))
    }

    pub fn broadcast() -> PacketID {
        PacketID::Broadcast
    }
}

impl From<ServoID> for PacketID {
    fn from(id: ServoID) -> PacketID {
        PacketID::Unicast(id)
    }
}

impl From<PacketID> for u8 {
    fn from(id: PacketID) -> u8 {
        match id {
            PacketID::Unicast(x) => u8::from(x),
            PacketID::Broadcast => 254,
        }
    }
}

impl From<ServoID> for u8 {
    fn from(id: ServoID) -> u8 {
        id.0
    }
}
