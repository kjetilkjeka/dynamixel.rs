pub mod control_table;

use Interface;
use Servo;

protocol1_servo!(MX28, ::dynamixel::mx28::control_table::WriteRegister, ::dynamixel::mx28::control_table::ReadRegister, 0x001D);

pub enum OperatingModes {
    Position,
}

impl<T: Interface> Servo for MX28<T> {
    type OperatingModes = OperatingModes;
    type Error = ::protocol1::Error;
    
    fn set_enable_torque(&mut self, enable_torque: bool) -> Result<(), Self::Error> {
        self.write_data(control_table::TorqueEnable::new(enable_torque))?;
        Ok(())
    }
    
    fn set_operating_mode(&mut self, _operating_mode: Self::OperatingModes) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_setpoint(&mut self, _operating_mode: Self::OperatingModes, value: f32) -> Result<(), Self::Error> {
        let goal_position = (2048i32 + (value*651.08854) as i32) as u16;
        self.write_data(control_table::GoalPosition::new(goal_position))?;
        Ok(())
    }
    
    fn get_position(&mut self) -> Result<f32, Self::Error> {
        unimplemented!()
    }
}
