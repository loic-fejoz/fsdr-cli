pub extern crate async_trait;
#[macro_use]
extern crate pest_derive;

pub mod blocks;
pub mod cmd_grammar;
pub mod cmd_line;
// pub mod csdr;
pub mod csdr_cmd;
pub mod grc;
pub mod grc_cmd;
pub mod iqengine_blockconverter;
pub mod iqengine_userdef;

pub fn join(iter: impl Iterator<Item = String>) -> String {
    iter.fold(String::new(), |mut a, b| {
        a.reserve(b.len() + 1);
        a.push(' ');
        a.push_str(&b);
        a
    })
}
