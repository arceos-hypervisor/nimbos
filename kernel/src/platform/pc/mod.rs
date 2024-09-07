#[cfg(not(test))]
mod multiboot;

pub fn power_off() -> ! {
    unsafe {
        // This is the QEMU power down command
        x86::io::outw(0x604, 0x2000);
    }
    unreachable!();
}
