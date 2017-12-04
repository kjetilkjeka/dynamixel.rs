pub(crate) mod instruction;
#[macro_use]
mod control_table;
mod crc;
mod bit_stuffer;

use bit_field::BitField;
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
            
            pub fn ping(&mut self) -> Result<::protocol2::instruction::Pong, ::protocol2::Error> {
                let ping = ::protocol2::instruction::Ping::new(::protocol2::PacketID::from(self.id));
                for b in ::protocol2::Instruction::serialize(&ping) {
                    self.interface.write(&[b])?;
                }
                
                let mut deserializer = ::protocol2::Deserializer::<::protocol2::instruction::Pong>::new();
                let mut header_data = [0u8; 7];
                self.interface.read(&mut header_data)?;
                deserializer.deserialize(&mut header_data)?;
                let mut received_data = [0u8];
                while !deserializer.is_finished() {
                    self.interface.read(&mut received_data)?;
                    deserializer.deserialize(&mut received_data)?;
                }
                
                Ok(deserializer.build()?)
            }
            
            pub fn write<W: $write>(&mut self, register: W) -> Result<(), ::protocol2::Error> {
                let write = ::protocol2::instruction::Write::new(::protocol2::PacketID::from(self.id), register);
                for b in ::protocol2::Instruction::serialize(&write) {
                    self.interface.write(&[b])?;
                }

                let mut deserializer = ::protocol2::Deserializer::<::protocol2::instruction::WriteResponse>::new();
                let mut header_data = [0u8; 7];
                self.interface.read(&mut header_data)?;
                deserializer.deserialize(&mut header_data)?;
                let mut received_data = [0u8];
                while !deserializer.is_finished() {
                    self.interface.read(&mut received_data)?;
                    deserializer.deserialize(&mut received_data)?;
                }
                
                deserializer.build()?;
                Ok(())
            }

            pub fn read<R: $read>(&mut self) -> Result<R, ::protocol2::Error> {
                let read = ::protocol2::instruction::Read::<R>::new(::protocol2::PacketID::from(self.id));
                for b in ::protocol2::Instruction::serialize(&read) {
                    self.interface.write(&[b])?;
                }
                
                let mut deserializer = ::protocol2::Deserializer::<::protocol2::instruction::ReadResponse<R>>::new();
                let mut header_data = [0u8; 7];
                self.interface.read(&mut header_data)?;
                deserializer.deserialize(&mut header_data)?;
                let mut received_data = [0u8];
                while !deserializer.is_finished() {
                    self.interface.read(&mut received_data)?;
                    deserializer.deserialize(&mut received_data)?;
                }
                
                Ok(deserializer.build()?.value)
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
    const PARAMETERS: u16;

    fn deserialize_parameters(parameters: &[u8]) -> Self;
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DeserializationStatus {
    Ok,
    Finished,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Deserializer<T: Status> {
    finished: bool,
    pos: usize,
    id: Option<ServoID>,
    length_l: Option<u8>,
    length: Option<u16>,
    crc_l: Option<u8>,
    crc_calc: crc::CRC,
    bit_stuffer: BitStuffer,
    alert: bool,
    processing_error: Option<ProcessingError>,
    parameters: [u8; 6],
    phantom: ::lib::marker::PhantomData<T>,
}

impl<T: Status> Deserializer<T> {

    pub fn new() -> Deserializer<T>  {
        Deserializer{
            finished: false,
            pos: 0,
            id: None,
            length_l: None,
            length: None,
            crc_l: None,
            crc_calc: crc::CRC::new(),
            bit_stuffer: BitStuffer::new(),
            alert: false,
            processing_error: None,
            parameters: [0u8; 6],
            phantom: ::lib::marker::PhantomData{},
        }
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }
    
    pub fn build(self) -> Result<T, Error> {
        if !self.is_finished() {
            Err(Error::Unfinished)
        } else if let Some(error) = self.processing_error {
            Err(Error::Processing(error))
        } else {
            Ok(T::deserialize_parameters(&self.parameters[..T::PARAMETERS as usize]))
        }
    }
    
    pub fn deserialize(&mut self, data: &[u8]) -> Result<DeserializationStatus, FormatError> {
        for b in data {
            if self.finished {
                return Err(FormatError::Length);
            } else if self.bit_stuffer.stuff_next() && self.pos <= 8+T::PARAMETERS as usize {
                self.bit_stuffer = self.bit_stuffer.add_byte(*b)?;
            } else {
                if self.pos <= 8+T::PARAMETERS as usize {
                    self.bit_stuffer = self.bit_stuffer.add_byte(*b)?;
                    self.crc_calc.add(&[*b]);
                }
                
                match self.pos {
                    0 => if *b != 0xff {return Err(FormatError::Header)},
                    1 => if *b != 0xff {return Err(FormatError::Header)},
                    2 => if *b != 0xfd {return Err(FormatError::Header)},
                    3 => if *b != 0x00 {return Err(FormatError::Header)},
                    4 => self.id = Some(ServoID::new(*b)),
                    5 => self.length_l = Some(*b),
                    6 => self.length = Some( (self.length_l.unwrap() as u16) | ((*b as u16) << 8) ),
                    7 => if *b != 0x55 {return Err(FormatError::Instruction)},
                    8 => {
                        self.alert = b.get_bit(7);
                        self.processing_error = ProcessingError::decode(b.get_bits(0..7))?;
                    },
                    x if x <= 6 + self.length.unwrap() as usize - 2 => self.parameters[x - 9] = *b,
                    x if x == 6 + self.length.unwrap() as usize - 1 => self.crc_l = Some(*b),
                    x if x == 6 + self.length.unwrap() as usize - 0 => {
                        let crc = self.crc_l.unwrap() as u16 | (*b as u16) << 8;
                        if crc != u16::from(self.crc_calc) {
                            return Err(FormatError::CRC);
                        }
                    },
                    _ => return Err(FormatError::Length),
                };
                
                self.pos += 1;
            }
            
            
            if self.length.is_some() && self.length.unwrap() + 7 == self.pos as u16 {
                self.finished = true;
            }
        }

        if self.finished {
            Ok(DeserializationStatus::Finished)
        } else {
            Ok(DeserializationStatus::Ok)
        }
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
    Unfinished,
    Format(FormatError),
    Processing(ProcessingError),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FormatError {
    Header,
    ID,
    Length,
    Instruction,
    InvalidError(u8),
    CRC,
    StuffByte,
    NotFinished,
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

impl From<FormatError> for Error {
    fn from(e: FormatError) -> Error {
        Error::Format(e)
    }
}

impl ProcessingError {
    fn decode(e: u8) -> Result<Option<ProcessingError>, FormatError> {
        match e {
            0x00 => Ok(None),
            0x01 => Ok(Some(ProcessingError::ResultFail)),
            0x02 => Ok(Some(ProcessingError::InstructionError)),
            0x03 => Ok(Some(ProcessingError::CRCError)),
            0x04 => Ok(Some(ProcessingError::DataRangeError)),
            0x05 => Ok(Some(ProcessingError::DataLengthError)),
            0x06 => Ok(Some(ProcessingError::DataLimitError)),
            0x07 => Ok(Some(ProcessingError::AccessError)),
            x => Err(FormatError::InvalidError(x)),
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
