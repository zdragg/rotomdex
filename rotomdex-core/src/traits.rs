use crate::Session;

pub trait SessionRw {
    fn read(&self) -> Session;
    fn write(&self, session: Session);
}
