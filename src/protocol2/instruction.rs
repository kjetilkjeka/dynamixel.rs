use protocol2::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Ping {
    id: PacketID,
}

impl Ping {
    pub fn new(id: PacketID) -> Self {
        Ping{id: id}
    }
}

impl Instruction for Ping {
    const PARAMETERS: u16 = 0;
    const INSTRUCTION_VALUE: u8 = 0x01;

    fn id(&self) -> PacketID {
        self.id
    }

    fn parameter(&self, _index: usize) -> u8 {
        panic!("No parameters exists for Ping");
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Pong {
    model_number: u16,
    fw_version: u8,
}

impl Status for Pong {
    const PARAMETERS: u16 = 3;
    
    fn deserialize_parameters(parameters: &[u8]) -> Self {
        assert_eq!(parameters.len(), 3);
        Pong {
            model_number: (parameters[0] as u16) | (parameters[1] as u16) << 8,
            fw_version: parameters[2],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
    const PARAMETERS: u16 = 4;
    const INSTRUCTION_VALUE: u8 = 0x02;

    fn id(&self) -> PacketID {
        self.id
    }

    fn parameter(&self, index: usize) -> u8 {
        match index {
            0 => T::ADDRESS as u8,
            1 => (T::ADDRESS >> 8) as u8,
            2 => T::SIZE as u8,
            3 => (T::SIZE >> 8) as u8,
            x => panic!("Read instruction parameter indexed with {}, only 4 parameters exists", x),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ReadResponse<T: ReadRegister> {
    pub value: T,
}

impl<T: ReadRegister> Status for ReadResponse<T> {
    const PARAMETERS: u16 = T::SIZE;

    fn deserialize_parameters(parameters: &[u8]) -> Self{
        ReadResponse{
            value: T::deserialize(parameters)
        }
    }
}


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
    const PARAMETERS: u16 = 2 + T::SIZE;
    const INSTRUCTION_VALUE: u8 = 0x03;

    fn id(&self) -> PacketID {
        self.id
    }
    
    fn parameter(&self, index: usize) -> u8 {
        match index {
            0 => T::ADDRESS as u8,
            1 => (T::ADDRESS >> 8) as u8,
            2 => self.data.serialize()[0],
            3 => self.data.serialize()[1],
            4 => self.data.serialize()[2],
            5 => self.data.serialize()[3],
            x => panic!("Read instruction parameter indexed with {}, only 6 parameters exists", x),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct WriteResponse {
}

impl Status for WriteResponse {
    const PARAMETERS: u16 = 0;
    
    fn deserialize_parameters(parameters: &[u8]) -> Self {
        assert_eq!(parameters.len(), 0);
        WriteResponse {}
    }
}

#[cfg(test)]
mod tests {
    // Using the same test case that can be found at:
    // http://support.robotis.com/en/product/actuator/dynamixel_pro/communication/instruction_status_packet.htm
    
    use protocol2::*;
    use protocol2::instruction::*;

    #[test]
    fn test_ping() {
        let ping = Ping::new(PacketID::unicast(1));
        let mut array = [0u8; 10];
        for (i, b) in ping.serialize().enumerate() {
            array[i] = b;
        }
        assert_eq!(array, [0xff, 0xff, 0xfd, 0x00, 0x01, 0x03, 0x00, 0x01, 0x19, 0x4e]);
        
        let ping = Ping::new(PacketID::broadcast());
        let mut array = [0u8; 10];
        for (i, b) in ping.serialize().enumerate() {
            array[i] = b;
        }
        assert_eq!(array, [0xff, 0xff, 0xfd, 0x00, 0xfe, 0x03, 0x00, 0x01, 0x31, 0x42]);
    }
    #[test]
    fn test_pong() {
        let mut deserializer = Deserializer::<Pong>::new();

        assert_eq!(deserializer.deserialize(&[0xff, 0xff, 0xfd, 0x00, 0x01, 0x07, 0x00, 0x55, 0x00, 0x06, 0x04, 0x026, 0x65, 0x5d]), Ok(DeserializationStatus::Finished));
        
        assert_eq!(deserializer.build(),
                   Ok(Pong{
                       model_number: 0x0406,
                       fw_version: 0x26,
                   })
        );
    }

    #[test]
    fn test_pong_mixed() {
        let mut deserializer = Deserializer::<Pong>::new();

        assert_eq!(deserializer.deserialize(&[0xff, 0xff, 0xfd, 0x00, 0x01, 0x07, 0x00]), Ok(DeserializationStatus::Ok));
        assert_eq!(deserializer.deserialize(&[0x55]), Ok(DeserializationStatus::Ok));
        assert_eq!(deserializer.deserialize(&[0x00]), Ok(DeserializationStatus::Ok));
        assert_eq!(deserializer.deserialize(&[0x18]), Ok(DeserializationStatus::Ok));
        assert_eq!(deserializer.deserialize(&[0xa9]), Ok(DeserializationStatus::Ok));
        assert_eq!(deserializer.deserialize(&[0x19]), Ok(DeserializationStatus::Ok));
        assert_eq!(deserializer.deserialize(&[0x76]), Ok(DeserializationStatus::Ok));
        
        assert!(!deserializer.is_finished());
        assert_eq!(deserializer.deserialize(&[0x32]), Ok(DeserializationStatus::Finished));        
        assert!(deserializer.is_finished());
        
        assert_eq!(deserializer.build(),
                   Ok(Pong{
                       model_number: 0xa918,
                       fw_version: 0x19,
                   })
        );

    }

    
    #[test]
    fn test_write() {
        let mut array = [0u8; 16];
        let write = Write::new(PacketID::unicast(1), ::pro::control_table::GoalPosition::new(0xabcd));
        for (i, b) in write.serialize().enumerate() {
            array[i] = b;
        }
        assert_eq!(
            array,
            [0xff, 0xff, 0xfd, 0x00, 0x01, 0x09, 0x00, 0x03, 0x54, 0x02, 0xcd, 0xab, 0x00, 0x00, 0x0d, 0xe5]
        );

        // Test write that needs stuffing
        let mut array = [0u8; 17];
        let write = Write::new(PacketID::unicast(1), ::pro::control_table::GoalPosition::new(0xfdffff));
        for (i, b) in write.serialize().enumerate() {
            array[i] = b;
        }
        assert_eq!(
            array,
            [0xff, 0xff, 0xfd, 0x00, 0x01, 0x0a, 0x00, 0x03, 0x54, 0x02, 0xff, 0xff, 0xfd, 0xfd, 0x00, 33, 53]
        );

    }

    #[test]
    fn test_write_response_byte() {
        let mut deserializer = Deserializer::<WriteResponse>::new();

        assert_eq!(deserializer.deserialize(&[0xff]), Ok(DeserializationStatus::Ok));
        assert_eq!(deserializer.deserialize(&[0xff]), Ok(DeserializationStatus::Ok));
        assert_eq!(deserializer.deserialize(&[0xfd]), Ok(DeserializationStatus::Ok));
        assert_eq!(deserializer.deserialize(&[0x00]), Ok(DeserializationStatus::Ok));
        assert_eq!(deserializer.deserialize(&[0x01]), Ok(DeserializationStatus::Ok));
        assert_eq!(deserializer.deserialize(&[0x04]), Ok(DeserializationStatus::Ok));
        assert_eq!(deserializer.deserialize(&[0x00]), Ok(DeserializationStatus::Ok));
        assert_eq!(deserializer.deserialize(&[0x55]), Ok(DeserializationStatus::Ok));
        assert_eq!(deserializer.deserialize(&[0x00]), Ok(DeserializationStatus::Ok));
        assert_eq!(deserializer.deserialize(&[0xa1]), Ok(DeserializationStatus::Ok));
        assert_eq!(deserializer.deserialize(&[0x0c]), Ok(DeserializationStatus::Finished));        

        assert!(deserializer.is_finished());
        
        assert_eq!(deserializer.build(),
                   Ok(WriteResponse{})
        );

    }

    #[test]
    fn test_write_response_mixed() {
        let mut deserializer = Deserializer::<WriteResponse>::new();

        assert_eq!(deserializer.deserialize(&[0xff, 0xff, 0xfd, 0x00, 0x01, 0x04, 0x00]), Ok(DeserializationStatus::Ok));
        assert_eq!(deserializer.deserialize(&[0x55]), Ok(DeserializationStatus::Ok));
        assert_eq!(deserializer.deserialize(&[0x00]), Ok(DeserializationStatus::Ok));
        assert_eq!(deserializer.deserialize(&[0xa1]), Ok(DeserializationStatus::Ok));
        
        assert!(!deserializer.is_finished());
        assert_eq!(deserializer.deserialize(&[0x0c]), Ok(DeserializationStatus::Finished));        
        assert!(deserializer.is_finished());
        
        assert_eq!(deserializer.build(),
                   Ok(WriteResponse{})
        );

    }


    #[test]
    fn test_read() {
        let mut array = [0u8; 14];
        let read = Read::<::pro::control_table::PresentPosition>::new(PacketID::unicast(1));
        for (i, b) in read.serialize().enumerate() {
            array[i] = b;
        }
        assert_eq!(
            array,
            [0xff, 0xff, 0xfd, 0x00, 0x01, 0x07, 0x00, 0x02, 611u16 as u8, (611u16 >> 8) as u8, 0x04, 0x00, 27, 249]
        );
    }

    #[test]
    fn test_read_response() {
        let mut deserializer = Deserializer::<ReadResponse<::pro::control_table::GoalPosition>>::new();

        assert_eq!(deserializer.deserialize(&[0xff, 0xff, 0xfd, 0x00, 0x01, 0x08, 0x00, 0x55, 0x00, 0xa6, 0x00, 0x00, 0x00, 0x8c, 0xc0]), Ok(DeserializationStatus::Finished));
        
        assert_eq!(deserializer.build(),
                   Ok(ReadResponse{
                       value: ::pro::control_table::GoalPosition::new(0x000000a6),
                   })
        );

    }
}
