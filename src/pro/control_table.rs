pub trait Register: ::protocol2::Register {}
pub trait ReadRegister: ::protocol2::ReadRegister {}
pub trait WriteRegister: ::protocol2::WriteRegister {}

rw_reg2!(OperatingMode, u8, 11);
rw_reg2!(TorqueEnable, bool, 562);
rw_reg2!(LedRed, u8, 563);
rw_reg2!(LedGreen, u8, 564);
rw_reg2!(LedBlue, u8, 565);
rw_reg2!(GoalPosition, i32, 596);
rw_reg2!(GoalTorque, i16, 604);
r_reg2!(PresentPosition, i32, 611);
r_reg2!(PresentVelocity, i32, 615);
r_reg2!(PresentCurrent, i16, 621);



