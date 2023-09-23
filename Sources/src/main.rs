// Todo : remove this unused code ignore macro once the project will start
#![allow(unused)]

use std::fmt::Error;
use crate::libs::utils::u8_to_vec_be;

mod tests;
mod enums;
mod datagrams;
mod libs;

fn main() -> Result<(), Error> {
    println!("Hello, world!");

    Ok(())
}
