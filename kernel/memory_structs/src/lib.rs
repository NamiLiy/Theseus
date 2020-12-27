//! This crate contains common types used for memory mapping. 

#![no_std]
#![feature(const_fn)]
#![feature(step_trait, step_trait_ext)]

extern crate kernel_config;
extern crate multiboot2;
extern crate xmas_elf;
#[macro_use] extern crate derive_more;
#[macro_use] extern crate raw_cpuid;
extern crate bit_field;
#[cfg(target_arch = "x86_64")]
extern crate entryflags_x86_64;
extern crate zerocopy;

use bit_field::BitField;
use core::{
    fmt,
    iter::Step,
    ops::{Add, AddAssign, Deref, DerefMut, RangeInclusive, Sub, SubAssign},
};
use kernel_config::memory::{MAX_PAGE_NUMBER, PAGE_SIZE};
#[cfg(target_arch = "x86_64")]
use entryflags_x86_64::EntryFlags;
use zerocopy::FromBytes;

/// A virtual memory address, which is a `usize` under the hood.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, 
    Binary, Octal, LowerHex, UpperHex, 
    BitAnd, BitOr, BitXor, BitAndAssign, BitOrAssign, BitXorAssign, 
    Add, Sub, AddAssign, SubAssign,
    FromBytes,
)]
#[repr(transparent)]
pub struct VirtualAddress(usize);

impl VirtualAddress {
    /// Creates a new `VirtualAddress`,
    /// checking that the address is canonical,
    /// i.e., bits (64:48] are sign-extended from bit 47.
    pub fn new(virt_addr: usize) -> Result<VirtualAddress, &'static str> {
        match virt_addr.get_bits(47..64) {
            0 | 0b1_1111_1111_1111_1111 => Ok(VirtualAddress(virt_addr)),
            _ => Err("VirtualAddress bits 48-63 must be a sign-extension of bit 47"),
        }
    }

    /// Creates a new `VirtualAddress` that is guaranteed to be canonical
    /// by forcing the upper bits (64:48] to be sign-extended from bit 47.
    pub const fn new_canonical(virt_addr: usize) -> VirtualAddress {
        // match virt_addr.get_bit(47) {
        //     false => virt_addr.set_bits(48..64, 0),
        //     true => virt_addr.set_bits(48..64, 0xffff),
        // };

        // The below code is semantically equivalent to the above, but it works in const functions.
        VirtualAddress(((virt_addr << 16) as isize >> 16) as usize)
    }

    /// Creates a VirtualAddress with the value 0.
    pub const fn zero() -> VirtualAddress {
        VirtualAddress(0)
    }

    /// Returns the underlying `usize` value for this `VirtualAddress`.
    #[inline]
    pub const fn value(&self) -> usize {
        self.0
    }

    /// Returns the offset that this VirtualAddress specifies into its containing memory Page.
    ///
    /// For example, if the PAGE_SIZE is 4KiB, then this will return
    /// the least significant 12 bits (12:0] of this VirtualAddress.
    pub const fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }

    pub const fn hugepage_offset(&self, page_size : HugePageSize) -> usize {
        self.0 & (page_size.value() - 1)
    }
}
impl fmt::Debug for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "v{:#X}", self.0)
    }
}
impl fmt::Display for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl fmt::Pointer for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Add<usize> for VirtualAddress {
    type Output = VirtualAddress;

    fn add(self, rhs: usize) -> VirtualAddress {
        VirtualAddress::new_canonical(self.0.saturating_add(rhs))
    }
}

impl AddAssign<usize> for VirtualAddress {
    fn add_assign(&mut self, rhs: usize) {
        *self = VirtualAddress::new_canonical(self.0.saturating_add(rhs));
    }
}

impl Sub<usize> for VirtualAddress {
    type Output = VirtualAddress;

    fn sub(self, rhs: usize) -> VirtualAddress {
        VirtualAddress::new_canonical(self.0.saturating_sub(rhs))
    }
}

impl SubAssign<usize> for VirtualAddress {
    fn sub_assign(&mut self, rhs: usize) {
        *self = VirtualAddress::new_canonical(self.0.saturating_sub(rhs));
    }
}

impl From<VirtualAddress> for usize {
    #[inline]
    fn from(virt_addr: VirtualAddress) -> usize {
        virt_addr.0
    }
}


