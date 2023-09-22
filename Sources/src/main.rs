// Todo : remove this unused code ignore macro once the project will start
#![allow(unused)]

use std::fmt::Error;

mod tests;
mod enums;
mod datagrams;
mod libs;

fn main() -> Result<(), Error> {
    println!("Hello, world!");
    Ok(())
}
