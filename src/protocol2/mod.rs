pub(crate) mod instruction;
#[macro_use]
mod control_table;
mod crc;
mod bit_stuffer;

use self::bit_stuffer::BitStuffer;

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
                for b in ::protocol2::Instruction::serialize(&ping) {
                    self.interface.write(&[b])?;
                }
                let mut received_data = [0u8; 14];
                self.read_response(&mut received_data)?;
                <::protocol2::instruction::Pong as ::protocol2::Status>::deserialize(&received_data)
            }
            
            pub fn write<W: $write>(&mut self, register: W) -> Result<(), ::protocol2::Error> {
                let write = ::protocol2::instruction::Write::new(::protocol2::PacketID::from(self.id), register);
                for b in ::protocol2::Instruction::serialize(&write) {
                    self.interface.write(&[b])?;
                }
                let mut received_data = [0u8; 11];
                self.read_response(&mut received_data)?;
                match <::protocol2::instruction::WriteResponse as ::protocol2::Status>::deserialize(&received_data) {
                    Ok(::protocol2::instruction::WriteResponse{}) => Ok(()),
                    Err(e) => Err(e),
                }
            }

            pub fn read<R: $read>(&mut self) -> Result<R, ::protocol2::Error> {
                let read = ::protocol2::instruction::Read::<R>::new(::protocol2::PacketID::from(self.id));
                for b in ::protocol2::Instruction::serialize(&read) {
                    self.interface.write(&[b])?;
                }
                let mut received_data = [0u8; 15];
                self.read_response(&mut received_data)?;
                match <::protocol2::instruction::ReadResponse<R> as ::protocol2::Status>::deserialize(&received_data) {
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
    
pub trait ReadRegister: Register {
    fn deserialize(&[u8]) -> Self;
}

pub trait WriteRegister: Register {
    // TODO: change 4 to Self::SIZE when const generics land
    fn serialize(&self) -> [u8; 4];
}

pub trait Instruction {
    const PARAMETERS: u16;
    const INSTRUCTION_VALUE: u8;

    fn id(&self) -> PacketID;
    
    fn parameter(&self, index: usize) -> u8;

    fn serialize<'a>(&'a self) -> Serializer<'a, Self> where Self: Sized {
        let serializer = Serializer{
            pos: 0,
            length: 10 + Self::PARAMETERS,
            crc: crc::CRC::new(),
            bit_stuffer: BitStuffer::new(),
            instruction: self,
        };

        let mut length = 0;
        for _b in serializer.skip(7) {
            length += 1;
        }

        Serializer{
            pos: 0,
            length: length,
            crc: crc::CRC::new(),
            bit_stuffer: BitStuffer::new(),
            instruction: self,
        }
    }
}

pub trait Status {
    const LENGTH: u16;

    fn deserialize_parameters(parameters: &[u8]) -> Self;
    
    fn deserialize(data: &[u8]) -> Result<Self, Error>
        where Self: Sized {
        // check for formating error stuff
        
        // check for processing errors
        if let Some(error) = ProcessingError::decode(data[8]).map_err(|()| Error::Format(FormatError::InvalidError))? {
            return Err(Error::Processing(error));
        }

        let length = data[5] as u16 | ((data[6] as u16) << 8);
        if length != Self::LENGTH {
            return Err(Error::Format(FormatError::Length));
        }
        
        let parameters_range = 9..(9 + Self::LENGTH as usize - 4);
        Ok( Self::deserialize_parameters(&data[parameters_range]) )
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Serializer<'a, T: Instruction + 'a> {
    pos: usize,
    length: u16,
    crc: crc::CRC,
    bit_stuffer: BitStuffer,
    instruction: &'a T,
}

impl<'a, T: Instruction + 'a> ::lib::iter::Iterator for Serializer<'a, T> {
    type Item = u8;
    
    fn next(&mut self) -> Option<u8> {
        let should_stuff = self.bit_stuffer.stuff_next() && self.pos < 9+T::PARAMETERS as usize;
        let next_byte = if should_stuff {
            Some(0xfd)
        } else {
            let next_byte = match self.pos {
                0 => Some(0xff),
                1 => Some(0xff),
                2 => Some(0xfd),
                3 => Some(0x00),
                4 => Some(u8::from(self.instruction.id())),
                5 => Some(self.length as u8),
                6 => Some((self.length >> 8) as u8),
                7 => Some(T::INSTRUCTION_VALUE),
                x if x < 8+T::PARAMETERS as usize => Some(self.instruction.parameter(x-8)),
                x if x == 8+T::PARAMETERS as usize => Some(u16::from(self.crc) as u8),
                x if x == 9+T::PARAMETERS as usize => Some((u16::from(self.crc) >> 8) as u8),
                _ => None,
            };
            

            next_byte
        };

        if self.pos < 8+T::PARAMETERS as usize {
            self.bit_stuffer = self.bit_stuffer.add_byte(next_byte.unwrap()).unwrap();
            self.crc.add(&[next_byte.unwrap()]);
        }

        if next_byte.is_some() && !should_stuff {
            self.pos += 1;
        }
        
        next_byte
    }
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
    InvalidError,
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
    fn decode(e: u8) -> Result<Option<ProcessingError>, ()> {
        match e {
            0x00 => Ok(None),
            0x01 => Ok(Some(ProcessingError::ResultFail)),
            0x02 => Ok(Some(ProcessingError::InstructionError)),
            0x03 => Ok(Some(ProcessingError::CRCError)),
            0x04 => Ok(Some(ProcessingError::DataRangeError)),
            0x05 => Ok(Some(ProcessingError::DataLengthError)),
            0x06 => Ok(Some(ProcessingError::DataLimitError)),
            0x07 => Ok(Some(ProcessingError::AccessError)),
            _ => Err(()),
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
