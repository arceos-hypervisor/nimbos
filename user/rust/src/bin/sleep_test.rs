#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{exit, fork, get_time_us, sleep, waitpid};

fn sleepy() {
    let time: usize = 1;
    for i in 0..5 {
        let before = get_time_us();
        sleep(1);
        let after = get_time_us();
        println!("[test]: sleep {} x {} seconds, sleep {} usecs.", i + 1, time, after - before);
        assert!(after - before > 1000000);
        assert!(after - before < 1500000);
    }
    exit(0);
}

#[no_mangle]
pub fn main() -> i32 {
    let current_time = get_time_us();
    let pid = fork();
    let mut exit_code = 0;
    if pid == 0 {
        sleepy();
    }
    assert!(waitpid(pid, Some(&mut exit_code), 0) == pid && exit_code == 0);
    println!("use {} usecs.", get_time_us() - current_time);
    println!("sleep passed!");
    0
}
