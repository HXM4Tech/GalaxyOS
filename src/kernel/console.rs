#![macro_use]

// There will be more console drivers in future, so it is good to have unified interface for printing to all of them
use crate::drivers::{vga_textmode};

use alloc::vec::Vec;
use core::fmt;
use spin::Mutex;
use lazy_static::lazy_static;
use pc_keyboard::{DecodedKey, KeyCode};

// TODO: More color modes (eg. Color8, Color255)
#[derive(Clone, Copy, PartialEq, Eq)]
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

#[derive(PartialEq, Eq)]
pub struct Console {
    width: usize,
    height: usize,
    print_func: fn(char),
    set_color_func: fn(Color, Color),
    get_current_color_func: fn() -> (Color, Color),
    clear_func: fn(),
    set_cursor_pos_func: fn(usize, usize),
    get_cursor_pos_func: fn() -> (usize, usize),
    dec: (usize, usize),
    sco: (usize, usize),
    in_ansi: bool,
    in_escape_seq: bool,
    escape_seq: Vec<u8>,
}


// TODO: Separate stdout, stderr and stdin
// TODO: Add support for more ANSI escape sequences
// TODO: Fix the backspace
impl Console {
    pub fn new(width: usize, height: usize, print_func: fn(char), set_color_func: fn(Color, Color), get_current_color_func: fn() -> (Color, Color), clear_func: fn(), set_cursor_pos_func: fn(usize, usize), get_cursor_pos_func: fn() -> (usize, usize)) -> Console {
        Console {
            width,
            height,
            print_func,
            set_color_func,
            get_current_color_func,
            clear_func,
            set_cursor_pos_func,
            get_cursor_pos_func,
            dec: (0, 0),
            sco: (0, 0),
            in_ansi: false,
            in_escape_seq: false,
            escape_seq: Vec::new(),
        }
    }

    pub fn print(&mut self, s: &str) {
        for c in s.bytes() {
            if c == b'\x1b' {
                self.in_ansi = true;
                continue;
            }
            if !self.in_ansi {
                (self.print_func)(c as char);
                continue;
            }
            if c == b'n' {
                self.in_ansi = false;
                (self.print_func)('\n');
                continue;
            }
            if c == b'r' {
                self.in_ansi = false;
                let current_pos = (self.get_cursor_pos_func)();
                (self.set_cursor_pos_func)(current_pos.0, 0);
                continue;
            }
            if c == b'b' {
                self.in_ansi = false;
                (self.print_func)(0x08 as char);
                continue;
            }
            if c == b'c' {
                self.in_ansi = false;
                (self.clear_func)();
                continue;
            }
            if c == b'7' {
                self.in_ansi = false;
                self.dec = (self.get_cursor_pos_func)();
                continue;
            }
            if c == b'8' {
                self.in_ansi = false;
                (self.set_cursor_pos_func)(self.dec.0, self.dec.1);
                continue;
            }
            if c == b']' || c == b'P' {
                // TODO: Implement OSC (Operating System Command) and DCS (Device Control String)
                self.in_ansi = false;
                continue;
            }
            if c == b'[' {
                self.in_escape_seq = true;
                self.escape_seq.clear();
                continue;
            }
            if !self.in_escape_seq {
                self.in_ansi = false;
                (self.print_func)(c as char);
                continue;
            }
            if c == b'H' {
                self.in_ansi = false;
                self.in_escape_seq = false;

                let mut args = self.escape_seq.split(|&x| x == b';');
                let mut row = usize::from_str_radix(core::str::from_utf8(args.next().unwrap_or(b"0")).unwrap_or("0"), 10).unwrap_or(0);
                let mut col = usize::from_str_radix(core::str::from_utf8(args.next().unwrap_or(b"0")).unwrap_or("0"), 10).unwrap_or(0);

                if row > self.height {
                    row = self.height;
                }
                if col > self.width {
                    col = self.width;
                }

                (self.set_cursor_pos_func)(row, col);
                continue;
            }
            if c == b'm' {
                self.in_ansi = false;
                self.in_escape_seq = false;

                let args = self.escape_seq.split(|&x| x == b';');

                let mut light = false;
                let current_color = (self.get_current_color_func)();
                let mut fg = current_color.0;
                let mut bg = current_color.1;

                for arg in args {
                    let arg = usize::from_str_radix(core::str::from_utf8(arg).unwrap_or("0"), 10).unwrap_or(0);
                    match arg {
                        0 => {
                            light = false;
                            fg = Color::LightGray;
                            bg = Color::Black;
                        },
                        1 => light = true,
                        2 => light = false,
                        22 => light = false,
                        30 => fg = Color::Black,
                        31 => fg = Color::Red,
                        32 => fg = Color::Green,
                        33 => fg = Color::Brown,
                        34 => fg = Color::Blue,
                        35 => fg = Color::Magenta,
                        36 => fg = Color::Cyan,
                        37 => fg = Color::LightGray,
                        39 => fg = Color::LightGray, // Default foreground color, TODO: Set this in constructor
                        40 => bg = Color::Black,
                        41 => bg = Color::Red,
                        42 => bg = Color::Green,
                        43 => bg = Color::Brown,
                        44 => bg = Color::Blue,
                        45 => bg = Color::Magenta,
                        46 => bg = Color::Cyan,
                        47 => bg = Color::LightGray,
                        49 => bg = Color::Black, // Default background color, TODO: Set this in constructor
                        _ => (),
                    }
                }

                if light {
                    fg = match fg {
                        Color::Black => Color::DarkGray,
                        Color::Red => Color::LightRed,
                        Color::Green => Color::LightGreen,
                        Color::Brown => Color::Yellow,
                        Color::Blue => Color::LightBlue,
                        Color::Magenta => Color::LightMagenta,
                        Color::Cyan => Color::LightCyan,
                        Color::LightGray => Color::White,
                        _ => fg,
                    };
                }

                (self.set_color_func)(fg, bg);
                continue;
            }

            self.escape_seq.push(c);
        }

    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        use x86_64::instructions::interrupts;

        interrupts::without_interrupts(|| {
            self.print(s);
        });
        Ok(())
    }
}