/// A physical memory address, which is a `usize` under the hood.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, 
    Binary, Octal, LowerHex, UpperHex, 
    BitAnd, BitOr, BitXor, BitAndAssign, BitOrAssign, BitXorAssign, 
    Add, Sub, AddAssign, SubAssign,
    FromBytes,
)]
#[repr(transparent)]
pub struct PhysicalAddress(usize);

impl PhysicalAddress {
    /// Creates a new `PhysicalAddress`,
    /// checking that the bits (64:52] are 0.
    pub fn new(phys_addr: usize) -> Result<PhysicalAddress, &'static str> {
        match phys_addr.get_bits(52..64) {
            0 => Ok(PhysicalAddress(phys_addr)),
            _ => Err("PhysicalAddress bits 52-63 must be zero"),
        }
    }

    /// Creates a new `PhysicalAddress` that is guaranteed to be canonical
    /// by forcing the upper bits (64:52] to be 0.
    pub fn new_canonical(mut phys_addr: usize) -> PhysicalAddress {
        phys_addr.set_bits(52..64, 0);
        PhysicalAddress(phys_addr)
    }

    /// Returns the underlying `usize` value for this `PhysicalAddress`.
    #[inline]
    pub fn value(&self) -> usize {
        self.0
    }

    /// Creates a PhysicalAddress with the value 0.
    pub const fn zero() -> PhysicalAddress {
        PhysicalAddress(0)
    }

    /// Returns the offset that this PhysicalAddress specifies into its containing memory Frame.
    ///
    /// For example, if the PAGE_SIZE is 4KiB, then this will return
    /// the least significant 12 bits (12:0] of this PhysicalAddress.
    pub fn frame_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }

    pub fn hugepage_frame_offset(&self, page_size : HugePageSize) -> usize {
        self.0 & (page_size.value() - 1)
    }
}
impl fmt::Debug for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "p{:#X}", self.0)
    }
}
impl fmt::Display for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl fmt::Pointer for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Add<usize> for PhysicalAddress {
    type Output = PhysicalAddress;

    fn add(self, rhs: usize) -> PhysicalAddress {
        PhysicalAddress::new_canonical(self.0.saturating_add(rhs))
    }
}

impl AddAssign<usize> for PhysicalAddress {
    fn add_assign(&mut self, rhs: usize) {
        *self = PhysicalAddress::new_canonical(self.0.saturating_add(rhs));
    }
}

impl Sub<usize> for PhysicalAddress {
    type Output = PhysicalAddress;

    fn sub(self, rhs: usize) -> PhysicalAddress {
        PhysicalAddress::new_canonical(self.0.saturating_sub(rhs))
    }
}

impl SubAssign<usize> for PhysicalAddress {
    fn sub_assign(&mut self, rhs: usize) {
        *self = PhysicalAddress::new_canonical(self.0.saturating_sub(rhs));
    }
}

impl From<PhysicalAddress> for usize {
    #[inline]
    fn from(virt_addr: PhysicalAddress) -> usize {
        virt_addr.0
    }
}


/// An area of physical memory.
#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct PhysicalMemoryArea {
    pub base_addr: PhysicalAddress,
    pub size_in_bytes: usize,
    pub typ: u32,
    pub acpi: u32,
}
impl PhysicalMemoryArea {
    pub fn new(
        paddr: PhysicalAddress,
        size_in_bytes: usize,
        typ: u32,
        acpi: u32,
    ) -> PhysicalMemoryArea {
        PhysicalMemoryArea {
            base_addr: paddr,
            size_in_bytes: size_in_bytes,
            typ: typ,
            acpi: acpi,
        }
    }
}

/// A structure indicating a page size the CPU supports
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct PageSize(usize);
pub struct HugePageSize(usize);

impl HugePageSize {
    /// Creates a new `HugePageSize`,
    /// checking that the CPU actually support the size.
    pub fn new(page_size_in_bytes: usize) -> Result<HugePageSize, &'static str> {

        const MB_2: usize = 2*1024*1024;
        const GB_1: usize = 1024*1024*1024;

