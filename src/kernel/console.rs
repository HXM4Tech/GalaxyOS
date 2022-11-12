#![macro_use]

// There will be more console drivers in future, so it is good to have unified interface for printing to all of them
use crate::drivers::{vga_textmode};

// TODO: More color modes (eg. Color8, Color255)
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    Yellow = 14,
    White = 15,
}

// TODO: ANSI codes support
#[macro_export]
macro_rules! print_all {
    ($($arg:tt)*) => ($crate::drivers::vga_textmode::print!("{}", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println_all {
    () => (print_all!("\n"));
    ($($arg:tt)*) => (print_all!("{}\n", format_args!($($arg)*)));
}

pub fn set_color_all(fg: Color, bg: Color) {
    vga_textmode::set_color(fg, bg);
}

// TODO: Create logger and use print(ln)_all only in it
// TODO: Console object - to access each console's specific parameters (eg. get_size(), get_term()) and write to specific one