#[doc(hidden)]
pub fn _console_print(con: &mut Console, args: fmt::Arguments) {
    use core::fmt::Write;
    return con.write_fmt(args).unwrap();
}

pub struct Consoles {
    pub con_list: Mutex<Vec<Console>>,
}

impl Consoles {
    pub fn new() -> Consoles {
        return Consoles {
            con_list: Mutex::new(Vec::new()),
        };
    }

    pub fn register_console(&self, console: Console) -> usize {
        self.con_list.lock().push(console);
        return self.con_list.lock().len() - 1;
    }

    pub fn remove_console(&self, console: Console) {
        let mut con_list = self.con_list.lock();
        let mut index = 0;
        for con in con_list.iter() {
            if con as *const Console == &console as *const Console {
                break;
            }
            index += 1;
        }
        con_list.remove(index);
    }
}

lazy_static! {
    pub static ref CONSOLES: Consoles = Consoles::new();
}

pub fn init() {
    // Register VGA Text Mode Console
    // TODO: Serial, VESA, UEFI Graphics, etc.
    let vga_textmode_con = Console::new(
        vga_textmode::BUFFER_WIDTH,
        vga_textmode::BUFFER_HEIGHT,
        vga_textmode::raw_print_char,
        vga_textmode::set_color,
        vga_textmode::get_current_color,
        vga_textmode::clear_screen,
        vga_textmode::set_cursor_position,
        vga_textmode::get_cursor_position,
    );
    CONSOLES.register_console(vga_textmode_con);
}

#[macro_export]
macro_rules! print_all {
    ($($arg:tt)*) => {
        $crate::console::CONSOLES.con_list.lock().iter_mut().for_each(|con| { $crate::console::_console_print(con, format_args!($($arg)*)); });
    };
}

#[macro_export]
macro_rules! println_all {
    () => (print_all!("\n"));
    ($( $arg:tt )*) => (print_all!("{}\n", format_args!($($arg)*)));
}

pub fn register_input(key: DecodedKey) {
    let mut consoles = CONSOLES.con_list.lock();
    let mut con = consoles.get_mut(0).unwrap(); // TODO: Get active console

    match key {
        DecodedKey::Unicode(c) => {
            let mut s = [0; 4];
            let s = c.encode_utf8(&mut s);
            con.print(s);
        },
        DecodedKey::RawKey(raw_key) => match raw_key {
            KeyCode::Backspace => con.print("\x08"),
            _ => (),
        },
    }       
}
