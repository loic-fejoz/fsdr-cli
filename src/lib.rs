#![feature(core_intrinsics)]
#![allow(internal_features)]

pub extern crate async_trait;
#[macro_use]
extern crate pest_derive;

pub mod audio;
pub mod beacon;
pub mod cat;
pub mod cmd_grammar;
pub mod cmd_line;
pub mod config;
// pub mod csdr;
pub mod blocks;
pub mod csdr_cmd;
pub mod grc;
pub mod grc_cmd;
pub mod iqengine_blockconverter;
pub mod iqengine_cmd;
#[cfg(feature = "iqengine")]
pub mod iqengine_plugin;
pub mod iqengine_userdef;
pub mod math;
pub mod telemetry;

pub fn join(iter: impl Iterator<Item = String>) -> String {
    iter.fold(String::new(), |mut a, b| {
        a.reserve(b.len() + 1);
        a.push(' ');
        a.push_str(&b);
        a
    })
}
