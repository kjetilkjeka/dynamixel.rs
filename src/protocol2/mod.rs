pub(crate) mod instruction;
#[macro_use]
mod control_table;
mod crc;
mod bit_stuffer;

use Interface;
use BaudRate;
use CommunicationError;

use bit_field::BitField;
use self::bit_stuffer::BitStuffer;

/// Write the instruction on the interface
pub(crate) fn write_instruction<I: ::Interface, T: Instruction>(interface: &mut I, instruction: T) -> Result<(), CommunicationError> {
    for b in instruction.serialize() {
        interface.write(&[b])?
    }
    Ok(())
}

/// Read a status from the interface
///
/// If no instructions have been sent, there will not be any status to read
pub(crate) fn read_status<I: ::Interface, T: Status>(interface: &mut I) -> Result<T, Error> {
    let mut header = [0u8; 9];
    interface.read(&mut header)?;
    
    let mut deserializer = Deserializer::<T>::new()
        .deserialize_header(header)?;

    let mut body = [0u8; 10];

    loop {
        let remaining_bytes = deserializer.remaining_bytes() as usize;
        if remaining_bytes > 10 {
            interface.read(&mut body)?;
            deserializer.deserialize(&body)?;
        } else {
            interface.read(&mut body[..remaining_bytes])?;
            deserializer.deserialize(&body[..remaining_bytes])?;
            break;
        }
    }
    
    Ok(deserializer.build()?)
}

/// Enumerate all protocol 2 servos connected to the interface
#[cfg(feature="std")]
pub fn enumerate<I: ::Interface>(interface: &mut I) -> Result<Vec<ServoInfo>, CommunicationError> {
    let mut servos = Vec::new();

    for b in BaudRate::variants() {

        if let Err(e) = interface.set_baud_rate(*b) {
            warn!(target: "protocol2", "not able to enumerate devices on baudrate: {}", u32::from(*b));
        }

        interface.flush();
        let ping = ::protocol2::instruction::Ping::new(::protocol2::PacketID::Broadcast);
        write_instruction(interface, ping)?;

        loop {
            match read_status::<I, instruction::Pong>(interface) {
                Ok(pong) => servos.push(
                    ServoInfo{
                        baud_rate: *b,
                        model_number: pong.model_number,
                        fw_version: pong.fw_version,
                        id: pong.id,
                    }
                ),
                Err(Error::Communication(CommunicationError::TimedOut)) => break,
                Err(e) => {
                    warn!(target: "protocol2", "received error: {:?} when waiting for enumeration on baud: {}", e, u32::from(*b));
                    break;
                },
            };
        }   
    }
    Ok(servos)
}

#[cfg(feature="std")]
pub fn connect<I: Interface + 'static>(interface: &mut I, info: ServoInfo) -> Result<Box<::Servo<I>>, CommunicationError>{
    match info.model_number {
        ::pro::M4210S260R::<I>::MODEL_NUMBER => Ok(Box::new(::pro::M4210S260R::<I>::new(info.id, info.baud_rate))),
        _ => unimplemented!(),
    }
}

