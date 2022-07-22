## Chapter 2:

### Cargo bootimage init

    https://hacking-yi.kro.kr/?p=233

#### Cargo.toml 파일에 아래 코드 추가

    [dependencies]
    bootloader = "0.9.8"

#### bootimage 설치

    cargo install bootimage

    rustup component add llvm-tools-preview

### 빌드 방법

    cargo build 대신 cargo bootimage를 사용하여 빌드할 수 있습니다.

### QEMU init

    sudo apt install qemu

    sudo apt install qemu-system-x86 && sudo apt install qemu-system-x86-xen

#### QEMU 가상환경 실행 코드

    qemu-system-x86_64 -drive format=raw,file=target/x86_64-blog_os/debug/bootimage-blog_os.bin

    qemu-system-x86_64 -drive format=raw,file=target/파일의 경로

#### 실제 컴퓨터에서 부팅하기

    dd if=target/x86_64-blog_os/debug/bootimage-blog_os.bin of=/dev/sdX && sync
    /dev/기기명

### Source code

    #![no_std] // Rust 표준 라이브러리를 링크하지 않도록 합니다.
    #![no_main] // Rust 언어에서 사용하는 실행 시작 지점 (main 함수)을 사용하지 않습니다.

    use core::panic::PanicInfo;

    // 패닉이 일어날 경우, 이 함수가 호출됩니다.
    #[panic_handler]
    fn panic(_info: &PanicInfo) -> ! {
        loop {}
    }

    static HELLO: &[u8] = b"Hello World!";

    #[no_mangle]
    pub extern "C" fn _start() -> ! {
        let vga_buffer = 0xb8000 as *mut u8;

        for (i, &byte) in HELLO.iter().enumerate() {
            unsafe { // 무슨 일이 있어도 unsafe는 최소한으로 사용할 것
                *vga_buffer.offset(i as isize * 2) = byte;
                *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
            }
        }

        loop {}
        /*
            우선 정수 0xb8000을 raw 포인터로 형변환 합니다.
            그 다음 static (정적 변수) 바이트 문자열 HELLO의 반복자를 통해 각 바이트를 읽고,
            enumerate 함수를 통해 각 바이트의 문자열 내에서의 인덱스 값 i를 얻습니다.
            for문의 내부에서는 offset 함수를 통해 VGA 버퍼에 문자열의 각 바이트 및 색상 코드를 저장합니다.
            (0xb: light cyan 색상 코드)
        */
    }
