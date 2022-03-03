#![no_std]
#![no_main]
#![feature(asm_sym)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(const_maybe_uninit_zeroed)]

extern crate alloc;

#[macro_use]
mod console;
mod arch;
mod config;
mod entry;
mod gicv2;
mod lang_items;
mod loader;
mod mm;
mod pl011;
mod psci;
mod sync;
mod syscall;
mod task;
mod timer;
mod trap;

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

pub fn rust_main() -> ! {
    clear_bss();
    println!("[kernel] Hello, world!");
    mm::init();
    println!("[kernel] back to world!");
    trap::init();

    gicv2::init();
    timer::init();

    task::init();
    task::run();
}