macro_rules! protocol2_servo {
    ($name:ident, $write:path, $read:path, $model_number:expr) => {
        pub struct $name<I: ::Interface> {
            id: ::protocol2::ServoID,
            baudrate: ::BaudRate,
            interface: ::lib::marker::PhantomData<I>,
        }

        impl<I: ::Interface> $name<I> {
            pub const MODEL_NUMBER: u16 = $model_number;

            /// Create a new servo without `ping`ing or taking any other measure to make sure it exists.
            pub fn new(id: ::protocol2::ServoID, baudrate: ::BaudRate) -> Self {
                $name{
                    id: id,
                    baudrate: baudrate,
                    interface: ::lib::marker::PhantomData{},
                }
            }
            
            /// Ping the servo, returning `Ok(ServoInfo)` if it exists.
            pub fn ping(&mut self, interface: &mut I) -> Result<::protocol2::ServoInfo, ::protocol2::Error> {
                interface.set_baud_rate(self.baudrate)?;
                interface.flush();
                
                let ping = ::protocol2::instruction::Ping::new(::protocol2::PacketID::from(self.id));
                ::protocol2::write_instruction(interface, ping)?;
                let pong = ::protocol2::read_status::<I, ::protocol2::instruction::Pong>(interface)?;
                Ok(
                    ::protocol2::ServoInfo{
                        baud_rate: self.baudrate,
                        model_number: pong.model_number,
                        fw_version: pong.fw_version,
                        id: pong.id,
                    }
                )
            }

            /// Write the given data `register` to the servo.
            pub fn write<W: $write>(&mut self, interface: &mut I, register: W) -> Result<(), ::protocol2::Error> {
                interface.set_baud_rate(self.baudrate)?;
                let write = ::protocol2::instruction::Write::new(::protocol2::PacketID::from(self.id), register);
                ::protocol2::write_instruction(interface, write)?;
                ::protocol2::read_status::<I, ::protocol2::instruction::WriteResponse>(interface)?;
                Ok(())
            }

            /// Read data from a register
            pub fn read<R: $read>(&mut self, interface: &mut I) -> Result<R, ::protocol2::Error> {
                interface.set_baud_rate(self.baudrate)?;
                interface.flush();
                
                let read = ::protocol2::instruction::Read::<R>::new(::protocol2::PacketID::from(self.id));
                ::protocol2::write_instruction(interface, read)?;
                Ok(::protocol2::read_status::<I, ::protocol2::instruction::ReadResponse<R>>(interface)?.value)
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

pub(crate) trait Instruction {
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

pub(crate) trait Status {
    const PARAMETERS: u16;

    fn deserialize(id: ServoID, parameters: &[u8]) -> Self;
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct Serializer<'a, T: Instruction + 'a> {
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
pub(crate) enum DeserializationStatus {
    Ok,
    Finished,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct Deserializer<T: Status> {
    phantom: ::lib::marker::PhantomData<T>,
}

impl<T: Status> Deserializer<T> {
    fn new() -> Self {
        Deserializer {
            phantom: ::lib::marker::PhantomData{},
        }
    }
    
    fn deserialize_header(self, data: [u8; 9]) -> Result<BodyDeserializer<T>, FormatError> {
        if data[0] != 0xff {return Err(FormatError::Header)};
        if data[1] != 0xff {return Err(FormatError::Header)};
        if data[2] != 0xfd {return Err(FormatError::Header)};
        if data[3] != 0x00 {return Err(FormatError::Header)};
        if data[7] != 0x55 {return Err(FormatError::Instruction)};

        let length = data[5] as u16 | (data[6] as u16) << 8;
        
        let mut crc = crc::CRC::new();
        crc.add(&data);
        
        let mut bit_stuffer = BitStuffer::new();
        for b in data.iter() {
            bit_stuffer = bit_stuffer.add_byte(*b)?;
        }
        
        Ok(BodyDeserializer {
            parameter_index: 0,
            remaining_bytes: length-2,
            id: ServoID::new(data[4]),
            crc_l: None,
            crc_calc: crc,
            bit_stuffer: bit_stuffer,
            alert: data[8].get_bit(7),
            processing_error: ProcessingError::decode(data[8].get_bits(0..7))?,
            parameters: [0u8; 6],
            phantom: ::lib::marker::PhantomData{},
        })
    }
}
    
#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct BodyDeserializer<T: Status> {
    remaining_bytes: u16,
    parameter_index: usize,
    id: ServoID,
    crc_l: Option<u8>,
    crc_calc: crc::CRC,
    bit_stuffer: BitStuffer,
    alert: bool,
    processing_error: Option<ProcessingError>,
    parameters: [u8; 6],
    phantom: ::lib::marker::PhantomData<T>,
}

impl<T: Status> BodyDeserializer<T> {

    pub fn is_finished(&self) -> bool {
        self.remaining_bytes == 0
    }
    
    pub fn remaining_bytes(&self) -> u16 {
        self.remaining_bytes
    }
    
    pub fn build(self) -> Result<T, Error> {
        if !self.is_finished() {
            Err(Error::Unfinished)
        } else if let Some(error) = self.processing_error {
            Err(Error::Processing(error))
        } else {
            Ok(T::deserialize(self.id, &self.parameters[..T::PARAMETERS as usize]))
        }
    }
    
    pub fn deserialize(&mut self, data: &[u8]) -> Result<DeserializationStatus, FormatError> {
        for b in data {
            if self.bit_stuffer.stuff_next() && self.remaining_bytes > 2 {
                self.bit_stuffer = self.bit_stuffer.add_byte(*b)?;
                self.remaining_bytes -= 1;
            } else if self.remaining_bytes > 2 {
                self.bit_stuffer = self.bit_stuffer.add_byte(*b)?;
                self.crc_calc.add(&[*b]);
                self.parameters[self.parameter_index] = *b;
                self.parameter_index += 1;
                self.remaining_bytes -= 1;
            } else if self.remaining_bytes == 2 {
                self.crc_l = Some(*b);
                self.remaining_bytes -= 1;
            } else if self.remaining_bytes == 1 {
                let crc = self.crc_l.unwrap() as u16 | (*b as u16) << 8;
                if crc != u16::from(self.crc_calc) {
                    return Err(FormatError::CRC);
                }
                self.remaining_bytes -= 1;
            } else {
                return Err(FormatError::Length);
            }
        }
        
        if self.remaining_bytes == 0 {
            Ok(DeserializationStatus::Finished)
        } else {
            Ok(DeserializationStatus::Ok)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServoInfo {
    pub baud_rate: ::BaudRate,
    pub model_number: u16,
    pub fw_version: u8,
    pub id: ServoID,
}


impl From<::CommunicationError> for Error {
    fn from(e: ::CommunicationError) -> Error {
        Error::Communication(e)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    Unfinished,
    Communication(::CommunicationError),
    Format(FormatError),
    Processing(ProcessingError),
}

impl From<::protocol2::Error> for ::Error {
    fn from(e: ::protocol2::Error) -> ::Error {
        match e {
            ::protocol2::Error::Unfinished => ::Error::Unfinished,
            ::protocol2::Error::Communication(ce) => ::Error::Communication(ce),
            ::protocol2::Error::Format(_) => ::Error::Format,
            ::protocol2::Error::Processing(_) => ::Error::Processing,
        }
    }
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
