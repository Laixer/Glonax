use bytes::{Buf, BytesMut};
use tokio::{io::AsyncReadExt, net::TcpStream};

mod proto {
    use bytes::Buf;

    #[derive(Debug)]
    pub enum FrameError {
        Corrupt,
        Incomplete,
    }

    type FrameResult<T> = std::result::Result<T, FrameError>;

    #[derive(Debug)]
    pub enum DemuxVersion {
        Version1 = 1,
    }

    #[derive(Debug)]
    pub enum DemuxApplication {
        Motion(u32, i16),
    }

    #[derive(Debug)]
    pub struct ApplicationHeader {
        pub version: DemuxVersion,
        pub application: DemuxApplication,
    }

    #[derive(Debug)]
    pub struct Protocol {
        pub header: ApplicationHeader,
    }

    pub fn parse<T: AsRef<[u8]> + Buf>(mut buf: T) -> FrameResult<Protocol> {
        const PREAMBLE: &[u8] = b"GLNX1\r\n\0";

        if buf.remaining() <= 8 {
            return Err(FrameError::Incomplete);
        }

        let header = buf.copy_to_bytes(PREAMBLE.len());
        if header != PREAMBLE {
            return Err(FrameError::Corrupt);
        }

        let ver = match buf.get_u16() {
            1 => Ok(DemuxVersion::Version1),
            _ => Err(()),
        };

        let app = match ver {
            Ok(_) => match buf.get_u16() {
                64 => Ok(DemuxApplication::Motion(buf.get_u32(), buf.get_i16())),
                _ => Err(()),
            },
            Err(_) => Err(()),
        };

        Ok(Protocol {
            header: ApplicationHeader {
                version: ver.unwrap(),
                application: app.unwrap(),
            },
        })
    }
}

pub(super) struct Connection {
    stream: TcpStream,
    buffer: bytes::BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream,
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    pub async fn read_frame(&mut self) {
        loop {
            match self.stream.read_buf(&mut self.buffer).await {
                Ok(0) => {
                    info!("Connection closed");
                    break;
                }
                Ok(_) => {
                    while self.buffer.has_remaining() {
                        match proto::parse(&mut self.buffer) {
                            Ok(proto) => println!("Proto {:?}", proto),
                            Err(_) => eprintln!("Failed to parse packet"),
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    println!("Err {}", e);
                }
            }
        }
    }
}
