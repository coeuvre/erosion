#![crate_name = "erosion"]
#![crate_type = "lib"]

#![feature(slicing_syntax)]
#![feature(phase)]

#![allow(dead_code)]

#[phase(plugin, link)]
extern crate log;

pub mod config;
pub mod member;
pub mod membership;
pub mod message;
pub mod gossip;
