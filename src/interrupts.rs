use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::print;
use crate::println;
use crate::gdt;
use lazy_static::lazy_static;

use pic8259::ChainedPics;
use spin;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
/// The following pic shows the Intel PIC8259 Architecture
///                      ____________                          ____________
/// Real Time Clock --> |            |   Timer -------------> |            |
/// ACPI -------------> |            |   Keyboard-----------> |            |      _____
/// Available --------> | Secondary  |----------------------> | Primary    |     |     |
/// Available --------> | Interrupt  |   Serial Port 2 -----> | Interrupt  |---> | CPU |
/// Mouse ------------> | Controller |   Serial Port 1 -----> | Controller |     |_____|
/// Co-Processor -----> |            |   Parallel Port 2/3 -> |            |
/// Primary ATA ------> |            |   Floppy disk -------> |            |
/// Secondary ATA ----> |____________|   Parallel Port 1----> |____________|


pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    // 无需显示指定对应值，默认情况下，对应值是上一个枚举对应值加一。
    Keyboard, // new
}
impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

// static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();
// pub fn init_idt() {
//     // let mut idt = InterruptDescriptorTable::new();
//     unsafe {
//         IDT.breakpoint.set_handler_fn(breakpoint_handler);
//         IDT.load();
//     }
// }

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        // idt.double_fault.set_handler_fn(double_fault_handler);  // new
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);  // new
        }

        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);  // new for timer on PIC 8259

        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler); // new for keyboard on PIC 8259
        idt
    };
}

pub fn init_idt() {
    IDT.load()
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame)
{
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame)
    // unimplemented!()
}

extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame)
{
    print!(".");

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: InterruptStackFrame)
{
    // print!("k");
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    // let mut port = Port::new(0x60);
    // let scancode: u8 = unsafe { port.read() };

    // map the num key to scancode
    // 此种方法过于笨，需要逐一对scancode进行对应，因此可以使用现成的crate
    // let key = match scancode {
    //     0x02 => Some('1'),
    //     0x03 => Some('2'),
    //     0x04 => Some('3'),
    //     0x05 => Some('4'),
    //     0x06 => Some('5'),
    //     0x07 => Some('6'),
    //     0x08 => Some('7'),
    //     0x09 => Some('8'),
    //     0x0a => Some('9'),
    //     0x0b => Some('0'),
    //     _ => None,
    // };
    // // print!("{}", scancode);
    // if let Some(key) = key {
    //     print!("{}", key);
    // }

    // 创建一个受到Mutex同步锁保护的Keyboard对象，初始化参数为美式键盘布局以及Set-1.至于HandleControl，它可以设定为将ctrl+[a-z]映射为Unicode字符U+0001至U+001A，
    // 但我们不想这样，因此使用了Ignore选项让ctrl仅仅表现为一个正常键位。
    // 对于每一个中断，我们都会为KEYBOARD加锁，从键盘控制器获取扫描码并将其传入add_byte函数，并将其转换为Option<KeyEvent>结构。KeyEvent包含了触发本次中断的按键信息，
    // 以及子动作是按下还是释放
    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1,
                HandleControl::Ignore)
            );
    }
    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        // 要处理KeyEvent，还需要将其传入process_keyevent函数，将其转换为人类可读的字符，有必要的话还需处理大小写。
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}