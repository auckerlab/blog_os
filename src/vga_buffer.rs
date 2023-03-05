use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

// src/vga_buffer.rs
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// follow the 'copy semantics', which can be compared, debug, printed
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
    LighBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts; // these lines of codes are used to prevent deadlock

    interrupts::without_interrupts(|| {  // prevent deadlock (interrupt is disabled when Mutex is locked)
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: color_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII or '\n'
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not included bytes
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        // todo!()
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        // todo!()
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// pub fn print_something() {
//     use core::fmt::Write;
//     let mut writer = Writer {
//         column_position: 0,
//         color_code: ColorCode::new(Color::Yellow, Color::Black),
//         buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
//     };

//     writer.write_byte(b'H');
//     writer.write_string("ellos ");
//     // writer.write_string("Wörld!");
//     write!(writer, "The numbers are {} and {}", 42, 1.0/3.0).unwrap();
// }

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;  // new
    use x86_64::instructions::interrupts; // new

    // 使用lock()函数显式加锁，然后将println改为writeln宏，以此绕开输出必须加锁的限制
    // 为了避免死锁，同时在测试函数执行期间禁用中断，否则中断处理函数可能会意外被触发。
    // 为了防止在测试执行前计时器中断被触发所造成干扰，我们先输出一句\n，即可避免行首出现多余的.造成干扰。
    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {  // new
        let mut writer = WRITER.lock();  // new
        writeln!(writer, "\n{}", s).expect("writeln failed");  // new
        // println!("{}", s);
        for (i, c) in s.chars().enumerate() {
            // let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            // 由于触发了竞态条件，在println和检测逻辑之间触发了计时器中断，其处理函数同样调用了输出语句。
            // 为了修复此问题，需要将WRITER加锁的范围扩大到整个测试函数，使计时器中断处理函数无法输出.，
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
    
}