// main.rs
#![no_std] // not link to Rust std lib
#![no_main] // 
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

mod vga_buffer;
mod serial;

// fn main() {
    // println!("Hello, world!");
// }

/// invoked when panic
#[cfg(not(test))] // new attribute
#[panic_handler]
// This function cannot return, diverging function, 'never' type
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

// our panic handler in test mode
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

static HELLO: &[u8] = b"Hello World!";

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[no_mangle]
// this mean no name mangling which ensure Rust compiler output a _start funciton
pub extern "C" fn _start() -> ! {
    // loop {}
    // let vga_buffer = 0xb8000 as *mut u8;

    // for (i, &byte) in HELLO.iter().enumerate() {
    //     unsafe {
    //         *vga_buffer.offset(i as isize * 2) = byte;
    //         *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
    //     }
    // }

    // loop {}
    // vga_buffer::print_something();
    use core::fmt::Write;
    vga_buffer::WRITER.lock().write_str("Hello agains").unwrap();
    write!(vga_buffer::WRITER.lock(), ", some numbers: {} {}", 42, 1.339).unwrap();
    println!("\n");
    println!("HELLO WORLD {}", "!");

    #[cfg(test)]
    test_main();
    panic!("Some panic message");

    loop {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    /// new 
    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trivial_assertion() {
    serial_print!("trivial assertion... ");
    print!("trivial assertion... ");
    assert_eq!(0, 1);
    println!("[ok]");
    serial_println!("[ok]");
}