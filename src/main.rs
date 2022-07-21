#![no_std] // Rust 표준 라이브러리를 링크하지 않도록 합니다.
#![no_main] // Rust 언어에서 사용하는 실행 시작 지점 (main 함수)을 사용하지 않습니다.
#![reexport_test_harness_main = "test_main"]

#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]

//VGA Print
// #[cfg(test)]
// fn test_runner(tests: &[&dyn Fn()]) {
//     println!("Running {} tests", tests.len());
//     for test in tests {
//         test();
//     }
//     // new
//     exit_qemu(QemuExitCode::Success);
// }

//Serial Print
#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    // new
    exit_qemu(QemuExitCode::Success);
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

use core::panic::PanicInfo;
mod vga_buffer; //vga_buffer Module 사용
mod serial; // serial Module 사용

/// 패닉이 일어날 경우, 이 함수가 호출됩니다.
#[cfg(not(test))] // new attribute
#[panic_handler]
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

#[no_mangle]
pub extern "C" fn _start() -> ! {

    println!("Testing println macro");

    #[cfg(test)]
    test_main(); //test_main 함수 호출

    loop {}
}

// #[test_case]
// fn trivial_assertion() {
//     print!("trivial assertion... ");
//     assert_eq!(1, 1);
//     println!("[ok]");
// }

#[test_case]
fn trivial_assertion() {
    serial_print!("trivial assertion... ");
    assert_eq!(1, 1);
    serial_println!("[ok]");
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
/*
    변경할 수 없는 수신자를 사용하는 호출 연산자의 버전입니다.

    이 특성( )을 함수 포인터 ( ) Fn와 혼동해서는 안 됩니다.

    fn의 인스턴스는 Fn상태를 변경하지 않고 반복적으로 호출할 수 있습니다.

    
    Fn캡처된 변수에 대한 변경 불가능한 참조만 사용하거나 아무 것도 캡처하지 않는 클로저와 (안전한) 함수 포인터 에 의해 자동으로 구현됩니다. 
    
    또한 구현 하는 모든 유형의 함수에 대해 구현 합니다.
    
    Fn&FFn FnMut및 둘 다 의 초특성이므로 의 모든 인스턴스 FnOnce는 또는가 예상 되는 매개변수로 사용할 수 있습니다 .FnFnFnMutFnOnce 
*/
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}