        match page_size_in_bytes {
            // 4K pages
            4096 => Ok(HugePageSize(page_size_in_bytes)),

            // 2M pages
            // if CR0.PG = 1, CR4.PAE = 1, and IA32_EFER.LME = 1, IA-32e paging is used
            // IA-32e supports 2M paging
            MB_2 => Ok(HugePageSize(page_size_in_bytes)),

            // 1G pages
            // If CPUID.80000001H:EDX.Page1GB [bit 26] = 1,
            GB_1 => {
                let res = cpuid!(0x80000001);
                if (res.edx >> 26) & 1  == 1 {
                    Ok(HugePageSize(page_size_in_bytes))
                } else {
                    Err("The architecture does not support 1GB page size")
                }
            },

            _ => {
                Err("The architecture does not support the requested page size")
            },
        }
        
    }

    // ratio of huge_page_size_to_standard_page_size
    pub fn huge_page_ratio(&self) -> usize {
        // self.0 / PAGE_SIZE

        const MB_2: usize = 2*1024*1024;
        const GB_1: usize = 1024*1024*1024;
        match self.0 {
            4096 => 1,
            MB_2 => 512,
            GB_1 => 512*512,
            _ => 1,
        }
    }

    // Convenience function to get the actual size
    #[inline]
    pub const fn value(&self) -> usize {
        self.0
    }

    
}

impl PageSize {
    /// Creates a new `PageSize`,
    /// checking that the CPU actually support the size.
    pub fn new(page_size_in_bytes: usize) -> Result<PageSize, &'static str> {

        const KB_4: usize =         4*1024;
        const MB_2: usize =    2*1024*1024;
        const GB_1: usize = 1024*1024*1024;

        match page_size_in_bytes {
            // 4K pages
            KB_4 => Ok(PageSize(page_size_in_bytes)),

            // 2M pages
            // if CR0.PG = 1, CR4.PAE = 1, and IA32_EFER.LME = 1, IA-32e paging is used
            // IA-32e supports 2M paging
            MB_2 => Ok(PageSize(page_size_in_bytes)),

            // 1G pages
            // If CPUID.80000001H:EDX.Page1GB [bit 26] = 1,
            GB_1 => {
                let res = cpuid!(0x80000001);
                if (res.edx >> 26) & 1  == 1 {
                    Ok(PageSize(page_size_in_bytes))
                } else {
                    Err("The architecture does not support 1GB page size")
                }
            },

            _ => {
                Err("The architecture does not support the requested page size")
            },
        }
        
    }

    // ratio of huge_page_size_to_standard_page_size
    pub fn huge_page_ratio(&self) -> usize {
        // self.0 / PAGE_SIZE
        const KB_4: usize =         4*1024;
        const MB_2: usize = 2*1024*1024;
        const GB_1: usize = 1024*1024*1024;
        
        match self.0 {
            KB_4 => 1,
            MB_2 => 512,
            GB_1 => 512*512,
            _ => 1,
        }
    }

    // Convenience function to get the actual size
    #[inline]
    pub const fn value(&self) -> usize {
        self.0
    } 
}

impl Default for HugePageSize {
    fn default() -> Self { HugePageSize(PAGE_SIZE) }
}

/// A `Frame` is a chunk of **physical** memory,
/// similar to how a `Page` is a chunk of **virtual** memory.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    pub number: usize,
}
impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Frame(p{:#X})", self.start_address())
    }
}

impl Frame {
    /// Returns the `Frame` containing the given `PhysicalAddress`.
    pub fn containing_address(phys_addr: PhysicalAddress) -> Frame {
        Frame {
            number: phys_addr.value() / PAGE_SIZE,
        }
    }

    pub fn containing_huagepage_address(phys_addr: PhysicalAddress, page_size : HugePageSize) -> Frame {
        Frame {
            number: phys_addr.value() / page_size.value(),
        }
    }

    /// Returns the `PhysicalAddress` at the start of this `Frame`.
    pub fn start_address(&self) -> PhysicalAddress {
        PhysicalAddress::new_canonical(self.number * PAGE_SIZE)
    }

    pub fn huagepage_start_address(&self, page_size : HugePageSize) -> PhysicalAddress {
        PhysicalAddress::new_canonical(self.number * page_size.value())
    }
}

impl Add<usize> for Frame {
    type Output = Frame;

    fn add(self, rhs: usize) -> Frame {
        // cannot exceed max page number (which is also max frame number)
        Frame {
            number: core::cmp::min(MAX_PAGE_NUMBER, self.number.saturating_add(rhs)),
        }
    }
}

