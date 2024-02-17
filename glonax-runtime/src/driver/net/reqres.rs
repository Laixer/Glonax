use j1939::{protocol, Frame, PGN};

use crate::net::Parsable;

#[derive(Default)]
pub struct RequestResponder {
    /// Source address.
    source_address: u8,
}

impl RequestResponder {
    pub fn new(sa: u8) -> Self {
        Self { source_address: sa }
    }
}

impl Parsable<PGN> for RequestResponder {
    fn parse(&mut self, frame: &Frame) -> Option<PGN> {
        if frame.id().pgn() == PGN::Request {
            if frame.id().destination_address() != Some(self.source_address) {
                return None;
            }

            Some(protocol::request_from_pdu(frame.pdu()))
        } else {
            None
        }
    }
}
