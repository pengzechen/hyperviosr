//! AArch64 VMSAv8-64 translation table format descriptors.

use core::fmt;
use memory_addr::PhysAddr;

use crate::{GenericPTE, MappingFlags};

bitflags::bitflags! {
    /// Memory attribute fields in the VMSAv8-64 translation table format descriptors.
    #[derive(Debug)]
    pub struct DescriptorAttr: u64 {
        /// Whether the descriptor is valid.
        const VALID =       1 << 0;
        /// The descriptor gives the address of the next level of translation table or 4KB page.
        /// (not a 2M, 1G block)
        const NON_BLOCK =   1 << 1;
        /// Memory attributes [2:5]
        const ATTR =   0b1111 << 2;
       /// Access permission: read-only.
        const S2AP_RO =      1 << 6;
        /// Access permission: write-only.
        const S2AP_WO =       1 << 7;
        /// Shareability: Inner Shareable (otherwise Outer Shareable).
        const INNER =       1 << 8;
        /// Shareability: Inner or Outer Shareable (otherwise Non-shareable).
        const SHAREABLE =   1 << 9;
        /// The Access flag.
        const AF =          1 << 10;
        /// The not global bit.
        const NG =          1 << 11;
        /// Indicates that 16 adjacent translation table entries point to contiguous memory regions.
        const CONTIGUOUS =  1 <<  52;
        /// The Privileged execute-never field.
        // const PXN =         1 <<  53;
        /// The Execute-never or Unprivileged execute-never field.
        const XN =         1 <<  54;

        // Next-level attributes in stage 1 VMSAv8-64 Table descriptors:

        /// PXN limit for subsequent levels of lookup.
        const PXN_TABLE =           1 << 59;
        /// XN limit for subsequent levels of lookup.
        const XN_TABLE =            1 << 60;
        /// Access permissions limit for subsequent levels of lookup: access at EL0 not permitted.
        const AP_NO_EL0_TABLE =     1 << 61;
        /// Access permissions limit for subsequent levels of lookup: write access not permitted.
        const AP_NO_WRITE_TABLE =   1 << 62;
        /// For memory accesses from Secure state, specifies the Security state for subsequent
        /// levels of lookup.
        const NS_TABLE =            1 << 63;
    }
}

#[repr(u64)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum MemType {
    Device = 0,
    Normal = 1,
    NormalNonCache = 2,
}

impl DescriptorAttr {
    const ATTR_INDEX_MASK: u64 = 0b1111_00;
    const PTE_S2_MEM_ATTR_NORMAL_INNER_WRITE_BACK_CACHEABLE: u64 = 0b11 << 2;
    const PTE_S2_MEM_ATTR_NORMAL_OUTER_WRITE_BACK_CACHEABLE: u64 = 0b11 << 4;
    const PTE_S2_MEM_ATTR_NORMAL_OUTER_WRITE_BACK_NOCACHEABLE: u64 = 0b1 << 4;

    const fn from_mem_type(mem_type: MemType) -> Self {
        let bits = match mem_type {
            MemType::Normal => 
                Self::PTE_S2_MEM_ATTR_NORMAL_INNER_WRITE_BACK_CACHEABLE
                | Self::PTE_S2_MEM_ATTR_NORMAL_OUTER_WRITE_BACK_CACHEABLE 
                | Self::SHAREABLE.bits(),
            MemType::NormalNonCache =>
                Self::PTE_S2_MEM_ATTR_NORMAL_INNER_WRITE_BACK_CACHEABLE
                | Self::PTE_S2_MEM_ATTR_NORMAL_OUTER_WRITE_BACK_NOCACHEABLE
                | Self::SHAREABLE.bits(),
            MemType::Device => Self::SHAREABLE.bits(),
        };
        Self::from_bits_retain(bits)
    }