impl AddAssign<usize> for Frame {
    fn add_assign(&mut self, rhs: usize) {
        *self = Frame {
            number: core::cmp::min(MAX_PAGE_NUMBER, self.number.saturating_add(rhs)),
        };
    }
}

impl Sub<usize> for Frame {
    type Output = Frame;

    fn sub(self, rhs: usize) -> Frame {
        Frame {
            number: self.number.saturating_sub(rhs),
        }
    }
}

impl SubAssign<usize> for Frame {
    fn sub_assign(&mut self, rhs: usize) {
        *self = Frame {
            number: self.number.saturating_sub(rhs),
        };
    }
}

// Implementing these functions allow `Frame` to be in an `Iterator`.
unsafe impl Step for Frame {
    #[inline]
    fn steps_between(start: &Frame, end: &Frame) -> Option<usize> {
        Step::steps_between(&start.number, &end.number)
    }
    #[inline]
    fn forward_checked(start: Frame, count: usize) -> Option<Frame> {
        Step::forward_checked(start.number, count).map(|n| Frame { number: n })
    }
    #[inline]
    fn backward_checked(start: Frame, count: usize) -> Option<Frame> {
        Step::backward_checked(start.number, count).map(|n| Frame { number: n })
    }
}


/// A range of `Frame`s that are contiguous in physical memory.
#[derive(Clone)]
pub struct FrameRange(RangeInclusive<Frame>);

impl FrameRange {
    /// Creates a new range of `Frame`s that spans from `start` to `end`,
    /// both inclusive bounds.
    pub fn new(start: Frame, end: Frame) -> FrameRange {
        FrameRange(RangeInclusive::new(start, end))
    }

    /// Creates a FrameRange that will always yield `None`.
    pub fn empty() -> FrameRange {
        FrameRange::new(Frame { number: 1 }, Frame { number: 0 })
    }

    /// A convenience method for creating a new `FrameRange`
    /// that spans all `Frame`s from the given physical address
    /// to an end bound based on the given size.
    pub fn from_phys_addr(starting_virt_addr: PhysicalAddress, size_in_bytes: usize) -> FrameRange {
        assert!(size_in_bytes > 0);
        let start_frame = Frame::containing_address(starting_virt_addr);
		// The end frame is an inclusive bound, hence the -1. Parentheses are needed to avoid overflow.
        let end_frame = Frame::containing_address(starting_virt_addr + (size_in_bytes - 1));
        FrameRange::new(start_frame, end_frame)
    }

    /// Returns the `PhysicalAddress` of the starting `Frame` in this `FrameRange`.
    pub fn start_address(&self) -> PhysicalAddress {
        self.0.start().start_address()
    }


    pub fn start_frame(&self) -> Frame {
        Frame::containing_address(self.start_address())
    }

    /// Returns the number of `Frame`s covered by this iterator.
    /// Use this instead of the Iterator trait's `count()` method.
    /// This is instant, because it doesn't need to iterate over each entry, unlike normal iterators.
    pub fn size_in_frames(&self) -> usize {
        // add 1 because it's an inclusive range
        self.0.end().number + 1 - self.0.start().number
    }

    /// Whether this `FrameRange` contains the given `PhysicalAddress`.
    pub fn contains_phys_addr(&self, phys_addr: PhysicalAddress) -> bool {
        self.0.contains(&Frame::containing_address(phys_addr))
    }

    /// Returns the offset of the given `PhysicalAddress` within this `FrameRange`,
    /// i.e., the difference between `phys_addr` and `self.start()`.
    pub fn offset_from_start(&self, phys_addr: PhysicalAddress) -> Option<usize> {
        if self.contains_phys_addr(phys_addr) {
            Some(phys_addr.value() - self.start_address().value())
        } else {
            None
        }
    }

    /// Returns a new, separate `FrameRange` that is extended to include the given `Frame`.
    pub fn to_extended(&self, frame_to_include: Frame) -> FrameRange {
        // if the current FrameRange was empty, return a new FrameRange containing only the given frame_to_include
        if self.is_empty() {
            return FrameRange::new(frame_to_include.clone(), frame_to_include);
        }

        let start = core::cmp::min(self.0.start(), &frame_to_include);
        let end = core::cmp::max(self.0.end(), &frame_to_include);
        FrameRange::new(start.clone(), end.clone())
    }
}
impl fmt::Debug for FrameRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self.0)
	}
}
impl Deref for FrameRange {
    type Target = RangeInclusive<Frame>;
    fn deref(&self) -> &RangeInclusive<Frame> {
        &self.0
    }
}
impl DerefMut for FrameRange {
    fn deref_mut(&mut self) -> &mut RangeInclusive<Frame> {
        &mut self.0
    }
}

