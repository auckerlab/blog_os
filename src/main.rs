// main.rs
#![no_std] // not link to Rust std lib
#![no_main] // 
#![feature(custom_test_frameworks)]
// #![test_runner(crate::test_runner)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use blog_os::println;

// mod vga_buffer;
// mod serial;

// fn main() {
    // println!("Hello, world!");
// }

static HELLO: &[u8] = b"Hello World!";

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
    // use core::fmt::Write;
    // vga_buffer::WRITER.lock().write_str("Hello agains").unwrap();
    // write!(vga_buffer::WRITER.lock(), ", some numbers: {} {}", 42, 1.339).unwrap();
    // println!("\n");
    println!("HELLO WORLD {}", "!");

    blog_os::init();  // new

    /// the following was implemented in interrupt.rs file, so no 
    // invoke a breakpoint exception
    // x86_64::instructions::interrupts::int3();  // new

    // iterative fault
    // fn stack_overflow() {
    //     stack_overflow();  // every time iterative will cause the return address into the stack.
    // }
    // trigger stackoverflow
    // stack_overflow();

    // trigger a page fault
    // unsafe {
    //     *(0xdeadbeef as *mut u64) = 42;
    // };

    #[cfg(test)]
    test_main();
    // panic!("Some panic message");

    println!("it did not crash!");
    // loop {
    //     // use blog_os::print;
    //     // for _ in 0..10000 {
    //     //     print!("-");
    //     // }
    // }

    blog_os::hlt_loop();  // new
}

/// invoked when panic
#[cfg(not(test))] // new attribute
#[panic_handler]
// This function cannot return, diverging function, 'never' type
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    // loop {}
    blog_os::hlt_loop(); // new
}

// our panic handler in test mode
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // serial_println!("[failed]\n");
    // serial_println!("Error: {}\n", info);
    // exit_qemu(QemuExitCode::Failed);
    // loop {}
    blog_os::test_panic_handler(info)
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// #[repr(u32)]
// pub enum QemuExitCode {
//     Success = 0x10,
//     Failed = 0x11,
// }

// pub fn exit_qemu(exit_code: QemuExitCode) {
//     use x86_64::instructions::port::Port;

//     unsafe {
//         let mut port = Port::new(0xf4);
//         port.write(exit_code as u32);
//     }
// }

// pub trait Testable {
//     fn run(&self) -> ();
// }

// impl<T> Testable for T
// where 
//     T: Fn(),
// {
//     fn run(&self) {
//         serial_print!("{}...\t", core::any::type_name::<T>());
//         self();
//         serial_println!("[ok]");
//     }
// }

// #[cfg(test)]
// // fn test_runner(tests: &[&dyn Fn()]) {
// fn test_runner(tests: &[&dyn Testable]) {
//     serial_println!("Running {} tests", tests.len());
//     println!("Running {} tests", tests.len());
//     for test in tests {
//         // test();
//         test.run();
//     }
//     /// new 
//     exit_qemu(QemuExitCode::Success);
// }

// #[test_case]
// fn trivial_assertion() {
//     // serial_print!("trivial assertion... ");
//     // print!("trivial assertion... ");
//     assert_eq!(1, 1);
//     // println!("[ok]");
//     // serial_println!("[ok]");

//     // loop {}
// }