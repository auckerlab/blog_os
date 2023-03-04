#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
// #![test_runner(blog_os::test_runner)]

use core::panic::PanicInfo;
use blog_os::println;

#[no_mangle]  // do not mangle the name of this function
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

fn test_runner(tests: &[&dyn Fn()]) {
    unimplemented!(); // 充当暂未实现的占位符
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // loop {}  // 作为panic处理器的内容
    blog_os::test_panic_handler(info)
}

#[test_case]
fn test_println() {
    println!("test_println output");
}