impl IntoIterator for FrameRange {
    type Item = Frame;
    type IntoIter = RangeInclusive<Frame>;

    fn into_iter(self) -> Self::IntoIter {
        self.0
    }
}


/// A virtual memory page, which contains the index of the page
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
    number: usize,
}
impl fmt::Debug for Page {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Page(v{:#X})", self.start_address())
    }
}

impl Page {
    /// Returns the `Page` that contains the given `VirtualAddress`.
    pub const fn containing_address(virt_addr: VirtualAddress) -> Page {
        Page {
            number: virt_addr.value() / PAGE_SIZE,
        }
    }

    // TODO_BOWEN : need to unify this function with the one above
    pub const fn containing_huge_page_address(virt_addr: VirtualAddress, page_size : PageSize) -> Page {
        Page {
            number: virt_addr.value() / page_size.value(),
        }
    }

    /// Returns the `VirtualAddress` as the start of this `Page`.
    pub const fn start_address(&self) -> VirtualAddress {
        // Cannot create VirtualAddress directly because the field is private
        VirtualAddress::new_canonical(self.number * PAGE_SIZE)
    }

    // TODO_BOWEN : need to unify this function with the one above
    pub const fn huage_page_start_address(&self) -> VirtualAddress {
        // Cannot create VirtualAddress directly because the field is private
        VirtualAddress::new_canonical(self.number * self.page_size.value())
    }

    // TODO_BOWEN : don't know what to do with it
    /// Convenience function to get the number of normal page at the first location of huge frame
    pub fn corresponding_normal_page(&self) -> Page {
        Page::containing_address(self.start_address())
    }

    // TODO_BOWEN : don't know what to do with it
    /// Convenience function to get the hugepage covering a normal page
    pub fn from_normal_page(page : Page, page_size: PageSize) -> Page {
        Page::containing_address(page.start_address(), page_size)
    }

    /// Returns the 9-bit part of this page's virtual address that is the index into the P4 page table entries list.
    pub fn p4_index(&self) -> usize {
        (self.number >> 27) & 0x1FF
    }

    /// Returns the 9-bit part of this page's virtual address that is the index into the P3 page table entries list.
    pub fn p3_index(&self) -> usize {
        (self.number >> 18) & 0x1FF
    }

    /// Returns the 9-bit part of this page's virtual address that is the index into the P2 page table entries list.
    pub fn p2_index(&self) -> usize {
        (self.number >> 9) & 0x1FF
    }

    /// Returns the 9-bit part of this page's virtual address that is the index into the P2 page table entries list.
    /// Using this returned `usize` value as an index into the P1 entries list will give you the final PTE,
    /// from which you can extract the mapped `Frame` (or its physical address) using `pointed_frame()`.
    pub fn p1_index(&self) -> usize {
        (self.number >> 0) & 0x1FF
    }
}

impl Add<usize> for Page {
    type Output = Page;

    fn add(self, rhs: usize) -> Page {
        // cannot exceed max page number
        Page {
            number: core::cmp::min(MAX_PAGE_NUMBER, self.number.saturating_add(rhs)),
        }
    }
}

impl AddAssign<usize> for Page {
    fn add_assign(&mut self, rhs: usize) {
        *self = Page {
            number: core::cmp::min(MAX_PAGE_NUMBER, self.number.saturating_add(rhs)),
        };
    }
}

impl Sub<usize> for Page {
    type Output = Page;

