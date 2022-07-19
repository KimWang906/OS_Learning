#![no_std] // Rust 표준 라이브러리를 링크하지 않도록 합니다.
#![no_main] // Rust 언어에서 사용하는 실행 시작 지점 (main 함수)을 사용하지 않습니다.

use core::panic::PanicInfo;
mod vga_buffer; //vag_buffer Module 사용

/// 패닉이 일어날 경우, 이 함수가 호출됩니다.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {

    println!("Testing println macro");

    loop {}
}