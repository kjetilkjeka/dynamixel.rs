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
