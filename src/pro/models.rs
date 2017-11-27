use Interface;

pub struct M4210S260R<T: Interface> {
    interface: T,
}

impl<T: Interface> M4210S260R<T> {
    pub fn new(interface: T) -> Self {
        M4210S260R{
            interface: interface,
        }
    }
}

impl<T: Interface> ::protocol2::Servo<T> for M4210S260R<T> {
    fn interface(&mut self) -> &mut T {
        &mut self.interface
    }
}
