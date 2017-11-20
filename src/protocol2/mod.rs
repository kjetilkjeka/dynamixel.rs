pub mod instruction;

pub trait WriteRegister {
    type DATA;
}

pub trait ReadRegister {
    type DATA;
}

pub trait Instruction {
    const INSTRUCTION_VALUE: u8;
}
