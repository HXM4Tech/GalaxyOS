#![no_std]

#![feature(ptr_internals)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]

extern crate volatile;
extern crate spin;
extern crate lazy_static;
extern crate multiboot2;
extern crate x86_64;
extern crate linked_list_allocator;
extern crate alloc;
extern crate bit_field;
extern crate pic8259;
extern crate pc_keyboard;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate once;

use core::panic::PanicInfo;

mod drivers;
mod console;
mod memory;
mod interrupts;
mod timer;

#[panic_handler]
fn _panic(info: &PanicInfo) -> ! {
    x86_64::instructions::interrupts::disable();

    println_all!("\n---[Kernel Panic: {}, at {}", info.message().unwrap(), info.location().unwrap());

    loop {
        x86_64::instructions::hlt();
    }
}

#[no_mangle]
pub extern "C" fn _start(multiboot_info_addr: usize) {
    let boot_info = unsafe { multiboot2::load(multiboot_info_addr) };

    let cmd_tag = boot_info.command_line_tag();
    let cmd;

    if cmd_tag.is_some() {
        cmd = cmd_tag.unwrap().command_line().trim();
    } else {
        cmd = "";
    }

    let mut memory_controller = memory::init(boot_info);
    interrupts::init(&mut memory_controller);
    console::init();

    println_all!("\x1b[1;32mGalaxyOS v{}", env!("CARGO_PKG_VERSION"));
    println_all!("Command line: {}\x1b[0m", cmd);

    loop {
        print_all!("\n\x1b[1;35m");
        println_all!("UPTIME: {}s", timer::get_uptime() / 1000);
        print_all!("\x1b[0m");
        timer::sleep(10000);
    }
}
