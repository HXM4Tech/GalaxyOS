use core::ops::{Deref, DerefMut, Add};
use multiboot2::BootInformation;

mod entry;
mod table;
mod temporary_page;
mod mapper;

use crate::memory::{PAGE_SIZE, Frame, FrameAllocator};
pub use self::entry::*;
use self::temporary_page::TemporaryPage;
use self::mapper::Mapper;

const ENTRY_COUNT: usize = 512;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
   number: usize,
}

impl Page {
    pub fn containing_address(address: VirtualAddress) -> Page {
        assert!(address < 0x0000_8000_0000_0000 ||
            address >= 0xffff_8000_0000_0000,
            "invalid address: 0x{:x}", address);
        return Page { number: address / PAGE_SIZE };
    }

    pub fn start_address(&self) -> usize {
        return self.number * PAGE_SIZE;
    }

    fn p4_index(&self) -> usize {
        return (self.number >> 27) & 0o777;
    }

    fn p3_index(&self) -> usize {
        return (self.number >> 18) & 0o777;
    }

    fn p2_index(&self) -> usize {
        return (self.number >> 9) & 0o777;
    }

    fn p1_index(&self) -> usize {
        return (self.number >> 0) & 0o777;
    }

    pub fn range_inclusive(start: Page, end: Page) -> PageIter {
        return PageIter {
            start: start,
            end: end,
        };
    }
}

impl Add<usize> for Page {
    type Output = Page;

    fn add(self, rhs: usize) -> Page {
        return Page { number: self.number + rhs };
    }
}

#[derive(Clone)]
pub struct PageIter {
    start: Page,
    end: Page,
}

impl Iterator for PageIter {
    type Item = Page;

    fn next(&mut self) -> Option<Page> {
        if self.start <= self.end {
            let page = self.start;
            self.start.number += 1;
            return Some(page);
        } else {
            return None;
        }
    }
}

pub struct ActivePageTable {
    mapper: Mapper,
}

impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Mapper {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Mapper {
        &mut self.mapper
    }
}

impl ActivePageTable {
    unsafe fn new() -> ActivePageTable {
        return ActivePageTable {
            mapper: Mapper::new(),
        };
    }

    pub fn with<F>(&mut self, table: &mut InactivePageTable, temporary_page: &mut temporary_page::TemporaryPage, f: F) where F: FnOnce(&mut Mapper) {
        use x86_64::instructions::tlb;
        use x86_64::registers::control;
        {
            let backup = Frame::containing_address(control::Cr3::read().0.start_address().as_u64() as usize);
    
            let p4_table = temporary_page.map_table_frame(backup.clone(), self);
    
            self.p4_mut()[511].set(table.p4_frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
            tlb::flush_all();
    
            f(self);
    
            p4_table[511].set(backup, EntryFlags::PRESENT | EntryFlags::WRITABLE);
            tlb::flush_all();
        }
    
        temporary_page.unmap(self);
    }

    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        use x86_64::PhysAddr;
        use x86_64::structures::paging::PhysFrame;
        use x86_64::registers::control;
    
        let old_table = InactivePageTable {
            p4_frame: Frame::containing_address(
                control::Cr3::read().0.start_address().as_u64() as usize
            ),
        };
        unsafe {
            control::Cr3::write(
                PhysFrame::from_start_address_unchecked(PhysAddr::new(new_table.p4_frame.start_address() as u64)),
                control::Cr3::read().1
            );
        }
        return old_table;
    }
}

pub struct InactivePageTable {
    p4_frame: Frame,
}

impl InactivePageTable {
    pub fn new(frame: Frame, active_table: &mut ActivePageTable, temporary_page: &mut TemporaryPage) -> InactivePageTable {
        {
            let table = temporary_page.map_table_frame(frame.clone(), active_table);
            table.zero();
            table[511].set(frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
        }
        temporary_page.unmap(active_table);

        return InactivePageTable { p4_frame: frame };
    }
}

pub fn remap_the_kernel<A>(allocator: &mut A, boot_info: &BootInformation) -> ActivePageTable where A: FrameAllocator {
    let mut temporary_page = TemporaryPage::new(Page { number: 0xfdcba987 }, allocator);

    let mut active_table = unsafe { ActivePageTable::new() };
    let mut new_table = {
        let frame = allocator.allocate_frame().expect("no more frames");
        InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
    };

    active_table.with(&mut new_table, &mut temporary_page, |mapper| {
        let elf_sections_tag = boot_info.elf_sections_tag().expect("Memory map tag required");

        for section in elf_sections_tag.sections() {
            if !section.is_allocated() {
                continue;
            }

            assert!(section.start_address() as usize % PAGE_SIZE == 0, "Section is not page aligned");
        
            let flags = EntryFlags::from_elf_section_flags(&section);
        
            let start_frame = Frame::containing_address(section.start_address() as usize);
            let end_frame = Frame::containing_address(section.end_address() as usize - 1);
            for frame in Frame::range_inclusive(start_frame, end_frame) {
                mapper.identity_map(frame, flags, allocator);
            }
        }

        let vga_buffer_frame = Frame::containing_address(0xb8000);
        mapper.identity_map(vga_buffer_frame, EntryFlags::WRITABLE, allocator);

        let multiboot_start = Frame::containing_address(boot_info.start_address());
        let multiboot_end = Frame::containing_address(boot_info.end_address() - 1);
        for frame in Frame::range_inclusive(multiboot_start, multiboot_end) {
            mapper.identity_map(frame, EntryFlags::PRESENT, allocator);
        }
    });

    let old_table = active_table.switch(new_table);

    // turn the old p4 page into a guard page
    let old_p4_page = Page::containing_address(old_table.p4_frame.start_address());
    active_table.unmap(old_p4_page, allocator);

    return active_table;
}
