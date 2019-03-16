use core::panic::PanicInfo;

use crate::arch::address::VirtAddr;
use crate::arch::paging::{ActiveMapping, EntryFlags};
use crate::arch::x86_64::qemu;
use crate::mem::MemoryMapper;

/// Memory test.
#[cfg(feature = "test-mem")]
pub fn test_main() {
    let mut mapping = unsafe { ActiveMapping::get() };

    // Note: `va1` and `va2` are in the same P2
    let va1 = VirtAddr::new(0x400_000);
    let va2 = VirtAddr::new(0xdeadb000);
    let va3 = VirtAddr::new(0x600_000);

    mapping.get_and_map_single(va1, EntryFlags::PRESENT | EntryFlags::WRITABLE)
        .expect("could not map page #1");
    mapping.get_and_map_single(va2, EntryFlags::PRESENT | EntryFlags::WRITABLE)
        .expect("could not map page #2");

    mapping.free_and_unmap_single(va2);

    // Should not PF
    let ptr = va1.as_usize() as *mut i32;
    unsafe { ptr.write_volatile(42); }

    let phys = mapping.translate(va1);
    mapping.free_and_unmap_single(va1);

    mapping.get_and_map_single(va3, EntryFlags::PRESENT)
        .expect("could not map page #3");
    assert_eq!(mapping.translate(va3), phys);
    mapping.free_and_unmap_single(va3);

    unsafe { qemu::qemu_exit(0); }
}
