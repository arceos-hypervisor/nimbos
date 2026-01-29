//! ARM Generic Interrupt Controller v2.

#![allow(dead_code)]

use core::u32;

use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};

use crate::mm::{PhysAddr, VirtAddr};
use crate::sync::LazyInit;
use crate::utils::irq_handler::{IrqHandler, IrqHandlerTable};

const GIC_BASE: usize = 0x0800_0000;
const GICD_BASE: PhysAddr = PhysAddr::new(GIC_BASE);
const GICR_BASE: PhysAddr = PhysAddr::new(GIC_BASE + 0xa0000);

const PPI_BASE: usize = 16;
const SPI_BASE: usize = 32;

const IRQ_COUNT: usize = 1024;

static GIC: LazyInit<Gic> = LazyInit::new();
static HANDLERS: IrqHandlerTable<IRQ_COUNT> = IrqHandlerTable::new();

register_structs! {
    #[allow(non_snake_case)]
    GicDistributorRegs {
        /// Distributor Control Register.
        (0x0000 => CTLR: ReadWrite<u32>),
        /// Interrupt Controller Type Register.
        (0x0004 => TYPER: ReadOnly<u32>),
        /// Distributor Implementer Identification Register.
        (0x0008 => IIDR: ReadOnly<u32>),
        (0x000c => _reserved_0),
        /// Interrupt Group Registers.
        (0x0080 => IGROUPR: [ReadWrite<u32>; 0x20]),
        /// Interrupt Set-Enable Registers.
        (0x0100 => ISENABLER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Clear-Enable Registers.
        (0x0180 => ICENABLER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Set-Pending Registers.
        (0x0200 => ISPENDR: [ReadWrite<u32>; 0x20]),
        /// Interrupt Clear-Pending Registers.
        (0x0280 => ICPENDR: [ReadWrite<u32>; 0x20]),
        /// Interrupt Set-Active Registers.
        (0x0300 => ISACTIVER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Clear-Active Registers.
        (0x0380 => ICACTIVER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Priority Registers.
        (0x0400 => IPRIORITYR: [ReadWrite<u32>; 0x100]),
        /// Interrupt Processor Targets Registers.
        (0x0800 => ITARGETSR: [ReadWrite<u32>; 0x100]),
        /// Interrupt Configuration Registers.
        (0x0c00 => ICFGR: [ReadWrite<u32>; 0x40]),
        (0x0d00 => _reserved_1),
        /// Software Generated Interrupt Register.
        (0x0f00 => SGIR: WriteOnly<u32>),
        (0x1000 => @END),
    }
}

register_structs! {
    #[allow(non_snake_case)]
    GicRedistributorRegs {
        /// Redistributor Control Register.
        (0x0000 => CTLR: ReadWrite<u32>),
        /// Implementer Identification Register.
        (0x0004 => IIDR: ReadOnly<u32>),
        /// Redistributor Type Register.
        (0x0008 => TYPER: ReadOnly<u64>),
        /// Error Reporting Status Register.
        (0x0010 => STATUSR: ReadWrite<u32>),
        /// Redistributor Wake Register.
        (0x0014 => WAKER: ReadWrite<u32>),
        (0x0018 => _reserved_0),
        /// Interrupt Group Register 0.
        (0x10080 => IGROUPR0: ReadWrite<u32>),
        (0x10084 => _reserved_1),
        /// Interrupt Set-Enable Register 0.
        (0x10100 => ISENABLER0: ReadWrite<u32>),
        (0x10104 => _reserved_2),
        /// Interrupt Clear-Enable Register 0.
        (0x10180 => ICENABLER0: ReadWrite<u32>),
        (0x10184 => _reserved_3),
        /// Interrupt Set-Pending Register 0.
        (0x10200 => ISPENDR0: ReadWrite<u32>),
        (0x10204 => _reserved_4),
        /// Interrupt Clear-Pending Register 0.
        (0x10280 => ICPENDR0: ReadWrite<u32>),
        (0x10284 => _reserved_5),
        /// Interrupt Set-Active Register 0.
        (0x10300 => ISACTIVER0: ReadWrite<u32>),
        (0x10304 => _reserved_6),
        /// Interrupt Clear-Active Register 0.
        (0x10380 => ICACTIVER0: ReadWrite<u32>),
        (0x10384 => _reserved_7),
        /// Interrupt Priority Registers. ARM says these are byte-accessible.
        (0x10400 => IPRIORITYR: [ReadWrite<u8>; 0x20]),
        (0x10420 => _reserved_8),
        /// Interrupt Configuration Registers.
        (0x10C00 => ICFGR: [ReadWrite<u32>; 2]),
        (0x10C08 => _reserved_9),
        /// Interrupt Group Modifier Register 0.
        (0x10D00 => IGRPMODR: ReadWrite<u32>),
        (0x10D04 => _reserved_10),
        (0x20000 => @END),
    }
}

enum TriggerMode {
    Edge = 0,
    Level = 1,
}

enum Polarity {
    ActiveHigh = 0,
    ActiveLow = 1,
}

struct Gic {
    gicd_base: VirtAddr,
    gicr_base: VirtAddr,
    max_irqs: usize,
}

impl Gic {
    fn new(gicd_base: VirtAddr, gicr_base: VirtAddr) -> Self {
        let mut gic = Self {
            gicd_base,
            gicr_base,
            max_irqs: 0,
        };
        gic.max_irqs = ((gic.gicd().TYPER.get() as usize & 0b11111) + 1) * 32;
        gic
    }

    const fn gicd(&self) -> &GicDistributorRegs {
        unsafe { &*(self.gicd_base.as_ptr() as *const _) }
    }

    const fn gicr(&self) -> &GicRedistributorRegs {
        unsafe { &*(self.gicr_base.as_ptr() as *const _) }
    }

    fn cpu_num(&self) -> usize {
        ((self.gicd().TYPER.get() as usize >> 5) & 0b111) + 1
    }

    fn configure_interrupt(&self, vector: usize, tm: TriggerMode, pol: Polarity) {
        // Only configurable for SPI interrupts
        assert!(vector < self.max_irqs);
        assert!(vector >= SPI_BASE);
        // TODO: polarity should actually be configure through a GPIO controller
        assert!(matches!(pol, Polarity::ActiveHigh));

        // type is encoded with two bits, MSB of the two determine type
        // 16 irqs encoded per ICFGR register
        let reg_ndx = vector >> 4;
        let bit_shift = ((vector & 0xf) << 1) + 1;
        let mut reg_val = self.gicd().ICFGR[reg_ndx].get();
        match tm {
            TriggerMode::Edge => reg_val |= 1 << bit_shift,
            TriggerMode::Level => reg_val &= !(1 << bit_shift),
        }
        self.gicd().ICFGR[reg_ndx].set(reg_val);
    }

    fn set_enable(&self, vector: usize, enable: bool) {
        assert!(vector < self.max_irqs);

        warn!("set_enable: vector = {}, enable = {}", vector, enable);

        if vector >= SPI_BASE {
            let reg = vector / 32;
            let mask = 1 << (vector % 32);
            if enable {
                self.gicd().ISENABLER[reg].set(mask);
            } else {
                self.gicd().ICENABLER[reg].set(mask);
            }
        } else {
            if enable {
                self.gicr().ISENABLER0.set(1 << vector);
            } else {
                self.gicr().ICENABLER0.set(1 << vector);
            }
        }
    }

    fn pending_irq(&self) -> Option<usize> {
        let iar: u64;
        unsafe {
            core::arch::asm!(
                "mrs {}, ICC_IAR1_EL1",
                out(reg) iar,
                options(nomem, nostack, preserves_flags)
            );
        }

        if iar >= 0x3fe {
            // spurious
            None
        } else {
            Some(iar as _)
        }
    }

    fn eoi(&self, vector: usize) {
        unsafe {
            core::arch::asm!(
                "msr ICC_EOIR1_EL1, {}",
                in(reg) vector,
                options(nomem, nostack, preserves_flags)
            );
        }
    }

    fn init_gicd(&self) {
        fn wait_gicd_ctlr(gicd: &GicDistributorRegs) {
            while (gicd.CTLR.get() >> 31) & 1 == 1 {
                // Wait for the distributor to be enabled
            }
        }

        let gicd = self.gicd();
        gicd.CTLR.set(0); // Disable the distributor
        wait_gicd_ctlr(gicd);

        for i in (0..self.max_irqs).step_by(32) {
            gicd.ICENABLER[i / 32].set(u32::MAX);
            gicd.ICPENDR[i / 32].set(u32::MAX);
            gicd.IGROUPR[i / 32].set(u32::MAX); // Set all interrupts to Group 1
        }

        for i in (0..self.max_irqs).step_by(16) {
            gicd.ICFGR[i / 16].set(0); // Set all interrupts to edge triggered
        }

        for i in (0..self.max_irqs).step_by(4) {
            // Set external interrupts to target cpu 0
            gicd.IPRIORITYR[i / 4].set(0xa0);
        }

        wait_gicd_ctlr(gicd);

        gicd.CTLR.set(0x52); // bit 6: DS, bit 4: Affinity routing, bit 1: Enable Group 1 interrupts

        wait_gicd_ctlr(gicd);
    }

    fn init_gicr(&self) {
        let gicr = self.gicr();

        gicr.WAKER.set(0); // Wake up the redistributor
        while gicr.WAKER.get() & 0x4 != 0 {
            // Wait for the `ChildrenAsleep` bit to be cleared
        }
        
        gicr.ICENABLER0.set(u32::MAX); // Disable all interrupts
        gicr.ICPENDR0.set(u32::MAX); // Clear all pending interrupts
        gicr.IGROUPR0.set(u32::MAX); // Set all interrupts to Group 1
        gicr.IGRPMODR.set(u32::MAX); // Set all interrupts to Group 1

        unsafe {
            core::arch::asm!(
                "msr icc_sre_el1, {icc_sre_el1:x}",
                "msr icc_pmr_el1, {icc_pmr_el1:x}",
                "msr icc_igrpen1_el1, {icc_igrpen1_el1:x}",
                "msr icc_ctlr_el1, {icc_ctlr_el1:x}",
                icc_sre_el1 = in(reg) 0x7,          // DIB | DFB | SRE
                icc_pmr_el1 = in(reg) 0xff,         // Unmask all interrupts
                icc_igrpen1_el1 = in(reg) 0x1,      // Enable Group 1 interrupts
                icc_ctlr_el1 = in(reg) 0x1,         // Enable the GICR
            );
        }
    }

    fn init(&self) {
        let gicd = self.gicd();
        // let gicr = self.gicr();

        if self.cpu_num() > 1 {
            for i in (SPI_BASE..self.max_irqs).step_by(4) {
                // Set external interrupts to target cpu 0
                gicd.ITARGETSR[i / 4].set(0x01_01_01_01);
            }
        }
        // Initialize all the SPIs to edge triggered
        for i in SPI_BASE..self.max_irqs {
            self.configure_interrupt(i, TriggerMode::Edge, Polarity::ActiveHigh);
        }

        // enable GIC
        self.init_gicd();
        self.init_gicr();
    }
}

pub fn set_enable(vector: usize, enable: bool) {
    GIC.set_enable(vector, enable);
}

pub fn handle_irq(_vector: usize) {
    if let Some(vector) = GIC.pending_irq() {
        HANDLERS.handle(vector);
        GIC.eoi(vector);
    }
}

pub fn register_handler(vector: usize, handler: IrqHandler) {
    HANDLERS.register_handler(vector, handler);
}

pub fn init() {
    let gic = Gic::new(GICD_BASE.into_kvaddr(), GICR_BASE.into_kvaddr());
    gic.init();
    GIC.init_by(gic);
}
