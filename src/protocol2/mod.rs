pub mod instruction;

pub trait Register {
    const SIZE: u16;
    const ADDRESS: u16;
}
    
pub trait ReadRegister : Register {}
pub trait WriteRegister : Register {}

pub trait Instruction {
    type Array;
    const INSTRUCTION_VALUE: u8;

    fn serialize(&self) -> Self::Array;
}

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
