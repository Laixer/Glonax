use super::{Body, Command, Header};

/// Protocol magic identifier.
///
/// The magic sequence indicates the start a new gloproto packet.
const MAGIC: &[u8] = b"\xa0\xf7";

#[derive(Debug)]
#[repr(C)]
pub struct Frame {
    /// Frame header.
    pub header: Header,
    /// Optional subcommand body.
    pub body: Option<Body>,
}

impl Frame {
    /// Minimum frame size.
    pub const MIN_FRAME_SIZE: usize = MAGIC.len() + std::mem::size_of::<Header>();

    fn parse_header(input: &[u8]) -> nom::IResult<&[u8], Header> {
        use nom::{
            bytes::streaming::{tag, take_until},
            number::streaming::u8,
            sequence::preceded,
            sequence::tuple,
        };

        nom::combinator::map_res(
            tuple((
                preceded(preceded(take_until(MAGIC), tag(MAGIC)), u8),
                u8,
                u8,
                u8,
            )),
            |(u1, u2, u3, u4)| Header::from_tuple((u1, u2, u3, u4)),
        )(input)
    }

    fn parse_body<'a>(input: &'a [u8], header: &Header) -> nom::IResult<&'a [u8], Option<Body>> {
        use nom::{
            bytes::streaming::take,
            combinator::map,
            number::streaming::{i8, le_u16, u8},
            sequence::tuple,
        };

        match header.ty {
            Command::CmdInfo => {
                let body_info = tuple((le_u16, u8, u8))(input)?;
                Ok((
                    body_info.0,
                    Some(Body::Info {
                        unique_id: body_info.1 .0,
                        firmware_major: body_info.1 .1,
                        firmware_minor: body_info.1 .2,
                    }),
                ))
            }
            Command::CmdBoot => Ok((input, None)),
            Command::CmdIdle => Ok((input, None)),
            Command::CmdCustom => {
                let body_custom = tuple((le_u16, le_u16, le_u16))(input)?;

                let payload: (&[u8], Vec<_>) =
                    map(take(body_custom.1 .2), |payload_raw: &[u8]| {
                        payload_raw.to_vec()
                    })(body_custom.0)?;

                Ok((
                    payload.0,
                    Some(Body::Custom {
                        subcommand: body_custom.1 .0,
                        flags: body_custom.1 .1,
                        payload: payload.1,
                    }),
                ))
            }
            Command::CmdError => {
                let body_error = i8(input)?;
                Ok((body_error.0, Some(Body::Error(body_error.1))))
            }
        }
    }

    fn proto_sequence(input: &[u8]) -> nom::IResult<&[u8], (Header, Option<Body>)> {
        let p1 = Self::parse_header;
        let (input, output1) = p1(input)?;

        let p2 = Self::parse_body;
        let (input, output2) = p2(input, &output1)?;

        Ok((input, (output1, output2)))
    }

    // FUTURE: Return local resultset.
    pub fn parse(data: &[u8]) -> nom::IResult<&[u8], Frame> {
        if data.len() < Self::MIN_FRAME_SIZE * 2 {
            return Err(nom::Err::Incomplete(nom::Needed::new(
                Self::MIN_FRAME_SIZE * 2 - data.len(),
            )));
        }
        nom::combinator::map(Self::proto_sequence, |(header, body)| Frame {
            header,
            body,
        })(data)
    }

    /// Return the frame as a byte array.
    pub fn to_bytes(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(Self::MIN_FRAME_SIZE * 4);

        let header = unsafe {
            std::slice::from_raw_parts(
                (&self.header as *const Header) as *const u8,
                std::mem::size_of::<Header>(),
            )
        };

        buf.extend(MAGIC);
        buf.extend(header);

        if let Some(body) = self.body {
            match body {
                Body::Error(error) => buf.push(error as u8),
                Body::Info {
                    unique_id,
                    firmware_major,
                    firmware_minor,
                } => {
                    buf.extend(unique_id.to_le_bytes());
                    buf.push(firmware_major);
                    buf.push(firmware_minor);
                }
                Body::Custom {
                    subcommand,
                    flags,
                    payload,
                } => {
                    buf.extend(subcommand.to_le_bytes());
                    buf.extend(flags.to_le_bytes());
                    buf.extend((payload.len() as u16).to_le_bytes());
                    buf.extend(payload.to_owned());
                }
            }
        }

        buf
    }
}