    fn sub(self, rhs: usize) -> Page {
        Page {
            number: self.number.saturating_sub(rhs),
        }
    }
}

impl SubAssign<usize> for Page {
    fn sub_assign(&mut self, rhs: usize) {
        *self = Page {
            number: self.number.saturating_sub(rhs),
        };
    }
}

// Implementing these functions allow `Page` to be in an `Iterator`.
unsafe impl Step for Page {
    #[inline]
    fn steps_between(start: &Page, end: &Page) -> Option<usize> {
        Step::steps_between(&start.number, &end.number)
    }
    #[inline]
    fn forward_checked(start: Page, count: usize) -> Option<Page> {
        Step::forward_checked(start.number, count).map(|n| Page { number: n })
    }
    #[inline]
    fn backward_checked(start: Page, count: usize) -> Option<Page> {
        Step::backward_checked(start.number, count).map(|n| Page { number: n })
    }
}

/// An inclusive range of `Page`s that are contiguous in virtual memory.
#[derive(Clone)]
pub struct PageRange(RangeInclusive<Page>);

impl PageRange {
    /// Creates a new range of `Page`s that spans from `start` to `end`,
    /// both inclusive bounds.
    pub const fn new(start: Page, end: Page) -> PageRange {
        PageRange(RangeInclusive::new(start, end))
    }

    /// Creates a PageRange that will always yield `None`.
    pub const fn empty() -> PageRange {
        PageRange::new(Page { number: 1 }, Page { number: 0 })
    }

    /// A convenience method for creating a new `PageRange`
    /// that spans all `Page`s from the given virtual address
    /// to an end bound based on the given size.
    pub fn from_virt_addr(starting_virt_addr: VirtualAddress, size_in_bytes: usize) -> PageRange {
        assert!(size_in_bytes > 0);
        let start_page = Page::containing_address(starting_virt_addr);
		// The end page is an inclusive bound, hence the -1. Parentheses are needed to avoid overflow.
        let end_page = Page::containing_address(starting_virt_addr + (size_in_bytes - 1));
        PageRange::new(start_page, end_page)
    }

    /// Returns the `VirtualAddress` of the starting `Page`.
    pub const fn start_address(&self) -> VirtualAddress {
        self.0.start().start_address()
    }

    /// Returns the size in number of `Page`s.
    /// Use this instead of the Iterator trait's `count()` method.
    /// This is instant, because it doesn't need to iterate over each `Page`, unlike normal iterators.
    pub const fn size_in_pages(&self) -> usize {
        // add 1 because it's an inclusive range
        self.0.end().number + 1 - self.0.start().number
    }

    /// Returns the size in number of bytes.
    pub const fn size_in_bytes(&self) -> usize {
        self.size_in_pages() * PAGE_SIZE
    }

    /// Whether this `PageRange` contains the given `VirtualAddress`.
    pub fn contains_virt_addr(&self, virt_addr: VirtualAddress) -> bool {
        self.0.contains(&Page::containing_address(virt_addr))
    }

    /// Returns the offset of the given `VirtualAddress` within this `PageRange`,
    /// i.e., the difference between `virt_addr` and `self.start_address()`.
    /// If the given `VirtualAddress` is not covered by this range of `Page`s, this returns `None`.
    ///  
    /// # Examples
    /// If the page range covered addresses `0x2000` to `0x4000`, then calling
    /// `offset_of_address(0x3500)` would return `Some(0x1500)`.
    pub fn offset_of_address(&self, virt_addr: VirtualAddress) -> Option<usize> {
        if self.contains_virt_addr(virt_addr) {
            Some(virt_addr.value() - self.start_address().value())
        } else {
            None
        }
    }

    /// Returns the `VirtualAddress` at the given `offset` into this mapping,  
    /// If the given `offset` is not covered by this range of `Page`s, this returns `None`.
    ///  
    /// # Examples
    /// If the page range covered addresses `0xFFFFFFFF80002000` to `0xFFFFFFFF80004000`,
    /// then calling `address_at_offset(0x1500)` would return `Some(0xFFFFFFFF80003500)`.
    pub fn address_at_offset(&self, offset: usize) -> Option<VirtualAddress> {
        if offset <= self.size_in_bytes() {
            Some(self.start_address() + offset)
        }
        else {
            None
        }
    }
}
impl fmt::Debug for PageRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self.0)
	}
}
impl Deref for PageRange {
    type Target = RangeInclusive<Page>;
    fn deref(&self) -> &RangeInclusive<Page> {
        &self.0
    }
}
impl DerefMut for PageRange {
    fn deref_mut(&mut self) -> &mut RangeInclusive<Page> {
        &mut self.0
    }
}

impl IntoIterator for PageRange {
    type Item = Page;
    type IntoIter = RangeInclusive<Page>;

