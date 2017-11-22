pub struct TorqueEnable(bool);

impl ::protocol2::Register for TorqueEnable {
    const SIZE: u16 = 1;
    const ADDRESS: u16 = 562;
}

impl ::protocol2::ReadRegister for TorqueEnable {
    fn deserialize(data: [u8; 4]) -> Self {
        TorqueEnable(data[0]&1 == 1)
    }
}

impl ::protocol2::WriteRegister for TorqueEnable {
    fn serialize(&self) -> [u8; 4] {
        if self.0 {
            [1, 0, 0, 0]
        } else {
            [0, 0, 0, 0]
        }
    }    
}

pub struct LedRed(u8);

impl ::protocol2::Register for LedRed {
    const SIZE: u16 = 1;
    const ADDRESS: u16 = 563;
}

impl ::protocol2::ReadRegister for LedRed {
    fn deserialize(data: [u8; 4]) -> Self {
        LedRed(data[0])
    }
}

impl ::protocol2::WriteRegister for LedRed {
    fn serialize(&self) -> [u8; 4] {
        [self.0 as u8, 0, 0, 0]
    }    
}

pub struct GoalPosition(u32);

impl GoalPosition {
    pub fn new(v: u32) -> Self {
        GoalPosition(v)
    }
}

impl ::protocol2::Register for GoalPosition {
    const SIZE: u16 = 4;
    const ADDRESS: u16 = 596;
}

impl ::protocol2::ReadRegister for GoalPosition {
    fn deserialize(data: [u8; 4]) -> Self {
        GoalPosition(data[0] as u32 | (data[1] as u32) << 8 | (data[2] as u32) << 16 | (data[3] as u32) << 24)
    }
}

impl ::protocol2::WriteRegister for GoalPosition {
    fn serialize(&self) -> [u8; 4] {
        [self.0 as u8, (self.0 >> 8) as u8, (self.0 >> 16) as u8, (self.0 >> 24) as u8]
    }    
}



