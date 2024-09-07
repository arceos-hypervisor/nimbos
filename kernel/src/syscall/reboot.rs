pub const REBOOT_MAGIC1: usize = 0xfee1dead;
pub const REBOOT_MAGIC2: usize = 672274793; // our respect to Linus Torvalds
pub const REBOOT_MAGIC2A: usize = 0x52564d21; // "RVM!"
pub const REBOOT_MAGIC2B: usize = 0x4e696d62; // "Nimb"

pub const REBOOT_CMD_RESTART: usize = 0x5265644f; // "RedO"
pub const REBOOT_CMD_HALT: usize = 0x46725a6e; // "FrZn"
pub const REBOOT_CMD_POWER_OFF: usize = 0x6f387333; // "o8s3"

pub fn sys_reboot(magic1: usize, magic2: usize, cmd: usize) -> isize {
    if magic1 != REBOOT_MAGIC1
        || (magic2 != REBOOT_MAGIC2 && magic2 != REBOOT_MAGIC2A && magic2 != REBOOT_MAGIC2B)
    {
        return -1;
    }

    match cmd {
        REBOOT_CMD_RESTART => {
            info!("Rebooting...");
            unimplemented!("Reboot is not implemented");
        }
        REBOOT_CMD_HALT => {
            info!("Halting...");
            unimplemented!("Halt is not implemented");
        }
        REBOOT_CMD_POWER_OFF => {
            info!("Powering off...");
            crate::platform::power_off();
            unreachable!();
        }
        _ => {
            error!("Unsupported reboot command: {:#x}", cmd);
            return -1;
        }
    }
}
