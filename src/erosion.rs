#![feature(slicing_syntax)]

extern crate erosion;

use std::io::net::ip::{
    Ipv4Addr,
    SocketAddr,
};
use std::io::timer::sleep;
use std::time::duration::Duration;

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
    let (index, membership) = bind();
    ping(index, membership);

    loop {
        sleep(Duration::seconds(1));
    }
}
