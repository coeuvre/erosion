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

static EROSION_ADDR: SocketAddr = SocketAddr {
    ip: Ipv4Addr(127, 0, 0, 1),
    port: 7201,
};

fn bind() -> (int, Membership) {
    let mut config = config::local("node1".to_string());
    let mut index = 1;
    config.bind_addr = EROSION_ADDR;

    loop {
        match Membership::bind(config.clone()) {
            Ok(membership) => return (index, membership),
            Err(e) => {
                println!("{}", e);

                index += 1;
                config.name = format!("node{}", index);
                config.bind_addr.port += 1;
            },
        };
    }
}

fn ping(index: int, mut membership: Membership) {
    match index {
        1 => {
            println!("First member, start!");
            membership.start();
        },
        _ => {
            println!("Join to {}", EROSION_ADDR);
            membership.join("node1".to_string(), EROSION_ADDR);
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
