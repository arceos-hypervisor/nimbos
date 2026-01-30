mod context;
mod gdt;
mod idt;
mod page_table;
mod percpu;
mod syscall;
mod trap;

pub mod config;
pub mod instructions;

pub use self::context::{TaskContext, TrapFrame};
#[allow(unused_imports)]
pub use self::page_table::{PageTable, PageTableEntry};
pub use self::percpu::ArchPerCpu;

pub fn init() {
    idt::init();
}

pub fn init_percpu() {
    idt::IDT.load();
    syscall::init_percpu();
}
