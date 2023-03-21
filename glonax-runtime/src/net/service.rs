use std::sync::Arc;

use glonax_j1939::*;

use super::{J1939Network, Routable};

pub struct J1939ApplicationInspector {
    software_indent: Option<(u8, u8, u8)>,
    request_pgn: Option<u32>,
    address_claim: Option<(u8, u8)>,
    acknowledged: Option<u8>,
}

impl Routable for J1939ApplicationInspector {
    fn ingress(&mut self, frame: &Frame) -> bool {
        self.software_indent = None;
        self.request_pgn = None;
        self.address_claim = None;
        self.acknowledged = None;

        match frame.id().pgn() {
            PGN::SoftwareIdentification => {
                let mut major = 0;
                let mut minor = 0;
                let mut patch = 0;

                if frame.pdu()[3] != 0xff {
                    major = frame.pdu()[3];
                }
                if frame.pdu()[4] != 0xff {
                    minor = frame.pdu()[4];
                }
                if frame.pdu()[5] != 0xff {
                    patch = frame.pdu()[5];
                }

                self.software_indent = Some((major, minor, patch));

                true
            }
            PGN::Request => {
                self.request_pgn = Some(u32::from_be_bytes([
                    0x0,
                    frame.pdu()[2],
                    frame.pdu()[1],
                    frame.pdu()[0],
                ]));

                true
            }
            PGN::AddressClaimed => {
                let function = frame.pdu()[5];
                let arbitrary_address = frame.pdu()[7] >> 7;

                self.address_claim = Some((function, arbitrary_address));

                true
            }
            PGN::AcknowledgmentMessage => {
                self.acknowledged = Some(frame.pdu()[0]);

                true
            }
            _ => false,
        }
    }
}

impl J1939ApplicationInspector {
    pub fn new() -> Self {
        Self {
            software_indent: None,
            request_pgn: None,
            address_claim: None,
            acknowledged: None,
        }
    }

    #[inline]
    pub fn software_identification(&self) -> Option<(u8, u8, u8)> {
        self.software_indent
    }

    #[inline]
    pub fn request(&self) -> Option<u32> {
        self.request_pgn
    }

    #[inline]
    pub fn address_claimed(&self) -> Option<(u8, u8)> {
        self.address_claim
    }

    #[inline]
    pub fn acknowledged(&self) -> Option<u8> {
        self.acknowledged
    }
}

impl Default for J1939ApplicationInspector {
    fn default() -> Self {
        Self::new()
    }
}

pub struct StatusService {
    net: Arc<J1939Network>,
    node: u8,
}

impl StatusService {
    pub fn new(net: Arc<J1939Network>, node: u8) -> Self {
        Self { net, node }
    }

    pub async fn set_led(&self, led_on: bool) {
        let frame = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietarilyConfigurableMessage1)
                .da(self.node)
                .build(),
        )
        .copy_from_slice(&[b'Z', b'C', u8::from(led_on)])
        .build();

        self.net.send(&frame).await.unwrap();
    }
}
