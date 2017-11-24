#[macro_export]
macro_rules! rw_reg{
    ($name:ident, $type:ident, $address:expr) => {
        register_impl!($name, $type, $address);

        impl $name {
            pub fn new(v: $type) -> Self {
                $name(v)
            }
        }

        read_register_impl!($name, $type);
        write_register_impl!($name, $type);
    };
}

macro_rules! register_impl{
    ($name:ident, bool, $address:expr) => {
        pub struct $name(bool);

        impl ::protocol2::Register for $name {
            const SIZE: u16 = 1;
            const ADDRESS: u16 = $address;
        }
    };
    ($name:ident, u8, $address:expr) => {
        pub struct $name(u8);
        
        impl ::protocol2::Register for $name {
            const SIZE: u16 = 1;
            const ADDRESS: u16 = $address;
        }
    };
    ($name:ident, i16, $address:expr) => {
        pub struct $name(i16);
        
        impl ::protocol2::Register for $name {
            const SIZE: u16 = 2;
            const ADDRESS: u16 = $address;
        }
    };
    ($name:ident, i32, $address:expr) => {
        pub struct $name(i32);
        
        impl ::protocol2::Register for $name {
            const SIZE: u16 = 4;
            const ADDRESS: u16 = $address;
        }
    };

}

macro_rules! read_register_impl{
    ($name:ident, bool) => {
        impl ReadRegister for $name {}
        
        impl ::protocol2::ReadRegister for $name {
            fn deserialize(data: [u8; 4]) -> Self {
                $name(data[0]&1 == 1)
            }
        }
    };
    ($name:ident, u8) => {
        impl ReadRegister for $name {}
        
        impl ::protocol2::ReadRegister for $name {
            fn deserialize(data: [u8; 4]) -> Self {
                $name(data[0])
            }
        }
    };
    ($name:ident, i8) => {
        impl ReadRegister for $name {}
        
        impl ::protocol2::ReadRegister for $name {
            fn deserialize(data: [u8; 4]) -> Self {
                $name(data[0] as i8)
            }
        }
    };
    ($name:ident, i16) => {
        impl ReadRegister for $name {}
        
        impl ::protocol2::ReadRegister for $name {
            fn deserialize(data: [u8; 4]) -> Self {
                $name(data[0] as i16 | ((data[1] as u16) << 8) as i16)
            }
        }
    };
    ($name:ident, u32) => {
        impl ReadRegister for $name {}

        impl ::protocol2::ReadRegister for $name {
            fn deserialize(data: [u8; 4]) -> Self {
                $name((data[0] as u32 | (data[1] as u32) << 8 | (data[2] as u32) << 16 | (data[3] as u32) << 24))
            }
        }
    };
    ($name:ident, i32) => {
        impl ReadRegister for $name {}
        
        impl ::protocol2::ReadRegister for $name {
            fn deserialize(data: [u8; 4]) -> Self {
                $name((data[0] as u32 | (data[1] as u32) << 8 | (data[2] as u32) << 16 | (data[3] as u32) << 24) as i32)
            }
        }
    };
}

macro_rules! write_register_impl{
    ($name:ident, bool) => {
        impl WriteRegister for $name {}
        
        impl ::protocol2::WriteRegister for $name {
            fn serialize(&self) -> [u8; 4] {
                [self.0 as u8, 0, 0, 0]
            }    
        }
    };
    ($name:ident, u8) => {
        impl WriteRegister for $name {}
        
        impl ::protocol2::WriteRegister for $name {
            fn serialize(&self) -> [u8; 4] {
                [self.0, 0, 0, 0]
            }    
        }
    };
    ($name:ident, i8) => {
        impl WriteRegister for $name {}
        
        impl ::protocol2::WriteRegister for $name {
            fn serialize(&self) -> [u8; 4] {
                [self.0 as u8, 0, 0, 0]
            }    
        }
    };
    ($name:ident, i16) => {
        impl WriteRegister for $name {}
        
        impl ::protocol2::WriteRegister for $name {
            fn serialize(&self) -> [u8; 4] {
                [self.0 as u8, (self.0 >> 8) as u8, 0, 0]
            }    
        }
    };
    ($name:ident, u32) => {
        impl WriteRegister for $name {}
        
        impl ::protocol2::WriteRegister for $name {
            fn serialize(&self) -> [u8; 4] {
                [self.0 as u8, (self.0 >> 8) as u8, (self.0 >> 16) as u8, (self.0 >> 24) as u8]
            }    
        }
    };
    ($name:ident, i32) => {
        impl WriteRegister for $name {}
        
        impl ::protocol2::WriteRegister for $name {
            fn serialize(&self) -> [u8; 4] {
                [self.0 as u8, (self.0 >> 8) as u8, (self.0 >> 16) as u8, (self.0 >> 24) as u8]
            }    
        }
    };
}