    fn mem_type(&self) -> MemType {
        let idx = (self.bits() & Self::ATTR_INDEX_MASK) >> 2;
        match idx {
            Self::PTE_S2_MEM_ATTR_NORMAL_INNER_WRITE_BACK_CACHEABLE
                | Self::PTE_S2_MEM_ATTR_NORMAL_OUTER_WRITE_BACK_CACHEABLE 
                => MemType::Normal,
            Self::PTE_S2_MEM_ATTR_NORMAL_OUTER_WRITE_BACK_NOCACHEABLE => MemType::NormalNonCache,
            0 => MemType::Device,
            _ => panic!("Invalid memory attribute index"),
        }
    }
}

impl From<DescriptorAttr> for MappingFlags {
    fn from(attr: DescriptorAttr) -> Self {
        let mut flags = Self::empty();
        if attr.contains(DescriptorAttr::VALID) {
            flags |= Self::READ;
        }
        if !attr.contains(DescriptorAttr::S2AP_WO) {
            flags |= Self::WRITE;
        }
        if !attr.contains(DescriptorAttr::XN) {
            flags |= Self::EXECUTE;
        }
        if attr.mem_type() == MemType::Device {
            flags |= Self::DEVICE;
        }
        flags
    }
}

impl From<MappingFlags> for DescriptorAttr {
    fn from(flags: MappingFlags) -> Self {
        let mut attr = if flags.contains(MappingFlags::DEVICE) {
            Self::from_mem_type(MemType::Device)
        } else {
            Self::from_mem_type(MemType::Normal)
        };

        if flags.contains(MappingFlags::READ) {
            attr |= Self::S2AP_RO | Self::VALID;
        }
        if flags.contains(MappingFlags::WRITE) {
            attr |= Self::S2AP_WO;
        }
        attr
    }
}

/// A VMSAv8-64 translation table descriptor.
///
/// Note that the **AttrIndx\[2:0\]** (bit\[4:2\]) field is set to `0` for device
/// memory, and `1` for normal memory. The system must configure the MAIR_ELx
/// system register accordingly.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct A64PTEHV(u64);

impl A64PTEHV {
    const PHYS_ADDR_MASK: usize = 0x0000_ffff_ffff_f000; // bits 12..48

    /// Creates an empty descriptor with all bits set to zero.
    pub const fn empty() -> Self {
        Self(0)
    }
}

impl GenericPTE for A64PTEHV {
    fn new_page(paddr: PhysAddr, flags: MappingFlags, is_huge: bool) -> Self {
        let mut attr = DescriptorAttr::from(flags) | DescriptorAttr::AF;
        if !is_huge {
            attr |= DescriptorAttr::NON_BLOCK;
        }
        Self(attr.bits() | (paddr.as_usize() & Self::PHYS_ADDR_MASK) as u64)
    }
    fn new_table(paddr: PhysAddr) -> Self {
        let attr = DescriptorAttr::NON_BLOCK | DescriptorAttr::VALID;
        Self(attr.bits() | (paddr.as_usize() & Self::PHYS_ADDR_MASK) as u64)
    }
    fn paddr(&self) -> PhysAddr {
        PhysAddr::from(self.0 as usize & Self::PHYS_ADDR_MASK)
    }
    fn flags(&self) -> MappingFlags {
        DescriptorAttr::from_bits_truncate(self.0).into()
    }
    fn is_unused(&self) -> bool {
        self.0 == 0
    }
    fn is_present(&self) -> bool {
        DescriptorAttr::from_bits_truncate(self.0).contains(DescriptorAttr::VALID)
    }
    fn is_huge(&self) -> bool {
        !DescriptorAttr::from_bits_truncate(self.0).contains(DescriptorAttr::NON_BLOCK)
    }
    fn clear(&mut self) {
        self.0 = 0
    }

    fn set_paddr(&mut self, _paddr: PhysAddr) {
    }
    fn set_flags(&mut self, _flags: MappingFlags, _is_huge: bool) {
    }
}

impl fmt::Debug for A64PTEHV {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("A64PTE");
        f.field("raw", &self.0)
            .field("paddr", &self.paddr())
            .field("attr", &DescriptorAttr::from_bits_truncate(self.0))
            .field("flags", &self.flags())
            .finish()
    }
}
