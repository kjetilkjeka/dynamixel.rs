pub mod instruction;
mod crc;

use Interface;

pub trait Servo<T: Interface> {
    fn interface(&mut self) -> &mut T;
    
    fn ping(&mut self, id: PacketID) -> Result<instruction::Pong, Error> {
        let ping = instruction::Ping::new(id);
        self.interface().write(&ping.serialize());
        let mut received_data = [0u8; 14];
        // TODO: timeout checking
        self.interface().read(&mut received_data);
        instruction::Pong::deserialize(received_data)
    }

    fn write<W: WriteRegister>(&mut self, id: PacketID, register: W) -> Result<instruction::WriteResponse, Error> {
        let write = instruction::Write::new(id, register);
        self.interface().write(&write.serialize()[0..instruction::Write::<W>::LENGTH as usize + 7]);
        let mut received_data = [0u8; 11];
        // TODO: timeout checking
        self.interface().read(&mut received_data);
        instruction::WriteResponse::deserialize(received_data)
    }
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
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
pub struct PacketID(u8);

impl PacketID {
    pub fn unicast(id: u8) -> PacketID {
        assert!(id <= 252);
        PacketID(id)
    }

    pub fn broadcast() -> PacketID {
        PacketID(254)
    }
}

impl From<PacketID> for u8 {
    fn from(id: PacketID) -> u8 {
        id.0
    }
}
