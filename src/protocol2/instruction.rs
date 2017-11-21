use protocol2::{
    Register,
    PacketID,
    ReadRegister,
    WriteRegister,
    Instruction,
};

pub struct Ping {
    id: PacketID,
}

impl Instruction for Ping {
    type Array = [u8; 10];
    const INSTRUCTION_VALUE: u8 = 0x01;

    fn serialize(&self) -> Self::Array {unimplemented!()}
}

pub struct Read<T: ReadRegister> {
    id: PacketID,
    data: T,
}

impl<T: ReadRegister> Instruction for Read<T> {
    type Array = [u8; 14];
    const INSTRUCTION_VALUE: u8 = 0x02;

    fn serialize(&self) -> Self::Array {unimplemented!()}
}

pub struct Write<T: WriteRegister> {
    id: PacketID,
    data: T,
}

impl<T: WriteRegister> Instruction for Write<T>{
    // Untill const generics land, use 4 bytes
    //type Array = [u8; 12 + T::Sized];
    type Array = [u8; 12 + 4];
    const INSTRUCTION_VALUE: u8 = 0x03;

    fn serialize(&self) -> Self::Array {unimplemented!()}
}

pub struct FactoryReset {
    id: PacketID,
}

impl Instruction for FactoryReset {
    type Array = [u8; 11];
    const INSTRUCTION_VALUE: u8 = 0x06;

    fn serialize(&self) -> Self::Array {unimplemented!()}
}

pub struct Reboot {
    id: PacketID,
}

impl Instruction for Reboot {
    type Array = [u8; 10];
    const INSTRUCTION_VALUE: u8 = 0x08;

    fn serialize(&self) -> Self::Array {unimplemented!()}
}
