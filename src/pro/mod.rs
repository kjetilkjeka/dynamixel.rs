pub mod control_table;
pub mod models;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum OperatingModes {
    Torque,
    Velocity,
    Position,
}
