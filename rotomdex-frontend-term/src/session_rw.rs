use rotomdex_core::Session;

// TODO
pub struct SessionRw {}

impl SessionRw {
    pub fn new() -> Self {
        Self {}
    }
}

impl rotomdex_core::SessionRw for SessionRw {
    fn read(&self) -> rotomdex_core::Session {
        Session::default()
    }

    fn write(&self, _session: Session) {}
}
