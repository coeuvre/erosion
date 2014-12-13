#![feature(slicing_syntax)]

extern crate serialize;
extern crate erosion;

use std::io::net::ip::{
    Ipv4Addr,
    SocketAddr,
};

use serialize::base64::{MIME, ToBase64, FromBase64};

use erosion::membership::Membership;
use erosion::config;

static EROSION_ADDR_1: SocketAddr = SocketAddr {
    ip: Ipv4Addr(127, 0, 0, 1),
    port: 7201,
};
static EROSION_ADDR_2: SocketAddr = SocketAddr {
    ip: Ipv4Addr(127, 0, 0, 1),
    port: 7202,
};

fn bind() -> (int, Membership) {
    let mut config = config::local("node1".to_string());
    config.bind_addr = EROSION_ADDR_1;

    match Membership::bind(config.clone()) {
        Ok(membership) => (1, membership),
        Err(e) => {
            println!("{}", e);

            config.name = "node2".to_string();
            config.bind_addr = EROSION_ADDR_2;

            if let Ok(membership) = Membership::bind(config) {
                (2, membership)
            } else {
                panic!("couldn't bind socket.");
            }
        },
    }
}

fn ping(index: int, mut membership: Membership) {
    match index {
        1 => {
            membership.start();
        },
        _ => {
            membership.join("node1".to_string(), EROSION_ADDR_1);
        },
    }
}

fn main() {
    let msg = "hello,world";
    let encrypted = msg.as_bytes().to_base64(MIME);
    let plain = String::from_utf8(encrypted.from_base64().unwrap()).unwrap();

    println!("{}", encrypted);
    println!("{}", plain);

    let (index, membership) = bind();
    ping(index, membership);
}
