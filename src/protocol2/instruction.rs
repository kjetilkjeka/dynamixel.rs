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
    const LENGTH: u16 = 10;
    const INSTRUCTION_VALUE: u8 = 0x01;
}

pub struct Read<T: ReadRegister> {
    id: PacketID,
    data: T,
}

impl<T: ReadRegister> Instruction for Read<T> {
    type Array = [u8; 14];
    const LENGTH: u16 = 14;
    const INSTRUCTION_VALUE: u8 = 0x02;
}

pub struct Write<T: WriteRegister> {
    id: PacketID,
    data: T,
}

impl<T: WriteRegister> Instruction for Write<T>{
    // Use max size (4) untill const generics land
    type Array = [u8; 16];
    const LENGTH: u16 = 12 + T::SIZE;
    const INSTRUCTION_VALUE: u8 = 0x03;
}

pub struct FactoryReset {
    id: PacketID,
}

impl Instruction for FactoryReset {
    type Array = [u8; 11];
    const LENGTH: u16 = 11;
    const INSTRUCTION_VALUE: u8 = 0x06;
}

pub struct Reboot {
    id: PacketID,
}

impl Instruction for Reboot {
    type Array = [u8; 10];
    const INSTRUCTION_VALUE: u8 = 0x08;

    fn serialize(&self) -> Self::Array {unimplemented!()}
}