    fn into_iter(self) -> Self::IntoIter {
        self.0
    }
}


/// The address bounds and mapping flags of a section's memory region.
#[derive(Debug)]
pub struct SectionMemoryBounds {
    /// The starting virtual address and physical address.
    pub start: (VirtualAddress, PhysicalAddress),
    /// The ending virtual address and physical address.
    pub end: (VirtualAddress, PhysicalAddress),
    /// The page table entry flags that should be used for mapping this section.
    pub flags: EntryFlags,
}

/// The address bounds and flags of the initial kernel sections that need mapping. 
/// 
/// It contains three main items, in which each item includes all sections that have identical flags:
/// * The `.text` section bounds cover all sections that are executable.
/// * The `.rodata` section bounds cover those that are read-only (.rodata, .gcc_except_table, .eh_frame).
/// * The `.data` section bounds cover those that are writable (.data, .bss).
/// 
/// It also contains the stack bounds, which are maintained separately.
#[derive(Debug)]
pub struct AggregatedSectionMemoryBounds {
   pub text:   SectionMemoryBounds,
   pub rodata: SectionMemoryBounds,
   pub data:   SectionMemoryBounds,
   pub stack:  SectionMemoryBounds,
}

/// A virtual memory page, which contains the index and the size of the page
/// HugePageSize contains only pagesizes supported by the architecture
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HugePage {
    number: usize,
    page_size: HugePageSize
}
impl fmt::Debug for HugePage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HugePage(v{:#X})", self.start_address())
    }
}

impl HugePage {
    /// Returns the `HugePage` that contains the given `VirtualAddress`.
    pub const fn containing_address(virt_addr: VirtualAddress, page_size : HugePageSize) -> HugePage {
        HugePage {
            number: virt_addr.value() / page_size.value(),
            page_size: page_size
        }
    }

    /// Returns the `VirtualAddress` as the start of this `HugePage`.
    pub const fn start_address(&self) -> VirtualAddress {
        // Cannot create VirtualAddress directly because the field is private
        VirtualAddress::new_canonical(self.number * self.page_size.value())
    }

    /// Convenience function to get the number of normal page at the first location of huge frame
    pub fn corresponding_normal_page(&self) -> Page {
        Page::containing_address(self.start_address())
    }

    /// Convenience function to get the hugepage covering a normal page
    pub fn from_normal_page(page : Page, page_size: HugePageSize) -> HugePage {
        HugePage::containing_address(page.start_address(), page_size)
    }

    // The following gives the corresponding values in the table.
    // However for huge pages not all entries may be relevent

    pub fn p4_index(&self) -> usize {
        (self.number*self.page_size.huge_page_ratio() >> 27) & 0x1FF
    }

    pub fn p3_index(&self) -> usize {
        (self.number*self.page_size.huge_page_ratio() >> 18) & 0x1FF
    }

    pub fn p2_index(&self) -> usize {
        (self.number*self.page_size.huge_page_ratio() >> 9) & 0x1FF
    }
    pub fn p1_index(&self) -> usize {
        (self.number*self.page_size.huge_page_ratio() >> 0) & 0x1FF
    }

    // Convenience function to get the page size
    pub fn page_size(&self) -> HugePageSize {
        self.page_size
    }
}

impl Add<usize> for HugePage {
    type Output = HugePage;

    fn add(self, rhs: usize) -> HugePage {
        // cannot exceed max page number
        // Division is safe as huge_page_ratio is guaranteed to be non zero
        HugePage {
            number: core::cmp::min(MAX_PAGE_NUMBER/self.page_size.huge_page_ratio(), self.number.saturating_add(rhs)),
            page_size: self.page_size,
        }
    }
}

impl AddAssign<usize> for HugePage {
    fn add_assign(&mut self, rhs: usize) {
        *self = HugePage {
            number: core::cmp::min(MAX_PAGE_NUMBER/self.page_size.huge_page_ratio(), self.number.saturating_add(rhs)),
            page_size: self.page_size,
        };
    }
}

impl Sub<usize> for HugePage {
    type Output = HugePage;

    fn sub(self, rhs: usize) -> HugePage {
        HugePage {
            number: self.number.saturating_sub(rhs),
            page_size: self.page_size,
        }
    }
}

