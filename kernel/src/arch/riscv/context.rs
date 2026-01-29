use core::arch::{asm, naked_asm};

use riscv::register::{sepc, sscratch};

use crate::arch::instructions;
use crate::mm::{PhysAddr, VirtAddr};

// include_asm_marcos! manually expanded here because macros defined there seems
// to be not visible to naked_asm!

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct GeneralRegisters {
    pub ra: usize,
    pub sp: usize,
    pub gp: usize, // only valid for user traps
    pub tp: usize, // only valid for user traps
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub s0: usize,
    pub s1: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct TrapFrame {
    pub regs: GeneralRegisters,
    pub sepc: usize,
    pub sstatus: usize,
}

impl TrapFrame {
    pub fn new_user(entry: VirtAddr, ustack_top: VirtAddr, arg0: usize) -> Self {
        const SPIE: usize = 1 << 5;
        const SUM: usize = 1 << 18;
        Self {
            regs: GeneralRegisters {
                a0: arg0,
                sp: ustack_top.as_usize(),
                ..Default::default()
            },
            sepc: entry.as_usize(),
            sstatus: SPIE | SUM,
        }
    }

    pub const fn new_clone(&self, ustack_top: VirtAddr) -> Self {
        let mut tf = *self;
        tf.regs.sp = ustack_top.as_usize();
        tf.regs.a0 = 0; // for child thread, clone returns 0
        tf
    }

    pub const fn new_fork(&self) -> Self {
        let mut tf = *self;
        tf.regs.a0 = 0; // for child process, fork returns 0
        tf
    }

    pub unsafe fn exec(&self, kstack_top: VirtAddr) -> ! {
        info!(
            "user task start: entry={:#x}, ustack={:#x}, kstack={:#x}",
            self.sepc,
            self.regs.sp,
            kstack_top.as_usize(),
        );
        instructions::disable_irqs();
        sscratch::write(kstack_top.as_usize());
        sepc::write(self.sepc);
        let kernel_tp_addr = kstack_top.as_usize() - core::mem::size_of::<TrapFrame>()
            + memoffset::offset_of!(GeneralRegisters, tp);
        #[cfg(target_arch = "riscv64")]
        asm!(
            "
            mv      sp, {tf}
            ld      t0, 32*{xlenb}(sp)
            csrw    sstatus, t0
            sd      tp, 0({kernel_tp_addr})
            ld      gp, 2*{xlenb}(sp)
            ld      tp, 3*{xlenb}(sp)
            ld      ra, 0*{xlenb}(sp)
            ld      t0, 4*{xlenb}(sp)
            ld      t1, 5*{xlenb}(sp)
            ld      t2, 6*{xlenb}(sp)
            ld      s0, 7*{xlenb}(sp)
            ld      s1, 8*{xlenb}(sp)
            ld      a0, 9*{xlenb}(sp)
            ld      a1, 10*{xlenb}(sp)
            ld      a2, 11*{xlenb}(sp)
            ld      a3, 12*{xlenb}(sp)
            ld      a4, 13*{xlenb}(sp)
            ld      a5, 14*{xlenb}(sp)
            ld      a6, 15*{xlenb}(sp)
            ld      a7, 16*{xlenb}(sp)
            ld      s2, 17*{xlenb}(sp)
            ld      s3, 18*{xlenb}(sp)
            ld      s4, 19*{xlenb}(sp)
            ld      s5, 20*{xlenb}(sp)
            ld      s6, 21*{xlenb}(sp)
            ld      s7, 22*{xlenb}(sp)
            ld      s8, 23*{xlenb}(sp)
            ld      s9, 24*{xlenb}(sp)
            ld      s10, 25*{xlenb}(sp)
            ld      s11, 26*{xlenb}(sp)
            ld      t3, 27*{xlenb}(sp)
            ld      t4, 28*{xlenb}(sp)
            ld      t5, 29*{xlenb}(sp)
            ld      t6, 30*{xlenb}(sp)
            ld      sp, 1*{xlenb}(sp)
            sret"
           ,
            tf = in(reg) self,
            kernel_tp_addr = in(reg) kernel_tp_addr,
            xlenb = const 8,
            options(noreturn),
        );
        #[cfg(target_arch = "riscv32")]
        asm!(
            "
            mv      sp, {tf}
            lw      t0, 32*{xlenb}(sp)
            csrw    sstatus, t0
            sw      tp, 0({kernel_tp_addr})
            lw      gp, 2*{xlenb}(sp)
            lw      tp, 3*{xlenb}(sp)
            lw      ra, 0*{xlenb}(sp)
            lw      t0, 4*{xlenb}(sp)
            lw      t1, 5*{xlenb}(sp)
            lw      t2, 6*{xlenb}(sp)
            lw      s0, 7*{xlenb}(sp)
            lw      s1, 8*{xlenb}(sp)
            lw      a0, 9*{xlenb}(sp)
            lw      a1, 10*{xlenb}(sp)
            lw      a2, 11*{xlenb}(sp)
            lw      a3, 12*{xlenb}(sp)
            lw      a4, 13*{xlenb}(sp)
            lw      a5, 14*{xlenb}(sp)
            lw      a6, 15*{xlenb}(sp)
            lw      a7, 16*{xlenb}(sp)
            lw      s2, 17*{xlenb}(sp)
            lw      s3, 18*{xlenb}(sp)
            lw      s4, 19*{xlenb}(sp)
            lw      s5, 20*{xlenb}(sp)
            lw      s6, 21*{xlenb}(sp)
            lw      s7, 22*{xlenb}(sp)
            lw      s8, 23*{xlenb}(sp)
            lw      s9, 24*{xlenb}(sp)
            lw      s10, 25*{xlenb}(sp)
            lw      s11, 26*{xlenb}(sp)
            lw      t3, 27*{xlenb}(sp)
            lw      t4, 28*{xlenb}(sp)
            lw      t5, 29*{xlenb}(sp)
            lw      t6, 30*{xlenb}(sp)
            lw      sp, 1*{xlenb}(sp)
            sret"
            ,
            tf = in(reg) self,
            kernel_tp_addr = in(reg) kernel_tp_addr,
            xlenb = const 4,
            options(noreturn),
        );
    }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct TaskContext {
    pub ra: usize, // return address (x1)
    pub sp: usize, // stack pointer (x2)

    pub s0: usize, // x8-x9
    pub s1: usize,

    pub s2: usize, // x18-x27
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,

    pub satp: usize,
}

impl TaskContext {
    pub const fn default() -> Self {
        unsafe { core::mem::MaybeUninit::zeroed().assume_init() }
    }

    pub fn init(
        &mut self,
        entry: usize,
        kstack_top: VirtAddr,
        page_table_root: PhysAddr,
        _is_kernel: bool,
    ) {
        self.sp = kstack_top.as_usize();
        self.ra = entry;
        self.satp = page_table_root.as_usize();
    }

    pub fn switch_to(&mut self, next_ctx: &Self) {
        unsafe {
            instructions::set_user_page_table_root(next_ctx.satp);
            context_switch(self, next_ctx)
        }
    }
}

#[unsafe(naked)]
unsafe extern "C" fn context_switch(_current_task: &mut TaskContext, _next_task: &TaskContext) {
    #[cfg(target_arch = "riscv64")]
    naked_asm!(
        "
        sd      ra, 0*{xlenb}(a0)
        sd      sp, 1*{xlenb}(a0)
        sd      s0, 2*{xlenb}(a0)
        sd      s1, 3*{xlenb}(a0)
        sd      s2, 4*{xlenb}(a0)
        sd      s3, 5*{xlenb}(a0)
        sd      s4, 6*{xlenb}(a0)
        sd      s5, 7*{xlenb}(a0)
        sd      s6, 8*{xlenb}(a0)
        sd      s7, 9*{xlenb}(a0)
        sd      s8, 10*{xlenb}(a0)
        sd      s9, 11*{xlenb}(a0)
        sd      s10, 12*{xlenb}(a0)
        sd      s11, 13*{xlenb}(a0)
        ld      s11, 13*{xlenb}(a1)
        ld      s10, 12*{xlenb}(a1)
        ld      s9, 11*{xlenb}(a1)
        ld      s8, 10*{xlenb}(a1)
        ld      s7, 9*{xlenb}(a1)
        ld      s6, 8*{xlenb}(a1)
        ld      s5, 7*{xlenb}(a1)
        ld      s4, 6*{xlenb}(a1)
        ld      s3, 5*{xlenb}(a1)
        ld      s2, 4*{xlenb}(a1)
        ld      s1, 3*{xlenb}(a1)
        ld      s0, 2*{xlenb}(a1)
        ld      sp, 1*{xlenb}(a1)
        ld      ra, 0*{xlenb}(a1)
        ret",
        xlenb = const 8,
    );
    #[cfg(target_arch = "riscv32")]
    naked_asm!(
        "
        sw      ra, 0*{xlenb}(a0)
        sw      sp, 1*{xlenb}(a0)
        sw      s0, 2*{xlenb}(a0)
        sw      s1, 3*{xlenb}(a0)
        sw      s2, 4*{xlenb}(a0)
        sw      s3, 5*{xlenb}(a0)
        sw      s4, 6*{xlenb}(a0)
        sw      s5, 7*{xlenb}(a0)
        sw      s6, 8*{xlenb}(a0)
        sw      s7, 9*{xlenb}(a0)
        sw      s8, 10*{xlenb}(a0)
        sw      s9, 11*{xlenb}(a0)
        sw      s10, 12*{xlenb}(a0)
        sw      s11, 13*{xlenb}(a0)
        lw      s11, 13*{xlenb}(a1)
        lw      s10, 12*{xlenb}(a1)
        lw      s9, 11*{xlenb}(a1)
        lw      s8, 10*{xlenb}(a1)
        lw      s7, 9*{xlenb}(a1)
        lw      s6, 8*{xlenb}(a1)
        lw      s5, 7*{xlenb}(a1)
        lw      s4, 6*{xlenb}(a1)
        lw      s3, 5*{xlenb}(a1)
        lw      s2, 4*{xlenb}(a1)
        lw      s1, 3*{xlenb}(a1)
        lw      s0, 2*{xlenb}(a1)
        lw      sp, 1*{xlenb}(a1)
        lw      ra, 0*{xlenb}(a1)
        ret",
        xlenb = const 4,
    );
}
