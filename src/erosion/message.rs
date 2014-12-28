use std::io::{
    IoError,
    IoErrorKind,
    IoResult,
    Writer,
};
use std::io::net::ip::SocketAddr;

#[repr(u8)]
#[deriving(Copy, FromPrimitive)]
pub enum MessageType {
    Ping = 0,
    IndirectPing,
    Ack,
}

#[deriving(Show)]
pub enum Message {
    // Name is sent so the target can verify they are the intended recipient.
    // This is to protect again a node restart with a new name.
    Ping {
        seq: u32,
        name: String
    },

    IndirectPing {
        addr: SocketAddr,
        seq: u32,
        name: String,
    },

    Ack {
        seq: u32,
    },

    Suspect,
    Alive,
    Dead,
    None,
}

impl Message {
    pub fn write<W: Writer>(&self, writer: &mut W) -> IoResult<()> {
        match self {
            &Message::Ping {
                ref seq,
                ref name,
            } => {
                if let Err(e) = writer.write_u8(MessageType::Ping as u8) {
                    return Err(e);
                }
                if let Err(e) = writer.write_be_u32(*seq) {
                    return Err(e);
                }
                if let Err(e) = write_str(writer, name.as_slice()) {
                    return Err(e);
                }
                Ok(())
            },

            &Message::Ack {
                ref seq,
            } => {
                if let Err(e) = writer.write_u8(MessageType::Ack as u8) {
                    return Err(e);
                }
                if let Err(e) = writer.write_be_u32(*seq) {
                    return Err(e);
                }
                Ok(())
            },

            _ => {
                Err(IoError {
                    kind: IoErrorKind::IoUnavailable,
                    desc: "Message not supported",
                    detail: None,
                })
            }
        }
    }

    pub fn read<R: Reader>(reader: &mut R) -> IoResult<Message> {
        let result = reader.read_u8();
        if let Err(e) = result {
            return Err(e);
        }
        let message_type: Option<MessageType> = FromPrimitive::from_u8(result.unwrap());
        if message_type.is_none() {
            return Err(IoError {
                    kind: IoErrorKind::IoUnavailable,
                    desc: "Message not supported",
                    detail: None,
            });
        }

        match message_type.unwrap() {
            MessageType::Ping => {
                let seq = reader.read_be_u32();
                if let Err(e) = seq {
                    return Err(e);
                }

                let name = read_str(reader);
                if let Err(e) = name {
                    return Err(e);
                }

                Ok(Message::Ping {
                    seq: seq.unwrap(),
                    name: name.unwrap(),
                })
            },

            MessageType::Ack => {
                let seq = reader.read_be_u32();
                if let Err(e) = seq {
                    return Err(e);
                }

                Ok(Message::Ack {
                    seq: seq.unwrap(),
                })
            },

            _ => Ok(Message::None)
        }
    }
}

fn write_str<W: Writer>(writer: &mut W, msg: &str) -> IoResult<()> {
    let len = msg.len().to_u8();
    if let None = len {
        return Err(IoError {
            kind: IoErrorKind::InvalidInput,
            desc: "Message is too long",
            detail: None,
        });
    }

    if let Err(e) = writer.write_u8(len.unwrap()) {
        return Err(e);
    }

    if let Err(e) = writer.write_str(msg) {
        return Err(e);
    }

    Ok(())
}

fn read_str<R: Reader>(reader: &mut R) -> IoResult<String> {
    let len = reader.read_u8();
    if let Err(e) = len {
        return Err(e);
    }
    match reader.read_exact(len.unwrap() as uint) {
        Ok(msg) => {
            match String::from_utf8(msg) {
                Ok(msg) => Ok(msg),
                Err(_) => Err(IoError {
                    kind: IoErrorKind::InvalidInput,
                    desc: "Not a valid UTF8 string",
                    detail: None,
                }),
            }
        },
        Err(e) => Err(e),
    }
}
