use crate::World;

pub enum ControlCommand {
    NoOps,
    TurnLeft,
    TurnRight,
}
pub trait Controller {
    fn feed_input(&mut self, world: &World);
    fn get_output(&mut self) -> ControlCommand;
}
