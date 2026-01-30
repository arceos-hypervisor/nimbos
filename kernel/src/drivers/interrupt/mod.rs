// GICv3 is only for aarch64: error if gicv3 enabled on other arches
#[cfg(all(feature = "gicv3", not(target_arch = "aarch64")))]
compile_error!("GICv3 is only supported for ARCH=aarch64");

cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        mod apic;
        mod i8259_pic;
        use apic as imp;
        pub use apic::local_apic;
        #[allow(unused_imports)]
        pub use apic::vectors::*;
    } else if #[cfg(target_arch = "aarch64")] {
        #[cfg(not(feature = "gicv3"))]
        mod gicv2;
        #[cfg(not(feature = "gicv3"))]
        use gicv2 as imp;
        #[cfg(feature = "gicv3")]
        mod gicv3;
        #[cfg(feature = "gicv3")]
        use gicv3 as imp;
    } else if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
        mod riscv_intc;
        use riscv_intc as imp;
        pub use riscv_intc::ScauseIntCode;
    }
}

pub use self::imp::handle_irq;

#[allow(unused_imports)]
pub(super) use self::imp::{init, register_handler, set_enable};
