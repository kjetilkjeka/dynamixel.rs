use Interface;
use Servo;

pub mod control_table;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum OperatingModes {
    Torque,
    Velocity,
    Position,
}

protocol2_servo!(M4210S260R, ::pro::control_table::WriteRegister, ::pro::control_table::ReadRegister, 0xA918);

impl Servo for M4210S260R {
    type OperatingModes = ::pro::OperatingModes;
    type Error = ::protocol2::Error;

    fn set_enable_torque<I: Interface>(&mut self, interface: &mut I, enable_torque: bool) -> Result<(), Self::Error> {
        self.write(interface, ::pro::control_table::TorqueEnable::new(enable_torque))?;
        Ok(())
    }
    
    fn set_operating_mode<I: Interface>(&mut self, interface: &mut I, operating_mode: Self::OperatingModes) -> Result<(), Self::Error> {
        match operating_mode {
            ::pro::OperatingModes::Torque => self.write(interface, ::pro::control_table::OperatingMode::new(0))?,
            ::pro::OperatingModes::Velocity => unimplemented!(),
            ::pro::OperatingModes::Position => self.write(interface, ::pro::control_table::OperatingMode::new(3))?,
        }
        Ok(())
    }

    fn set_setpoint<I: Interface>(&mut self, interface: &mut I, operating_mode: Self::OperatingModes, value: f32) -> Result<(), Self::Error> {
        match operating_mode {
            ::pro::OperatingModes::Torque => {
                let goal_torque = (value * 2048.0) as i16;
                self.write(interface, ::pro::control_table::GoalTorque::new(goal_torque))?;
            },                
            ::pro::OperatingModes::Velocity => unimplemented!(),
            ::pro::OperatingModes::Position => {
                let goal_position = ((value * 500.0) as i32) * 131593 / 1571;
                self.write(interface, ::pro::control_table::GoalPosition::new(goal_position))?;
            },
        }
        Ok(())
    }
    
    fn get_position<I: Interface>(&mut self, interface: &mut I) -> Result<f32, Self::Error> {
        let pos_fixed = i32::from(self.read::<I, ::pro::control_table::PresentPosition>(interface)?);
        let pos_rad = (pos_fixed as f32 * 1571.0)/(131593.0 * 500.0);
        Ok(pos_rad)
    }
}
