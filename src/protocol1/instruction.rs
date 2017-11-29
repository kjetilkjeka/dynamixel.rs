use protocol1::*;

pub struct Ping {
    id: PacketID,
}

impl Ping {
    pub fn new(id: PacketID) -> Self {
        Ping{id: id}
    }
}

impl Instruction for Ping {
    type Array = [u8; 6];
    const LENGTH: u8 = 2;
    const INSTRUCTION_VALUE: u8 = 0x01;

    fn serialize(&self) -> [u8; 6] {
        let mut array = [0xff, 0xff, u8::from(self.id), Self::LENGTH, Self::INSTRUCTION_VALUE, 0x00];
        array[5] = u8::from(checksum::Checksum::calc(&array[2..5]));
        array
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Pong {}

impl Status for Pong {
    const LENGTH: u8 = 2;

    fn deserialize_parameters(parameters: &[u8]) -> Self {
        assert_eq!(parameters.len(), 0);
        Pong {}
    }
}


#[cfg(test)]
mod tests {
    // Using the same test case that can be found at:
    // http://support.robotis.com/en/product/actuator/dynamixel/communication/dxl_instruction.htm
    
    use protocol1::*;
    use protocol1::instruction::*;

    #[test]
    fn test_ping() {
        assert_eq!(Ping::new(PacketID::unicast(1)).serialize(), [0xff, 0xff, 0x01, 0x02, 0x01, 0xfb]);
    }
    
    #[test]
    fn test_pong() {
        assert_eq!(Pong::deserialize(&[0xff, 0xff, 0x01, 0x02, 0x00, 0x00]), //todo: fix checksum
                   Ok(Pong{})
        );
    }
}
