use std::io::net::ip::SocketAddr;
use std::io::net::udp::UdpSocket;

use message::Message;

pub const UDP_MAX_SIZE: uint = 548;

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

    pub fn ping(&mut self, seq: u32, name: String, to: SocketAddr) {
        let msg = Message::Ping {
            seq: seq,
            name: name,
        };
        let mut buf = Vec::new();
        if let Err(e) = msg.write(&mut buf) {
            println!("Failed to encode message. Err: {}", e);
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
            println!("Failed to encode message. Err: {}", e);
            return;
        }
        self.send_to(buf.as_slice(), to);
    }

    fn send_to(&mut self, buf: &[u8], to: SocketAddr) {
        if buf.len() > UDP_MAX_SIZE {
            println!("Failed to send message. Message is too long ({} bytes).",
                     buf.len());
            return;
        }

        println!("Sending message to {} <= {}", to, buf);
        if let Err(e) = self.udp.send_to(buf, to) {
            println!("Failed to send packets to {}. Err: {}", to, e);
        }
    }
}
