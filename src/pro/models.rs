#[macro_use]
use protocol2;

protocol2_servo!(M4210S260R, ::pro::control_table::WriteRegister, ::pro::control_table::ReadRegister);
