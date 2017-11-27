pub mod instruction;
mod crc;

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
    
    fn deserialize(Self::Array) -> Result<Self, ()>
        where Self : Sized {
        unimplemented!()
    }
    
    fn error(&self) -> Option<Error>;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    ResultFail = 0x01,
    InstructionError = 0x02,
    CRCError = 0x03,
    DataRangeError = 0x04,
    DataLengthError = 0x05,
    DataLimitError = 0x06,
    AccessError = 0x07,
}

impl Error {
    fn decode(e: u8) -> Option<Error> {
        match e {
            0x01 => Some(Error::ResultFail),
            0x02 => Some(Error::InstructionError),
            0x03 => Some(Error::CRCError),
            0x04 => Some(Error::DataRangeError),
            0x05 => Some(Error::DataLengthError),
            0x06 => Some(Error::DataLimitError),
            0x07 => Some(Error::AccessError),
            _ => None,
        }
    }
}

impl From<Error> for u8 {
    fn from(e: Error) -> u8 {
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
