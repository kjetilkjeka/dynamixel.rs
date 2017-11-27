#[macro_use]
use control_table;

pub trait Register: ::protocol2::Register {}
pub trait ReadRegister: ::protocol2::ReadRegister {}
pub trait WriteRegister: ::protocol2::WriteRegister {}

rw_reg!(OperatingMode, u8, 11);
rw_reg!(TorqueEnable, bool, 562);
rw_reg!(LedRed, u8, 563);
rw_reg!(LedGreen, u8, 564);
rw_reg!(LedBlue, u8, 565);
rw_reg!(GoalPosition, i32, 596);
rw_reg!(GoalTorque, i16, 604);
r_reg!(PresentPosition, i32, 611);
r_reg!(PresentVelocity, i32, 615);
r_reg!(PresentCurrent, i16, 621);



