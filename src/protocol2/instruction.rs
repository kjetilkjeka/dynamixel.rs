use protocol2::*;

pub struct Ping {
    id: PacketID,
}

impl Ping {
    pub fn new(id: PacketID) -> Self {
        Ping{id: id}
    }
}

impl Instruction for Ping {
    type Array = [u8; 10];
    const LENGTH: u16 = 3;
    const INSTRUCTION_VALUE: u8 = 0x01;

    fn serialize(&self) -> [u8; 10] {
        let mut array = [0xff, 0xff, 0xfd, 0x00, u8::from(self.id), Self::LENGTH as u8, (Self::LENGTH >> 8) as u8, Self::INSTRUCTION_VALUE, 0x00, 0x00];
        let crc = u16::from(crc::CRC::calc(&array[0..8]));
        array[8] = crc as u8;
        array[9] = (crc >> 8) as u8;
        array
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Pong {
    model_number: u16,
    fw_version: u8,
}

impl Status for Pong {
    type Array = [u8; 14];
    const LENGTH: u16 = 7;

    fn deserialize(data: Self::Array) -> Result<Self, Error>
        where Self : Sized {
        // check for formating error stuff
        
        // check for processing errors
        if let Some(error) = ProcessingError::decode(data[8]) {
            return Err(Error::Processing(error));
        }
        
        Ok( Pong {
            model_number: (data[9] as u16) | (data[10] as u16) << 8,
            fw_version: data[11],
        } )
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Read<T: ReadRegister> {
    id: PacketID,
    phantom: ::lib::marker::PhantomData<T>,
}

impl<T: ReadRegister> Read<T> {
    pub fn new(id: PacketID) -> Self {
        Read{id: id, phantom: ::lib::marker::PhantomData}
    }
}

impl<T: ReadRegister> Instruction for Read<T> {
    type Array = [u8; 14];
    const LENGTH: u16 = 7;
    const INSTRUCTION_VALUE: u8 = 0x02;

    fn serialize(&self) -> [u8; 14] {
        let mut array = [0xff, 0xff, 0xfd, 0x00, u8::from(self.id), Self::LENGTH as u8, (Self::LENGTH >> 8) as u8, Self::INSTRUCTION_VALUE, T::ADDRESS as u8, (T::ADDRESS >> 8) as u8, T::SIZE as u8, (T::SIZE >> 8) as u8, 0x00, 0x00];
        let crc = u16::from(crc::CRC::calc(&array[0..(10+T::SIZE) as usize]));
        array[12] = crc as u8;
        array[13] = (crc >> 8) as u8;
        array
    }   
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ReadResponse<T: ReadRegister> {
    value: T,
}

impl<T: ReadRegister> Status for ReadResponse<T> {
    type Array = [u8; 15];
    const LENGTH: u16 = 4 + T::SIZE;
    
    fn deserialize(data: Self::Array) -> Result<Self, Error>
        where Self : Sized {
        // check for formating error stuff
        
        // check for processing errors
        if let Some(error) = ProcessingError::decode(data[8]) {
            return Err(Error::Processing(error));
        }

        let mut parameters = [0, 0, 0, 0];
        for i in 0..T::SIZE as usize {
            parameters[i] = data[9+i];
        }
        
        Ok( ReadResponse {
            value: T::deserialize(parameters),
        } )
    }
}


pub struct Write<T: WriteRegister> {
    id: PacketID,
    data: T,
}

impl<T: WriteRegister> Write<T> {
    pub fn new(id: PacketID, data: T) -> Self {
        Write{id: id, data: data}
    }
}

impl<T: WriteRegister> Instruction for Write<T>{
    // Use max size (4) untill const generics land
    type Array = [u8; 16];
    const LENGTH: u16 = 5 + T::SIZE;
    const INSTRUCTION_VALUE: u8 = 0x03;

    fn serialize(&self) -> [u8; 16] {
        let mut array = [0xff, 0xff, 0xfd, 0x00, u8::from(self.id), Self::LENGTH as u8, (Self::LENGTH >> 8) as u8, Self::INSTRUCTION_VALUE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        array[8] = T::ADDRESS as u8;
        array[9] = (T::ADDRESS >> 8) as u8;
        let data = self.data.serialize();
        for i in 0..T::SIZE as usize {
            array[10+i] = data[i];
        }
        let crc = u16::from(crc::CRC::calc(&array[0..(10+T::SIZE) as usize]));
        array[10+T::SIZE as usize] = crc as u8;
        array[11+T::SIZE as usize] = (crc >> 8) as u8;
        array
    }
}

pub struct WriteResponse {
}

impl Status for WriteResponse {
    type Array = [u8; 11];
    const LENGTH: u16 = 4;
    
    fn deserialize(data: Self::Array) -> Result<Self, Error>
        where Self : Sized {
        // check for formating error stuff
        
        // check for processing errors
        if let Some(error) = ProcessingError::decode(data[8]) {
            return Err(Error::Processing(error));
        }
        
        Ok( WriteResponse {
        } )
    }
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
    const LENGTH: u16 = 10;
    const INSTRUCTION_VALUE: u8 = 0x08;
}

#[cfg(test)]
mod tests {
    // Using the same test case that can be found at:
    // http://support.robotis.com/en/product/actuator/dynamixel_pro/communication/instruction_status_packet.htm
    
    use protocol2::*;
    use protocol2::instruction::*;

    #[test]
    fn test_ping() {
        assert_eq!(Ping::new(PacketID::unicast(1)).serialize(), [0xff, 0xff, 0xfd, 0x00, 0x01, 0x03, 0x00, 0x01, 0x19, 0x4e]);
        assert_eq!(Ping::new(PacketID::broadcast()).serialize(), [0xff, 0xff, 0xfd, 0x00, 0xfe, 0x03, 0x00, 0x01, 0x31, 0x42]);
    }
    
    #[test]
    fn test_pong() {
        assert_eq!(Pong::deserialize([0xff, 0xff, 0xfd, 0x00, 0x01, 0x07, 0x00, 0x55, 0x00, 0x06, 0x04, 0x026, 0x65, 0x5d]),
                   Ok(Pong{
                       model_number: 0x0406,
                       fw_version: 0x26,
                   })
        );
    }
    
    #[test]
    fn test_write() {
        assert_eq!(
            Write::new(PacketID::unicast(1), ::pro::control_table::GoalPosition::new(0xabcd)).serialize(),
            [0xff, 0xff, 0xfd, 0x00, 0x01, 0x09, 0x00, 0x03, 0x54, 0x02, 0xcd, 0xab, 0x00, 0x00, 0x0d, 0xe5]
        );
    }

    #[test]
    fn test_read() {
        assert_eq!(
            Read::<::pro::control_table::PresentPosition>::new(PacketID::unicast(1)).serialize(),
            [0xff, 0xff, 0xfd, 0x00, 0x01, 0x07, 0x00, 0x02, 611u16 as u8, (611u16 >> 8) as u8, 0x04, 0x00, 85, 150]
        );
    }

    #[test]
    fn test_read_response() {
        assert_eq!(ReadResponse::<::pro::control_table::GoalPosition>::deserialize([0xff, 0xff, 0xfd, 0x00, 0x01, 0x08, 0x00, 0x55, 0x00, 0xa6, 0x00, 0x00, 0x00, 0x8c, 0xc0]),
                   Ok(ReadResponse{
                       value: ::pro::control_table::GoalPosition::new(0x000000a6),
                   })
        );
    }
}
