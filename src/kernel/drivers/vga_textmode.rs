// The VGA text-mode driver.
// Displayed characters are encoded in code page 437. 

use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::Port;

use crate::console::Color;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    row_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                });

                self.column_position += 1;

                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                } else {
                    self.move_cursor(self.row_position, self.column_position);
                }
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }

    fn new_line(&mut self) {
        if self.row_position >= BUFFER_HEIGHT - 1 {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.buffer.chars[row][col].read();
                    self.buffer.chars[row - 1][col].write(character);
                }
            }

            self.clear_row(BUFFER_HEIGHT - 1);
        } else {
            self.row_position += 1;
        }

        self.column_position = 0;
        self.move_cursor(self.row_position, self.column_position);
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    fn clear_screen(&mut self) {
        self.row_position = 0;
        self.column_position = 0;
        
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }

        self.move_cursor(0, 0);
    }

    fn move_cursor(&mut self, row: usize, col: usize) {
        let pos = row * BUFFER_WIDTH + col;

        let mut p0x3d4 = Port::new(0x3d4);
        let mut p0x3d5 = Port::new(0x3d5);

        unsafe {
            p0x3d4.write(0x0f as u8);
            p0x3d5.write((pos & 0xff) as u8);
            p0x3d4.write(0x0e as u8);
            p0x3d5.write(((pos >> 8) & 0xff) as u8);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        row_position: 0,
        color_code: ColorCode::new(Color::LightGray, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::drivers::vga_textmode::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::drivers::vga_textmode::print!("\n"));
    ($($arg:tt)*) => ($crate::drivers::vga_textmode::print!("{}\n", format_args!($($arg)*)));
}

pub(crate) use print;
pub(crate) use println;

pub fn set_color(fg: Color, bg: Color) {
    WRITER.lock().color_code = ColorCode::new(fg, bg);
}

pub fn clear_screen() {
    WRITER.lock().clear_screen();
}

pub fn set_cursor_position(row: usize, col: usize) {
    let mut writer = WRITER.lock();
    writer.move_cursor(row, col);
    writer.row_position = row;
    writer.column_position = col;
}

pub fn get_cursor_position() -> (usize, usize) {
    let writer = WRITER.lock();
    return (writer.row_position, writer.column_position);
}
