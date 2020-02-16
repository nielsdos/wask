//! Allocator used to split an address space domain into virtual memory areas.

use crate::arch::address::VirtAddr;
use crate::arch::paging::{ActiveMapping, EntryFlags, PAGE_SIZE};
use crate::mm::avl_interval_tree::AVLIntervalTree;
use crate::mm::mapper::{MemoryError, MemoryMapper};
use crate::sync::spinlock::Spinlock;
use core::intrinsics::{likely, unlikely};

/// Virtual memory allocator.
pub struct VMAAllocator {
    tree: AVLIntervalTree,
}

/// Virtual memory area.
pub struct Vma {
    start: VirtAddr,
    size: usize,
}

/// Mapped of a Vma (may be partially).
pub struct MappedVma {
    vma: Vma,
}

/// Lazily mapped Vma, mapped on access.
pub struct LazilyMappedVma {
    vma: Vma,
    /// The flags to use when mapping the memory.
    flags: EntryFlags,
    /// The size of the real mapped part.
    mapped_size: usize,
}

impl Vma {
    /// Empty Vma.
    pub const fn empty() -> Self {
        Self {
            start: VirtAddr::null(),
            size: 0,
        }
    }

    /// Creates a new Vma of the requested size.
    pub fn create(size: usize) -> Result<Self, MemoryError> {
        with_vma_allocator(|allocator| allocator.alloc_region(size))
            .map(|start| Self { start, size })
            .ok_or(MemoryError::NoMoreVMA)
    }

    /// Convert to mapped Vma.
    pub fn map(
        self,
        map_off: usize,
        map_size: usize,
        flags: EntryFlags,
    ) -> Result<MappedVma, MemoryError> {
        debug_assert!(map_off % PAGE_SIZE == 0);
        debug_assert!(map_size % PAGE_SIZE == 0);

        if unlikely(map_off >= self.size || map_off + map_size > self.size) {
            Err(MemoryError::InvalidRange)
        } else {
            let mut mapping = ActiveMapping::get();
            mapping.map_range(self.start + map_off, map_size, flags)?;

            Ok(MappedVma { vma: self })
        }
    }

    /// Convert to a lazily mapped Vma.
    pub fn map_lazily(self, flags: EntryFlags) -> LazilyMappedVma {
        LazilyMappedVma {
            vma: self,
            flags,
            mapped_size: 0,
        }
    }

    /// Gets the starting address.
    #[inline]
    pub fn address(&self) -> VirtAddr {
        self.start
    }

    /// Gets the length.
    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }
}

impl MappedVma {
    /// Empty Vma.
    pub const fn empty() -> Self {
        Self { vma: Vma::empty() }
    }

    /// Gets the starting address.
    #[inline]
    pub fn address(&self) -> VirtAddr {
        self.vma.start
    }

    /// Gets the size.
    #[inline]
    pub fn size(&self) -> usize {
        self.vma.size
    }
}

impl LazilyMappedVma {
    /// Empty Vma.
    pub const fn empty() -> Self {
        Self {
            vma: Vma::empty(),
            flags: EntryFlags::empty(),
            mapped_size: 0,
        }
    }

    /// Gets the flags to use when mapping the memory.
    #[inline]
    pub fn flags(&self) -> EntryFlags {
        self.flags
    }

    /// Gets the starting address.
    #[inline]
    pub fn address(&self) -> VirtAddr {
        self.vma.start
    }

    /// Gets the size.
    #[inline]
    pub fn size(&self) -> usize {
        self.mapped_size
    }
}

impl Drop for Vma {
    fn drop(&mut self) {
        if likely(!self.address().is_null()) {
            with_vma_allocator(|allocator| allocator.insert_region(self.start, self.size));
        }
    }
}

fn drop_mapping(start: VirtAddr, size: usize) {
    let mut mapping = ActiveMapping::get();
    // We don't need to tell the exact mapped range, we own all of this.
    // For an empty mapping, the size will be zero, so we don't have to check that.
    mapping.free_and_unmap_range(start, size);
}

impl Drop for MappedVma {
    fn drop(&mut self) {
        drop_mapping(self.address(), self.size());
    }
}

impl Drop for LazilyMappedVma {
    fn drop(&mut self) {
        drop_mapping(self.address(), self.size());
    }
}

impl VMAAllocator {
    /// Creates a new VMA allocator.
    const fn new() -> Self {
        Self {
            tree: AVLIntervalTree::new(),
        }
    }

    /// Inserts a region.
    pub fn insert_region(&mut self, addr: VirtAddr, len: usize) {
        debug_assert!(addr.is_page_aligned());
        debug_assert!(len % PAGE_SIZE == 0);
        self.tree.return_interval(addr.as_usize(), len);
    }

    /// Allocates a region.
    pub fn alloc_region(&mut self, len: usize) -> Option<VirtAddr> {
        debug_assert!(len % PAGE_SIZE == 0);
        self.tree.find_len(len).map(VirtAddr::new)
    }
}

static VMA_ALLOCATOR: Spinlock<VMAAllocator> = Spinlock::new(VMAAllocator::new());

/// Execute something using the VMA allocator.
pub fn with_vma_allocator<F, T>(f: F) -> T
where
    F: FnOnce(&mut VMAAllocator) -> T,
{
    f(&mut VMA_ALLOCATOR.lock())
}
