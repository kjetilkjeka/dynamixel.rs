#![cfg_attr(not(feature = "std"), no_std)]

extern crate embedded_types;

#[macro_use]
mod control_table;

pub mod protocol2;
pub mod pro;

pub trait Interface {
    fn read(&mut self, &mut [u8]);
    fn write(&mut self, &[u8]);
}

#[cfg(feature = "std")]
impl<T: ::std::io::Read + ::std::io::Write> Interface for T {
    fn read(&mut self, buf: &mut [u8]) {
        self.read_exact(buf).unwrap();
    }
    
    fn write(&mut self, data: &[u8]) {
        self.write(data).unwrap();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
