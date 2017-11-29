use bit_field::BitField;

#[macro_use]
mod control_table;
pub(crate) mod instruction;
mod checksum;

macro_rules! protocol1_servo {
    ($name:ident, $write:path, $read:path) => {
        pub struct $name<T: ::Interface> {
            interface: T,
            id: ::protocol1::ServoID,
        }
        
        impl<T: ::Interface> $name<T> {
            pub fn new(interface: T, id: ::protocol1::ServoID) -> Self {
                $name{
                    interface: interface,
                    id: id,
                }
            }
            
            fn read_response(&mut self, data: &mut [u8]) -> Result<usize, ::Error> {
                // first read header
                self.interface.read(&mut data[..4])?;

                // then read rest of message depending on header length
                let length = data[3] as usize;
                self.interface.read(&mut data[4..4+length])?;
                Ok(4+length)
            }
            
            pub fn ping(&mut self) -> Result<::protocol1::instruction::Pong, ::protocol1::Error> {
                let ping = ::protocol1::instruction::Ping::new(::protocol1::PacketID::from(self.id));
                self.interface.write(&::protocol1::Instruction::serialize(&ping))?;
                let mut received_data = [0u8; 14];
                self.read_response(&mut received_data)?;
                <::protocol1::instruction::Pong as ::protocol1::Status>::deserialize(&received_data)
            }
            
            pub fn write_data<W: $write>(&mut self, register: W) -> Result<(), ::protocol1::Error> {
                let write = ::protocol1::instruction::WriteData::new(::protocol1::PacketID::from(self.id), register);
                self.interface.write(&::protocol1::Instruction::serialize(&write)[0..<::protocol1::instruction::WriteData<W> as ::protocol1::Instruction>::LENGTH as usize + 4])?;
                let mut received_data = [0u8; 11];
                let length = self.read_response(&mut received_data)?;
                match <::protocol1::instruction::WriteDataResponse as ::protocol1::Status>::deserialize(&received_data[0..length]) {
                    Ok(::protocol1::instruction::WriteDataResponse{}) => Ok(()),
                    Err(e) => Err(e),
                }
            }
        }
    };
}


pub trait Register {
    const SIZE: u8;
    const ADDRESS: u8;
}
    
pub trait ReadRegister: Register {
    fn deserialize(&[u8]) -> Self;
}

pub trait WriteRegister: Register {
    // TODO: change 4 to Self::SIZE when const generics land
    fn serialize(&self) -> [u8; 4];
}

pub trait Instruction {
    // The array type is no longer needed when const generics land
    // replace with [u8; Self::LENGTH]
    type Array;
    const LENGTH: u8;
    const INSTRUCTION_VALUE: u8;

    // Serialize can be implemented generically once const generics land
    fn serialize(&self) -> Self::Array;
}

pub trait Status {
    const LENGTH: u8;

    fn deserialize_parameters(parameters: &[u8]) -> Self;
    
    fn deserialize(data: &[u8]) -> Result<Self, Error>
        where Self: Sized {
        // check for formating error stuff
        
        // check for processing errors
        if let Some(error) = ProcessingError::decode(data[4]).map_err(|()| Error::Format(FormatError::InvalidError))? {
            return Err(Error::Processing(error));
        }
        
        let length = data[3];
        if length != Self::LENGTH {
            return Err(Error::Format(FormatError::Length));
        }
        
        let parameters_range = 4..(4 + Self::LENGTH as usize - 2);
        Ok( Self::deserialize_parameters(&data[parameters_range]) )
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    Timeout,
    Format(FormatError),
    Processing(ProcessingError),
}

impl From<::Error> for Error {
    fn from(e: ::Error) -> Error {
        match e {
            ::Error::Timeout => Error::Timeout
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FormatError {
    ID,
    Header,
    CRC,
    Length,
    InvalidError,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct ProcessingError(u8);

impl From<ProcessingError> for u8 {
    fn from(e: ProcessingError) -> u8 {
        e.0
    }
}

impl ProcessingError {
    pub fn decode(v: u8) -> Result<Option<Self>, ()> {
        if v == 0 {
            Ok(None)
        } else if v.get_bit(7) {
            Err(())
        } else {
            Ok(Some(ProcessingError(v)))
        }
    }
    
    pub fn instruction_error(&self) -> bool {
        self.0.get_bit(6)
    }

    pub fn overload_error(&self) -> bool {
        self.0.get_bit(5)
    }

    pub fn checksum_error(&self) -> bool {
        self.0.get_bit(4)
    }

    pub fn range_error(&self) -> bool {
        self.0.get_bit(3)
    }

    pub fn overheating_error(&self) -> bool {
        self.0.get_bit(3)
    }

    pub fn angle_limit_error(&self) -> bool {
        self.0.get_bit(1)
    }

    pub fn input_voltage_error(&self) -> bool {
        self.0.get_bit(0)
    }
}

impl ::lib::fmt::Debug for ProcessingError {
    fn fmt(&self, f: &mut ::lib::fmt::Formatter) -> ::lib::fmt::Result {
        write!(f, "The current ProcessingError, {:?}, decodes to the following errors: [", self.0)?;
        if self.instruction_error() {write!(f, "instruction_error")?;}
        if self.overload_error() {write!(f, "overload_error")?;}
        if self.checksum_error() {write!(f, "checksum_error")?;}
        if self.range_error() {write!(f, "range_error")?;}
        if self.overheating_error() {write!(f, "overheating_error")?;}
        if self.angle_limit_error() {write!(f, "angle_limit_error")?;}
        if self.input_voltage_error() {write!(f, "input_voltage_error")?;}
        write!(f, "]")
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct ServoID(u8);

impl ServoID {
    pub fn new(id: u8) -> ServoID {
        assert!(id <= 253);
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
        assert!(id <= 253);
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
