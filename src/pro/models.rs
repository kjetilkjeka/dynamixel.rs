#[macro_use]
use protocol2;
use Interface;
use Servo;
use Error;

protocol2_servo!(M4210S260R, ::pro::control_table::WriteRegister, ::pro::control_table::ReadRegister);

impl<T: Interface> Servo for M4210S260R<T> {
    type OperatingModes = ::pro::OperatingModes;

    fn set_enable_torque(&mut self, enable_torque: bool) -> Result<(), Error> {
        self.write(::pro::control_table::TorqueEnable::new(enable_torque)).unwrap();
        Ok(())
    }
    
    fn set_operating_mode(&mut self, operating_mode: Self::OperatingModes) -> Result<(), Error> {
        match operating_mode {
            ::pro::OperatingModes::Torque => self.write(::pro::control_table::OperatingMode::new(0)).unwrap(),
            ::pro::OperatingModes::Velocity => unimplemented!(),
            ::pro::OperatingModes::Position => self.write(::pro::control_table::OperatingMode::new(3)).unwrap(),
        }
        Ok(())
    }

    fn set_setpoint(&mut self, operating_mode: Self::OperatingModes, value: f32) -> Result<(), Error> {
        match operating_mode {
            ::pro::OperatingModes::Torque => {
                let goal_torque = ((value * 2048.0) as i16);
                self.write(::pro::control_table::GoalTorque::new(goal_torque)).unwrap();
            },                
            ::pro::OperatingModes::Velocity => unimplemented!(),
            ::pro::OperatingModes::Position => {
                let goal_position = ((value * 500.0) as i32) * 131593 / 1571;
                self.write(::pro::control_table::GoalPosition::new(goal_position)).unwrap();
            },
        }
        Ok(())
    }
    
    fn get_position(&mut self) -> Result<f32, Error> {
        let pos_fixed = i32::from(self.read::<::pro::control_table::PresentPosition>().unwrap());
        let pos_rad = (pos_fixed as f32 * 1571.0)/(131593.0 * 500.0);
        Ok(pos_rad)
    }
}
