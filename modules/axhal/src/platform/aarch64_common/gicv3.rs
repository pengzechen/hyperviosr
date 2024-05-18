

use arm_gicv3::gich_lrs_num;
pub use arm_gicv3::GICD;
pub use arm_gicv3::GICR;
pub use arm_gicv3::GICC;
pub use arm_gicv3::GICH;
use arm_gicv3::GICC_IAR_ID_OFF;
use arm_gicv3::GICC_IAR_ID_LEN;
use arm_gicv3::gic_is_priv;
use arm_gicv3::bit_extract;
use arm_gicv3::GIC_LRS_NUM;

pub use arm_gicv3::GIC_INTS_MAX as MAX_IRQ_COUNT;

use crate::cpu::this_cpu_id;

use core::sync::atomic::Ordering;

use crate::{irq::IrqHandler};

/* ====== InterFace ====== */

/// Initializes GICD, GICC on the primary CPU.
pub(crate) fn init_primary() {}

/// Enables or disables the given IRQ.
pub fn set_enable(irq_num: usize, enabled: bool) {}


/// Registers an IRQ handler for the given IRQ.
///
/// It also enables the IRQ if the registration succeeds. It returns `false` if
/// the registration failed.
pub fn register_handler(irq_num: usize, handler: IrqHandler) -> bool {true}

pub fn dispatch_irq(_unused: usize) {}

/* ====== InterFace ====== */


pub fn gic_glb_init() {
    set_gic_lrs(gich_lrs_num());
    GICD.global_init();
}

pub fn gic_cpu_init() {
    GICR.init(this_cpu_id());
    GICC.init();
}

pub fn gic_cpu_reset() {
    GICC.init();
}


/* =============== warn! ===================== 
   ========   current_cpu().current_irq   ====
   ===========================================
*/
// pub fn gicc_clear_current_irq(for_hypervisor: bool) {
//     let irq = current_cpu().current_irq as u32;
//     if irq == 0 {
//         return;
//     }
//     GICC.set_eoir(irq);
//     if for_hypervisor {
//         GICC.set_dir(irq);
//     }
//     current_cpu().current_irq = 0;
// }

// pub fn gicc_get_current_irq() -> Option<usize> {
//     let iar = GICC.iar();
//     let irq = iar as usize;
//     current_cpu().current_irq = irq;
//     let id = bit_extract(iar as usize, GICC_IAR_ID_OFF, GICC_IAR_ID_LEN);
//     if id >= 1024 {   // IntCtrl::NUM_MAX
//         None
//     } else {
//         Some(id)
//     }
// }


// Isn't right ?
// remove current_cpu().current_irq, return (usize, usize)
pub fn gicc_get_current_irq() -> (usize, usize) {
    let iar = GICC.iar();
    let irq = iar as usize;
    // debug!("this is iar:{:#x}", iar);
    // current_cpu().current_irq = irq;
    let irq = bit_extract(irq, 0, 10);
    let src = bit_extract(irq, 10, 3);
    (irq, src)
}

// remove current_cpu().current_irq ,add an argument
pub fn gicc_clear_current_irq( irq: u32, for_hypervisor: bool) {
    // let irq = current_cpu().current_irq as u32;
    if irq == 0 {
        return;
    }
    GICC.set_eoir(irq);
    if for_hypervisor {
        GICC.set_dir(irq);
    }
    // current_cpu().current_irq = 0;
}

pub fn gic_set_icfgr(int_id: usize, cfg: u8) {
    if !gic_is_priv(int_id) {
        GICD.set_icfgr(int_id, cfg);
    } else {
        GICR.set_icfgr(int_id, cfg, this_cpu_id() as u32);
    }
}

pub fn gic_lrs() -> usize {
    GIC_LRS_NUM.load(Ordering::Relaxed)
}

pub fn set_gic_lrs(lrs: usize) {
    GIC_LRS_NUM.store(lrs, Ordering::Relaxed);
}

pub fn gic_set_act(int_id: usize, act: bool, gicr_id: u32) {
    if !gic_is_priv(int_id) {
        GICD.set_act(int_id, act);
    } else {
        GICR.set_act(int_id, act, gicr_id);
    }
}

pub fn gic_set_pend(int_id: usize, pend: bool, gicr_id: u32) {
    if !gic_is_priv(int_id) {
        GICD.set_pend(int_id, pend);
    } else {
        GICR.set_pend(int_id, pend, gicr_id);
    }
}

pub fn gic_get_pend(int_id: usize) -> bool {
    if !gic_is_priv(int_id) {
        GICD.get_pend(int_id)
    } else {
        GICR.get_pend(int_id, this_cpu_id() as u32)
    }
}

pub fn gic_get_act(int_id: usize) -> bool {
    if !gic_is_priv(int_id) {
        GICD.get_act(int_id)
    } else {
        GICR.get_act(int_id, this_cpu_id() as u32)
    }
}

pub fn gic_set_enable(int_id: usize, en: bool) {
    if !gic_is_priv(int_id) {
        GICD.set_enable(int_id, en);
    } else {
        GICR.set_enable(int_id, en, this_cpu_id() as u32);
    }
}

pub fn gic_get_prio(int_id: usize) {
    if !gic_is_priv(int_id) {
        GICD.prio(int_id);
    } else {
        GICR.get_prio(int_id, this_cpu_id() as u32);
    }
}

pub fn gic_set_prio(int_id: usize, prio: u8) {
    if !gic_is_priv(int_id) {
        GICD.set_prio(int_id, prio);
    } else {
        GICR.set_prio(int_id, prio, this_cpu_id() as u32);
    }
}
