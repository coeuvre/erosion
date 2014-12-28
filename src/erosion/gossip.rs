use std::io::net::ip::SocketAddr;
use std::io::net::udp::UdpSocket;
use std::io::IoResult;

use message::Message;

pub const UDP_MAX_SIZE: uint = 548;

#[deriving(Clone)]
pub struct Gossip {
    pub udp: UdpSocket,
}

impl Gossip {
    pub fn new(addr: SocketAddr) -> Result<Gossip, String> {
        let udp = UdpSocket::bind(addr);
        if let Err(e) = udp {
            return Err(format!("Failed to start UDP listener at {}. Err: {}", addr, e));
        }

        Ok(Gossip {
            udp: udp.unwrap(),
        })
    }

    pub fn recv_from(&mut self) -> IoResult<(Message, SocketAddr)> {
        let mut buf = [0u8, ..UDP_MAX_SIZE];
        let result = self.udp.recv_from(&mut buf);
        if let Err(e) = result {
            return Err(e);
        }

        let (count, from) = result.unwrap();
        let mut buf = buf[..count];
        match Message::read(&mut buf) {
            Ok(msg) => {
                info!("Received message from {} => {}", from, msg);
                Ok((msg, from))
            },
            Err(e) => {
                error!("Failed to decode message from {} => {}", from, e);
                Err(e)
            },
        }
    }

    pub fn ping(&mut self, seq: u32, name: String, to: SocketAddr) {
        let msg = Message::Ping {
            seq: seq,
            name: name,
        };
        let mut buf = Vec::new();
        if let Err(e) = msg.write(&mut buf) {
            error!("Failed to encode message. Err: {}", e);
            return;
        }
        self.send_to(buf.as_slice(),  to);
    }

    pub fn ack (&mut self, seq: u32, to: SocketAddr) {
        let msg = Message::Ack {
            seq: seq,
        };
        let mut buf = Vec::new();
        if let Err(e) = msg.write(&mut buf) {
            error!("Failed to encode message. Err: {}", e);
            return;
        }
        self.send_to(buf.as_slice(), to);
    }

    fn send_to(&mut self, buf: &[u8], to: SocketAddr) {
        if buf.len() > UDP_MAX_SIZE {
            error!("Failed to send message. Message is too long ({} bytes).",
                     buf.len());
            return;
        }

        info!("Sending message to {} <= {}", to, buf);
        if let Err(e) = self.udp.send_to(buf, to) {
            error!("Failed to send packets to {}. Err: {}", to, e);
        }
    }
}
