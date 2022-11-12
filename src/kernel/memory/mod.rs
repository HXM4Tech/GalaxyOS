mod area_frame_allocator;
mod paging;
pub mod allocator;
mod stack_allocator;

use multiboot2::BootInformation;

pub use self::area_frame_allocator::AreaFrameAllocator;
use self::paging::PhysicalAddress;
pub use self::paging::remap_the_kernel;
pub use self::stack_allocator::Stack;

pub fn init(boot_info: BootInformation) -> MemoryController {
    assert_has_not_been_called!("memory::init can be called only once");

    enable_nxe_bit();
    enable_write_protect_bit();

    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");
    let elf_sections_tag = boot_info.elf_sections_tag().expect("Elf-sections tag required");

    let kernel_start = elf_sections_tag.sections().map(|s| s.start_address()).min().unwrap();
    let kernel_end = elf_sections_tag.sections().map(|s| s.end_address()).max().unwrap();

    println_all!("kernel start: {:#x}, kernel end: {:#x}", kernel_start, kernel_end);
    println_all!("multiboot start: {:#x}, multiboot end: {:#x}", boot_info.start_address(), boot_info.end_address());

    let mut frame_allocator = AreaFrameAllocator::new(
        kernel_start as usize, kernel_end as usize, boot_info.start_address(),
        boot_info.end_address(), memory_map_tag.memory_areas()
    );

    let mut active_table = paging::remap_the_kernel(&mut frame_allocator, &boot_info);

    use self::paging::Page;
    use self::allocator::{HEAP_START, HEAP_SIZE};

    let heap_start_page = Page::containing_address(HEAP_START);
    let heap_end_page = Page::containing_address(HEAP_START + HEAP_SIZE-1);

    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        active_table.map(page, paging::EntryFlags::WRITABLE, &mut frame_allocator);
    }

    allocator::init();

    let stack_allocator = {
        let stack_alloc_start = heap_end_page + 1;
        let stack_alloc_end = stack_alloc_start + 100;
        let stack_alloc_range = Page::range_inclusive(stack_alloc_start, stack_alloc_end);
        stack_allocator::StackAllocator::new(stack_alloc_range)
    };

    return MemoryController {
        active_table: active_table,
        frame_allocator: frame_allocator,
        stack_allocator: stack_allocator,
    };
}

fn enable_nxe_bit() {
    use x86_64::registers::model_specific::Efer;

    let nxe_bit = 1 << 11;
    unsafe {
        let efer = Efer::read_raw();
        Efer::write_raw(efer | nxe_bit);
    }
}

fn enable_write_protect_bit() {
    use x86_64::registers::control::{Cr0, Cr0Flags};
    unsafe { Cr0::write(Cr0::read() | Cr0Flags::WRITE_PROTECT) };
}


pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    number: usize,
}

impl Frame {
    fn containing_address(address: usize) -> Frame {
        return Frame{ number: address / PAGE_SIZE };
    }

    fn start_address(&self) -> PhysicalAddress {
        return self.number * PAGE_SIZE;
    }

    fn clone(&self) -> Frame {
        return Frame { number: self.number };
    }

    fn range_inclusive(start: Frame, end: Frame) -> FrameIter {
        return FrameIter {
            start: start,
            end: end,
        };
    }
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}

struct FrameIter {
    start: Frame,
    end: Frame,
}

impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start.number += 1;
            return Some(frame);
        } else {
            return None;
        }
    }
}

pub struct MemoryController {
    active_table: paging::ActivePageTable,
    frame_allocator: AreaFrameAllocator,
    stack_allocator: stack_allocator::StackAllocator,
}

impl MemoryController {
    pub fn alloc_stack(&mut self, size_in_pages: usize) -> Option<Stack> {
        let &mut MemoryController { ref mut active_table, ref mut frame_allocator, ref mut stack_allocator } = self;
        stack_allocator.alloc_stack(active_table, frame_allocator, size_in_pages)
    }
}
