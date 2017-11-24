#[macro_use]
use control_table;

pub trait Register: ::protocol2::Register {}
pub trait ReadRegister: ::protocol2::ReadRegister {}
pub trait WriteRegister: ::protocol2::WriteRegister {}

rw_reg!(TorqueEnable, bool, 562);
rw_reg!(LedRed, u8, 563);
rw_reg!(GoalPosition, i32, 596);



