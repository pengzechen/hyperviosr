use arrayvec::ArrayVec;
use fdt::Fdt;

#[derive(Clone, Debug)]
pub struct Device {
    pub base_address: usize,
    pub size: usize,
}

#[derive(Clone, Debug, Default)]
pub struct MachineMeta {
    pub physical_memory_offset: usize,
    pub physical_memory_size: usize,

    pub virtio: ArrayVec<Device, 32>,

    pub pl011: Option<Device>,

    pub intc: ArrayVec<Device, 2>,

    pub pcie: Option<Device>,
}

impl MachineMeta {
    pub fn parse(dtb: usize) -> Self {
        debug!("this is dtb: {:X}", dtb);
        let addr: *const i32 = dtb as *const i32; // 请替换为你要读取的地址

        let fdt = unsafe { Fdt::from_ptr(dtb as *const u8) }.unwrap();
        let memory = fdt.memory();
        let mut meta = MachineMeta::default();
        for region in memory.regions() {
            meta.physical_memory_offset = region.starting_address as usize;
            meta.physical_memory_size = region.size.unwrap();
        }
        // probe virtio mmio device
        for node in fdt.find_all_nodes("/virtio_mmio") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let paddr = reg.starting_address as usize;
                let size = reg.size.unwrap();
                libax::debug!("virtio mmio addr: {:#x}, size: {:#x}", paddr, size);
                meta.virtio.push(Device {
                    base_address: paddr,
                    size,
                })
            }
        }
        meta.virtio.sort_unstable_by_key(|v| v.base_address);

        // probe uart device
        for node in fdt.find_all_nodes("/pl011") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let base_addr = reg.starting_address as usize;
                let size = reg.size.unwrap();
                libax::debug!("pl011 addr: {:#x}, size: {:#x}", base_addr, size);
                meta.pl011 = Some(Device {
                    base_address: base_addr,
                    size,
                });
            }
        }

        // probe intc(gicc, gicd)
        for node in fdt.find_all_nodes("/intc") {
            let regions = node.reg().unwrap();
            for region in regions {
                let paddr = region. starting_address as usize;
                let size = region.size.unwrap();
                println!("intc addr: {:#x}, size: {:#x}", paddr, size);
                meta.intc.push(Device {
                    base_address: paddr,
                    size,
                })
            }
        }
        meta.intc.sort_unstable_by_key(|v| v.base_address);

        for node in fdt.find_all_nodes("/pcie") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let base_addr = reg.starting_address as usize;
                let size = reg.size.unwrap();
                libax::debug!("pcie addr: {:#x}, size: {:#x}", base_addr, size);
                meta.pcie = Some(Device {
                    base_address: base_addr,
                    size,
                });
            }
        }

        meta
    }
}
