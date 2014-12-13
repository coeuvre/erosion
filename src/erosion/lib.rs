#![crate_name = "erosion"]
#![crate_type = "lib"]

#![feature(slicing_syntax)]

#![allow(dead_code)]

extern crate serialize;

pub mod config;
pub mod member;
pub mod membership;
pub mod message;
pub mod gossip;
