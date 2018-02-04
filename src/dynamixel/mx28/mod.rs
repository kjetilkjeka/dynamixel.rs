pub mod control_table;

use Interface;
use Servo;

protocol1_servo!(MX28, ::dynamixel::mx28::control_table::WriteRegister, ::dynamixel::mx28::control_table::ReadRegister, 0x001D);

impl<I: Interface> Servo<I> for MX28<I> {
    fn set_enable_torque(&mut self, interface: &mut I, enable_torque: bool) -> Result<(), ::Error> {
        Ok(self.write_data(interface, control_table::TorqueEnable::new(enable_torque))?)
    }
    
    fn set_position(&mut self, interface: &mut I, value: f32) -> Result<(), ::Error> {
        let goal_position = (2048i32 + (value*651.08854) as i32) as u16;
        Ok(self.write_data(interface, control_table::GoalPosition::new(goal_position))?)
    }
    
    fn get_position(&mut self, interface: &mut I) -> Result<f32, ::Error> {
        let pos_fixed = i32::from(u16::from(self.read_data::<::dynamixel::mx28::control_table::PresentPosition>(interface)?));
        let pos_rad = ((pos_fixed - 2048i32) as f32)/652.23f32;
        Ok(pos_rad)
    }
}
