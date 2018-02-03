pub mod control_table;

use Interface;
use Servo;

protocol1_servo!(MX28, ::dynamixel::mx28::control_table::WriteRegister, ::dynamixel::mx28::control_table::ReadRegister, 0x001D);

pub enum OperatingModes {
    Position,
}

impl Servo for MX28 {
    type OperatingModes = OperatingModes;
    type Error = ::protocol1::Error;
    
    fn set_enable_torque<I: Interface>(&mut self, interface: &mut I, enable_torque: bool) -> Result<(), Self::Error> {
        self.write_data(interface, control_table::TorqueEnable::new(enable_torque))?;
        Ok(())
    }
    
    fn set_operating_mode<I: Interface>(&mut self, interface: &mut I, _operating_mode: Self::OperatingModes) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_setpoint<I: Interface>(&mut self, interface: &mut I, _operating_mode: Self::OperatingModes, value: f32) -> Result<(), Self::Error> {
        let goal_position = (2048i32 + (value*651.08854) as i32) as u16;
        self.write_data(interface, control_table::GoalPosition::new(goal_position))?;
        Ok(())
    }
    
    fn get_position<I: Interface>(&mut self, interface: &mut I) -> Result<f32, Self::Error> {
        unimplemented!()
    }
}
