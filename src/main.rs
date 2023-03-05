// main.rs
#![no_std] // not link to Rust std lib
#![no_main] // 
#![feature(custom_test_frameworks)]
// #![test_runner(crate::test_runner)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use blog_os::{println, memory::{translate_addr, self, BootInfoFrameAllocator}};
use bootloader::{BootInfo, entry_point};
use x86_64::structures::paging::{Translate, Page};

// mod vga_buffer;
// mod serial;

// fn main() {
    // println!("Hello, world!");
// }

static HELLO: &[u8] = b"Hello World!";

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // unimplemented!()
    // use blog_os::memory::active_level_4_table;
    use x86_64::VirtAddr;
    use x86_64::structures::paging::PageTable;

    println!("Hello world{}", "!");
    blog_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    // new: initialize a mapper
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    // let mut frame_allocator = memory::EmptyFrameAllocator;
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    // map the unused page
    let page = Page::containing_address(VirtAddr::new(0));
    // let page = Page::containing_address(VirtAddr::new(0xdeadbeef000));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // write string `New!` to screen by new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe {page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};
    // let l4_table = unsafe { active_level_4_table(phys_mem_offset) };

    // for (i, entry) in l4_table.iter().enumerate() {
    //     if !entry.is_unused() {
    //         println!("L4 Entry {}: {:?}", i, entry);

    //         // get the physical address from the entry and convert it
    //         let phys = entry.frame().unwrap().start_address();
    //         let virt = phys.as_u64() + boot_info.physical_memory_offset;
    //         let ptr = VirtAddr::new(virt).as_mut_ptr();
    //         let l3_table: &PageTable = unsafe { &*ptr };

    //         // print non-empty entries of the level 3 table
    //         for (i, entry) in l3_table.iter().enumerate() {
    //             if !entry.is_unused() {
    //                 println!(" L3 Entry {}: {:?}", i, entry);
    //             }
    //         }
    //     }
    // }

    // let addresses = [
    //     // the identity-mapped vga buffer page
    //     0xb8000,
    //     // some code page
    //     0x201008,
    //     // some stack page
    //     0x0100_0020_1a10,
    //     // virtual address mapped to physical address 0
    //     boot_info.physical_memory_offset,
    // ];

    // for &address in &addresses {
    //     let virt = VirtAddr::new(address);
    //     // let phys = unsafe { translate_addr(virt, phys_mem_offset) };
    //     // new: use `mapper.translate_addr` method
    //     let phys = mapper.translate_addr(virt);
    //     println!("{:?} -> {:?}", virt, phys);
    // }

    // as before
    #[cfg(test)]
    test_main();

    println!("it did not crash!");
    blog_os::hlt_loop();
}


// #[no_mangle]
// // this mean no name mangling which ensure Rust compiler output a _start funciton
// pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {  // add boot_info parameter
//     // loop {}
//     // let vga_buffer = 0xb8000 as *mut u8;

//     // for (i, &byte) in HELLO.iter().enumerate() {
//     //     unsafe {
//     //         *vga_buffer.offset(i as isize * 2) = byte;
//     //         *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
//     //     }
//     // }

//     // loop {}
//     // vga_buffer::print_something();
//     // use core::fmt::Write;
//     // vga_buffer::WRITER.lock().write_str("Hello agains").unwrap();
//     // write!(vga_buffer::WRITER.lock(), ", some numbers: {} {}", 42, 1.339).unwrap();
//     // println!("\n");
//     println!("HELLO WORLD {}", "!");

//     blog_os::init();  // new

//     /// the following was implemented in interrupt.rs file, so no 
//     // invoke a breakpoint exception
//     // x86_64::instructions::interrupts::int3();  // new

//     // iterative fault
//     // fn stack_overflow() {
//     //     stack_overflow();  // every time iterative will cause the return address into the stack.
//     // }
//     // trigger stackoverflow
//     // stack_overflow();

//     // trigger a page fault
//     // unsafe {
//     //     *(0xdeadbeef as *mut u64) = 42;
//     // };

//     use x86_64::registers::control::Cr3;
//     let (level_4_page_table, _) = Cr3::read();
//     println!("Level 4 page table at: {:?}", level_4_page_table.start_address());

//     // new
//     // let ptr = 0xdeadbeef as *mut u32;
//     let ptr = 0x2031b2 as *mut u32;
//     // unsafe { *ptr = 42; }
//     // read from a code page
//     unsafe {let x = *ptr;}
//     println!("read worked");

//     // write to a code page
//     unsafe {*ptr = 42;}
//     println!("write worked");

//     #[cfg(test)]
//     test_main();
//     // panic!("Some panic message");

//     println!("it did not crash!");
//     // loop {
//     //     // use blog_os::print;
//     //     // for _ in 0..10000 {
//     //     //     print!("-");
//     //     // }
//     // }

//     blog_os::hlt_loop();  // new
// }

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