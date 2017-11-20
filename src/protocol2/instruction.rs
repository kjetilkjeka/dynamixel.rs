use protocol2::{
    Instruction,
    WriteRegister,
    ReadRegister,
};

pub struct Ping ();

impl Instruction for Ping {
    const INSTRUCTION_VALUE: u8 = 0x01;
}

pub struct Read<T: ReadRegister> {
    data: T::DATA,
}

impl<T: ReadRegister> Instruction for Read<T> {
    const INSTRUCTION_VALUE: u8 = 0x02;
}

pub struct Write<T: WriteRegister> {
    data: T::DATA,
}

impl<T: WriteRegister> Instruction for Write<T> {
    const INSTRUCTION_VALUE: u8 = 0x03;
}

pub struct FactoryReset();

impl Instruction for FactoryReset {
    const INSTRUCTION_VALUE: u8 = 0x06;
}

pub struct Reboot();

impl Instruction for Reboot {
    const INSTRUCTION_VALUE: u8 = 0x08;
}
