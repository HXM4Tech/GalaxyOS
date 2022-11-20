// The VGA text-mode driver.
// Displayed characters are encoded in code page 437. 

use volatile::Volatile;
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

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// Writer is no longer needed here, so we should remove it in the future
//   making the printing and moving cursor functionality functions only.
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
            0x08 => self.backspace(),
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

    fn backspace(&mut self) {
        if self.column_position > 0 {
            self.column_position -= 1;
            self.buffer.chars[self.row_position][self.column_position].write(ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            });
        } else if self.row_position > 0 {
            self.row_position -= 1;
            self.column_position = BUFFER_WIDTH - 1;
            self.buffer.chars[self.row_position][self.column_position].write(ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            });
        }
        
        self.move_cursor(self.row_position, self.column_position);
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

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        row_position: 0,
        color_code: ColorCode::new(Color::LightGray, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

pub fn set_color(fg: Color, bg: Color) {
    WRITER.lock().color_code = ColorCode::new(fg, bg);
}

pub fn get_current_color() -> (Color, Color) {
    let color_code = WRITER.lock().color_code;
    let fg = match color_code.0 & 0x0f {
        0 => Color::Black,
        1 => Color::Blue,
        2 => Color::Green,
        3 => Color::Cyan,
        4 => Color::Red,
        5 => Color::Magenta,
        6 => Color::Brown,
        7 => Color::LightGray,
        8 => Color::DarkGray,
        9 => Color::LightBlue,
        10 => Color::LightGreen,
        11 => Color::LightCyan,
        12 => Color::LightRed,
        13 => Color::LightMagenta,
        14 => Color::Yellow,
        15 => Color::White,
        _ => Color::Black,
    };
    let bg = match (color_code.0 & 0xf0) >> 4 {
        0 => Color::Black,
        1 => Color::Blue,
        2 => Color::Green,
        3 => Color::Cyan,
        4 => Color::Red,
        5 => Color::Magenta,
        6 => Color::Brown,
        7 => Color::LightGray,
        8 => Color::DarkGray,
        9 => Color::LightBlue,
        10 => Color::LightGreen,
        11 => Color::LightCyan,
        12 => Color::LightRed,
        13 => Color::LightMagenta,
        14 => Color::Yellow,
        15 => Color::White,
        _ => Color::Black,
    };
    return (fg, bg);
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

pub fn raw_print_char(c: char) {
    WRITER.lock().write_byte(c as u8);
}
