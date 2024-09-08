#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{power_off};

#[no_mangle]
pub fn main() -> i32 {
    println!("powering off!");
    power_off();
    0
}
