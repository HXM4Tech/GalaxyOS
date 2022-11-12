use crate::memory::paging::{self, Page, PageIter, ActivePageTable};
use crate::memory::{PAGE_SIZE, FrameAllocator};

pub struct StackAllocator {
    range: PageIter,
}

impl StackAllocator {
    pub fn new(page_range: PageIter) -> StackAllocator {
        return StackAllocator { range: page_range };
    }

    pub fn alloc_stack<FA: FrameAllocator>(&mut self, active_table: &mut ActivePageTable, frame_allocator: &mut FA, size_in_pages: usize) -> Option<Stack> {
        if size_in_pages == 0 {
            return None;
        }

        let mut range = self.range.clone();

        let guard_page = range.next();
        let stack_start = range.next();
        let stack_end = if size_in_pages == 1 {
            stack_start
        } else {
            range.nth(size_in_pages - 2)
        };

        return match (guard_page, stack_start, stack_end) {
            (Some(_), Some(start), Some(end)) => {
                self.range = range;

                for page in Page::range_inclusive(start, end) {
                    active_table.map(page, paging::EntryFlags::WRITABLE, frame_allocator);
                }

                let top_of_stack = end.start_address() + PAGE_SIZE;
                Some(Stack::new(top_of_stack, start.start_address()))
            }
            _ => None,
        };
    }
}

#[derive(Debug)]
pub struct Stack {
    top: usize,
    bottom: usize,
}

impl Stack {
    fn new(top: usize, bottom: usize) -> Stack {
        assert!(top > bottom);
        return Stack {
            top: top,
            bottom: bottom,
        };
    }

    pub fn top(&self) -> usize {
        return self.top;
    }

    pub fn bottom(&self) -> usize {
        return self.bottom;
    }
}

