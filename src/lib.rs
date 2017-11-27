#![cfg_attr(not(feature = "std"), no_std)]

extern crate embedded_types;

#[macro_use]
mod control_table;

pub mod protocol2;
pub mod pro;

#[cfg(not(feature = "std"))]
pub trait Interface : ::embedded_types::io::Read + ::embedded_types::io::Write {}

#[cfg(feature = "std")]
pub trait Interface : ::std::io::BufRead + ::std::io::Write {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
