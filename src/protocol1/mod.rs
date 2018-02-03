use BaudRate;
use CommunicationError;

use bit_field::BitField;

#[macro_use]
mod control_table;
pub(crate) mod instruction;
mod checksum;

/// Enumerate all protocol 1 servos connected to the interface
#[cfg(feature="std")]
pub fn enumerate<I: ::Interface>(interface: &mut I) -> Result<Vec<ServoInfo>, CommunicationError> {
    let mut servos = Vec::new();

    for b in BaudRate::variants() {

        if let Err(_) = interface.set_baud_rate(*b) {
            warn!(target: "protocol1", "not able to enumerate devices on baudrate: {}", u32::from(*b));
        }

        interface.flush();
        let ping = ::protocol1::instruction::Ping::new(PacketID::Broadcast);
        interface.write(&::protocol1::Instruction::serialize(&ping))?;

        loop {
            let mut received_data = [0u8; 14];
            // first read header
            if let Err(e) = interface.read(&mut received_data[..4]) {
                warn!(target: "protocol1", "received error: {:?} when waiting for enumeration on baud: {}", e, u32::from(*b));
            }

            // then read rest of message depending on header length
            let length = received_data[3] as usize;
            interface.read(&mut received_data[4..4+length])?;

            match <::protocol1::instruction::Pong as ::protocol1::Status>::deserialize(&received_data) {
                Ok(pong) => servos.push(
                    ServoInfo{
                        baud_rate: *b,
                        model_number: 0x001D,
                        fw_version: 0,
                        id: pong.id,
                    }
                ),
                Err(Error::Communication(CommunicationError::TimedOut)) => break,
                Err(e) => {
                    warn!(target: "protocol1", "received error: {:?} when waiting for enumeration on baud: {}", e, u32::from(*b));
                    break;
                },
            };
        }   
    }
    Ok(servos)
}

#[cfg(feature="std")]
pub(crate) fn connect<I: ::Interface + 'static>(_interface: &mut I, info: ServoInfo) -> Result<Box<::Servo<I>>, CommunicationError>{
    match info.model_number {
        ::dynamixel::mx28::MX28::<I>::MODEL_NUMBER => Ok(Box::new(::dynamixel::mx28::MX28::<I>::new(info.id, info.baud_rate))),
        _ => unimplemented!(),
    }
}


macro_rules! protocol1_servo {
    ($name:ident, $write:path, $read:path, $model_number:expr) => {
        pub struct $name<I: ::Interface> {
            id: ::protocol1::ServoID,
            baudrate: ::BaudRate,
            interface: ::lib::marker::PhantomData<I>,
        }
        
        impl<I: ::Interface> $name<I> {
            pub const MODEL_NUMBER: u16 = $model_number;

            /// Creates a new servo without `ping`ing or taking any other measure to make sure it exists.
            pub fn new(id: ::protocol1::ServoID, baudrate: ::BaudRate) -> Self {
                $name{
                    id: id,
                    baudrate: baudrate,
                    interface: ::lib::marker::PhantomData{},
                }
            }
            
            fn read_response(&mut self, interface: &mut I, data: &mut [u8]) -> Result<usize, ::CommunicationError> {
                // first read header
                interface.read(&mut data[..4])?;

                // then read rest of message depending on header length
                let length = data[3] as usize;
                interface.read(&mut data[4..4+length])?;
                Ok(4+length)
            }

            /// Ping the servo, returning `Ok(())` if it exists.
            pub fn ping(&mut self, interface: &mut I) -> Result<(), ::protocol1::Error> {
                interface.set_baud_rate(self.baudrate)?;
                interface.flush();

                let ping = ::protocol1::instruction::Ping::new(::protocol1::PacketID::from(self.id));
                interface.write(&::protocol1::Instruction::serialize(&ping))?;
                let mut received_data = [0u8; 14];
                self.read_response(interface, &mut received_data)?;
                <::protocol1::instruction::Pong as ::protocol1::Status>::deserialize(&received_data)?;
                Ok(())
            }
            
            /// Write the given data `register` to the servo.
            pub fn write_data<W: $write>(&mut self, interface: &mut I, register: W) -> Result<(), ::protocol1::Error> {
                interface.set_baud_rate(self.baudrate)?;
                interface.flush();
                
                let write = ::protocol1::instruction::WriteData::new(::protocol1::PacketID::from(self.id), register);
                interface.write(&::protocol1::Instruction::serialize(&write)[0..<::protocol1::instruction::WriteData<W> as ::protocol1::Instruction>::LENGTH as usize + 4])?;
                let mut received_data = [0u8; 11];
                let length = self.read_response(interface, &mut received_data)?;
                match <::protocol1::instruction::WriteDataResponse as ::protocol1::Status>::deserialize(&received_data[0..length]) {
                    Ok(::protocol1::instruction::WriteDataResponse{id: _}) => Ok(()),
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

pub(crate) trait Instruction {
    // The array type is no longer needed when const generics land
    // replace with [u8; Self::LENGTH]
    type Array;
    const LENGTH: u8;
    const INSTRUCTION_VALUE: u8;

    // Serialize can be implemented generically once const generics land
    fn serialize(&self) -> Self::Array;
}

pub(crate) trait Status {
    const LENGTH: u8;

    fn deserialize_parameters(id: ServoID, parameters: &[u8]) -> Self;
    
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

        let id = ServoID::new(data[2]);
        
        let parameters_range = 4..(4 + Self::LENGTH as usize - 2);
        Ok( Self::deserialize_parameters(id, &data[parameters_range]) )
    }
}

#[derive(Debug, Clone)]
pub struct ServoInfo {
    baud_rate: ::BaudRate,
    model_number: u16,
    fw_version: u8,
    id: ServoID,
}

   
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    Communication(::CommunicationError),
    Format(FormatError),
    Processing(ProcessingError),
}

impl From<::protocol1::Error> for ::Error {
    fn from(e: ::protocol1::Error) -> ::Error {
        match e {
            ::protocol1::Error::Communication(ce) => ::Error::Communication(ce),
            ::protocol1::Error::Format(_) => ::Error::Format,
            ::protocol1::Error::Processing(_) => ::Error::Processing,
        }
    }
}

impl From<::CommunicationError> for Error {
    fn from(e: ::CommunicationError) -> Error {
        Error::Communication(e)
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
pub(crate) enum PacketID {
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
