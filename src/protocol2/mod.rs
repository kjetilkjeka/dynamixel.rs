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
