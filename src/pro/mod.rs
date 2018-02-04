//! Dynamixel Pro servos
//!
//! Documentation can be found in: http://support.robotis.com/en/product/actuator/dynamixel_pro.html


use Interface;
use Servo;

pub mod control_table;

protocol2_servo!(M4210S260R, ::pro::control_table::WriteRegister, ::pro::control_table::ReadRegister, 0xA918);

impl<I: Interface> Servo<I> for M4210S260R<I> {
    fn set_enable_torque(&mut self, interface: &mut I, enable_torque: bool) -> Result<(), ::Error> {
        self.write(interface, ::pro::control_table::TorqueEnable::new(enable_torque))?;
        Ok(())
    }

    fn set_position(&mut self, interface: &mut I, value: f32) -> Result<(), ::Error> {
        let goal_position = ((value * 500.0) as i32) * 131593 / 1571;
        Ok(self.write(interface, ::pro::control_table::GoalPosition::new(goal_position))?)
    }
    
    fn get_position(&mut self, interface: &mut I) -> Result<f32, ::Error> {
        let pos_fixed = i32::from(self.read::<::pro::control_table::PresentPosition>(interface)?);
        let pos_rad = (pos_fixed as f32 * 1571.0)/(131593.0 * 500.0);
        Ok(pos_rad)
    }
}