impl SubAssign<usize> for HugePage {
    fn sub_assign(&mut self, rhs: usize) {
        *self = HugePage {
            number: self.number.saturating_sub(rhs),
            page_size: self.page_size,
        };
    }
}

// Implementing these functions allow `HugePage` to be in an `Iterator`.
unsafe impl Step for HugePage {
    #[inline]
    fn steps_between(start: &HugePage, end: &HugePage) -> Option<usize> {
        Step::steps_between(&start.number, &end.number)
    }
    #[inline]
    fn forward_checked(start: HugePage, count: usize) -> Option<HugePage> {
        Step::forward_checked(start.number, count).map(|n| HugePage { number: n, page_size: start.page_size })
    }
    #[inline]
    fn backward_checked(start: HugePage, count: usize) -> Option<HugePage> {
        Step::backward_checked(start.number, count).map(|n| HugePage { number: n, page_size: start.page_size  })
    }
}

/// An inclusive range of `HugePage`s that are contiguous in virtual memory.
#[derive(Clone)]
pub struct HugePageRange(RangeInclusive<HugePage>);

impl HugePageRange {
    /// Creates a new range of `HugePage`s that spans from `start` to `end`,
    /// both inclusive bounds.
    pub const fn new(start: HugePage, end: HugePage) -> HugePageRange {
        HugePageRange(RangeInclusive::new(start, end))
    }

    /// Creates a HugePageRange that will always yield `None`.
    pub const fn empty(page_size : HugePageSize) -> HugePageRange {
        HugePageRange::new(HugePage { number: 1, page_size: page_size}, HugePage { number: 0, page_size: page_size })
    }

    /// A convenience method for creating a new `HugePageRange`
    /// that spans all `HugePage`s from the given virtual address
    /// to an end bound based on the given size.
    pub fn from_virt_addr(starting_virt_addr: VirtualAddress, size_in_bytes: usize, page_size: HugePageSize) -> HugePageRange {
        assert!(size_in_bytes > 0);
        let start_page = HugePage::containing_address(starting_virt_addr, page_size);
		// The end page is an inclusive bound, hence the -1. Parentheses are needed to avoid overflow.
        let end_page = HugePage::containing_address(starting_virt_addr + (size_in_bytes - 1), page_size);
        HugePageRange::new(start_page, end_page)
    }

    /// Returns the `VirtualAddress` of the starting `HugePage`.
    pub fn start_address(&self) -> VirtualAddress {
        self.0.start().start_address()
    }

    /// Returns the size in number of `HugePage`s.
    pub fn size_in_pages(&self) -> usize {
        // add 1 because it's an inclusive range
        self.0.end().number + 1 - self.0.start().number
    }

    /// Returns the size in number of bytes.
    pub fn size_in_bytes(&self) -> usize {
        self.size_in_pages() * self.0.start().page_size().value()
    }

    /// Whether this `HugePageRange` contains the given `VirtualAddress`.
    pub fn contains_virt_addr(&self, virt_addr: VirtualAddress) -> bool {
        self.0.contains(&HugePage::containing_address(virt_addr,self.page_size()))
    }

    pub fn page_size(&self) -> HugePageSize {
        self.0.start().page_size()
    }

    /// Returns the offset of the given `VirtualAddress` within this `HugePageRange`,
    pub fn offset_of_address(&self, virt_addr: VirtualAddress) -> Option<usize> {
        if self.contains_virt_addr(virt_addr) {
            Some(virt_addr.value() - self.start_address().value())
        } else {
            None
        }
    }

    /// Returns the `VirtualAddress` at the given `offset` into this mapping,  
    pub fn address_at_offset(&self, offset: usize) -> Option<VirtualAddress> {
        if offset <= self.size_in_bytes() {
            Some(self.start_address() + offset)
        }
        else {
            None
        }
    }
}
impl fmt::Debug for HugePageRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self.0)
	}
}
impl Deref for HugePageRange {
    type Target = RangeInclusive<HugePage>;
    fn deref(&self) -> &RangeInclusive<HugePage> {
        &self.0
    }
}
impl DerefMut for HugePageRange {
    fn deref_mut(&mut self) -> &mut RangeInclusive<HugePage> {
        &mut self.0
    }
}

impl IntoIterator for HugePageRange {
    type Item = HugePage;
    type IntoIter = RangeInclusive<HugePage>;

    fn into_iter(self) -> Self::IntoIter {
        self.0
    }
}