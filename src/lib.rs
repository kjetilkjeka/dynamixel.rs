#![no_std]

#[macro_use]
mod control_table;

pub mod protocol2;
pub mod pro;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
