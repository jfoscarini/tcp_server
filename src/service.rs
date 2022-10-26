use std::error::Error;

use mio::Events;

use crate::server::{Server, ServerEvent};

pub struct Service {}

impl Service {
    pub fn run(server: &mut Server) -> Result<(), Box<dyn Error>> {
        let mut events = Events::with_capacity(1024);

        loop {
            for server_event in server.step(&mut events)? {
                match server_event {
                    ServerEvent::ReceiveUTF8(_, _) => {}
                    ServerEvent::ReceiveBytes(_, _) => {}
                    ServerEvent::ReceiveAdminUTF8(_, _) => {}
                    ServerEvent::Connect(_) => {}
                    ServerEvent::Disconnect(token) => {
                        server.disconnect(&token);
                    }
                }
            }
        }
    }
}
