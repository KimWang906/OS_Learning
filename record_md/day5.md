# Chapter 4 : Testing - 2(End)

    요약 : 5일차에는 여러가지 테스트케이스를 진행해보았고, 코드를 파일의 목적에 맞게 정리하였습니다.

## 실행 결과화면
<p align="center">
    <img src="/record_image/day_5_console_ok.png">
    성공적으로 테스트케이스를 마친 모습
    <img src="/record_image/day_5_console_fail.png">
    패닉이 발생하였을 때의 모습
</p>


## Source code

### src/main.rs

    #![no_std] // Rust 표준 라이브러리를 링크하지 않도록 합니다.
    #![no_main] // Rust 언어에서 사용하는 실행 시작 지점 (main 함수)을 사용하지 않습니다.
    #![feature(custom_test_frameworks)]
    #![test_runner(blog_os::test_runner)]
    #![reexport_test_harness_main = "test_main"]

    use core::panic::PanicInfo;
    use blog_os::println;

    #[no_mangle]
    pub extern "C" fn _start() -> ! {
        println!("Hello World{}", "!");

        #[cfg(test)]
        test_main();

        loop {}
    }

    // 패닉이 발생했을 때, 이 함수가 호출됩니다.
    #[cfg(not(test))]
    #[panic_handler]
    fn panic(info: &PanicInfo) -> ! {
        println!("{}", info);
        loop {}
    }

    #[cfg(test)]
    #[panic_handler]
    fn panic(info: &PanicInfo) -> ! {
        blog_os::test_panic_handler(info)
    }

### src/vga_buffer.rs

    #[allow(dead_code)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u8)]
    //색상을 정의합니다.
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
        LightBlue = 9,
        LightGreen = 10,
        LightCyan = 11,
        LightRed = 12,
        Pink = 13,
        Yellow = 14,
        White = 15,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    struct ColorCode(u8);

    //배경색은 ColorCode를 통해 표현됩니다.
    impl ColorCode {
        fn new(foreground: Color, background: Color) -> ColorCode {
            ColorCode((background as u8) << 4 | (foreground as u8))
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(C)] 
    //컴파일 중 구조체의 각 필드가 저장되는 순서가 바뀌지 않게 하기 위해 C 구조체 처럼 사용합니다.
    struct ScreenChar {
        ascii_character: u8,
        color_code: ColorCode,
    }

    /*  
        버퍼의 크기,
        배경색은 배퍼의 크기만큼 지정됩니다.
    */
    const BUFFER_HEIGHT: usize = 25;
    const BUFFER_WIDTH: usize = 80;

    use volatile::Volatile;

    struct Buffer {
        chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
    }

    //실제로 화면에 출력되는 타입
    pub struct Writer {
        column_position: usize,
        color_code: ColorCode,
        buffer: &'static mut Buffer,
    }

    impl Writer { //self는 Writer를 가리키고 있습니다.
        // ASCII 바이트를 출력하는 함수를 만듭니다.
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
                        color_code,
                    });
                    self.column_position += 1;
                }
            }
        }

        // fn new_line(&mut self) {/* TODO */}
    }

    impl Writer {
        pub fn write_string(&mut self, s: &str) {
            for byte in s.bytes() {
                match byte {
                    // 출력 가능한 ASCII 바이트 혹은 개행 문자
                    0x20..=0x7e | b'\n' => self.write_byte(byte),
                    // ASCII 코드 범위 밖의 값
                    _ => self.write_byte(0xfe),
                }

            }
        }
    }

    use core::fmt;

    impl fmt::Write for Writer {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            self.write_string(s);
            Ok(())
        }
    }

    impl Writer {
        fn new_line(&mut self) {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.buffer.chars[row][col].read();
                    self.buffer.chars[row - 1][col].write(character);
                }
            }
            self.clear_row(BUFFER_HEIGHT - 1);
            self.column_position = 0;
        }
        // fn clear_row(&mut self, row: usize) {/* TODO */}
    }

    impl Writer {
        fn clear_row(&mut self, row: usize) {
            let blank = ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            };
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row][col].write(blank);
            }
        }
    }

    /*
        문제는 Rust의 const evaluator가 컴파일 시간에 raw pointer를 레퍼런스로 전환하지 못한다는 것입니다.
        추후에는 이것이 가능해질 수도 있겠지만, 현재로서는 다른 해결책을 찾아야 합니다.

        Wrong code :
            pub static WRITER: Writer = Writer {
                column_position: 0,
                color_code: ColorCode::new(Color::Yellow, Color::Black),
                buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },                           
            };
    */

    /*
        현재 WRITER는 immutable (읽기 가능, 쓰기 불가능) 하여 실질적인 쓸모가 없습니다.
        모든 쓰기 함수들은 첫 인자로 &mut self를 받기 때문에 WRITER로 어떤 쓰기 작업도 할 수가 없습니다.
        이에 대한 해결책으로 mutable static은 어떨까요? 
        이 선택지를 고른다면 모든 읽기 및 쓰기 작업이 데이터 경쟁 상태 (data race) 및 기타 위험에 노출되기에 안전을 보장할 수 없게 됩니다.
        Rust에서 static mut는 웬만하면 사용하지 않도록 권장되며, 심지어 Rust 언어에서 완전히 static mut를 제거하자는 제안이 나오기도 했습니다.
        이것 이외에도 대안이 있을까요? 내부 가변성 (interior mutability)을 제공하는 RefCell 혹은 UnsafeCell 을 통해 immutable한 정적 변수를 만드는 것은 어떨까요?
        이 타입들은 중요한 이유로 Sync 트레이트를 구현하지 않기에 정적 변수를 선언할 때에는 사용할 수 없습니다.

        Wrong code :
            use lazy_static::lazy_static;

            lazy_static! {
                pub static ref WRITER: Writer = Writer {
                    column_position: 0,
                    color_code: ColorCode::new(Color::Yellow, Color::Black),
                    buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
                };
            }
    */

    /*
        스핀 락 (Spinlocks)

        표준 라이브러리의 Mutex는 동기화된 내부 가변성 (interior mutability)을 제공합니다.
        Mutex는 접근하려는 리소스가 잠겼을 때 현재 스레드를 블로킹 (blocking) 하는 것으로 상호 배제 (mutual exclusion)를 구현합니다.
        우리의 커널은 스레드 블로킹은 커녕 스레드의 개념조차 구현하지 않기에 Mutex를 사용할 수 없습니다.
        그 대신 우리에게는 운영체제 기능이 필요 없는 원시적인 스핀 락 (spinlock)이 있습니다.
        스핀 락은 Mutex와 달리 스레드를 블로킹하지 않고,
        리소스의 잠김이 풀릴 때까지 반복문에서 계속 리소스 취득을 시도하면서 CPU 시간을 소모합니다.
    */

    // 이제 스핀 락을 이용해 전역 변수 WRITER에 안전하게 내부 가변성 (interior mutability) 을 구현할 수 있습니다.
    use spin::Mutex;
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
            column_position: 0,
            color_code: ColorCode::new(Color::Yellow, Color::Black),
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        });
    }

    //println macro_export
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
        WRITER.lock().write_fmt(args).unwrap();
    }

    #[test_case]
    fn test_println_output() {
        let s = "Some test string that fits on a single line";
        println!("{}", s);
        for (i, c) in s.chars().enumerate() {
            let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    }

### src/serial.rs

    //직렬포트용 기본 드라이버
    use uart_16550::SerialPort;
    use spin::Mutex;
    use lazy_static::lazy_static;

    //static를 사용하여 메서드가 처음 사용할 때 lazy_static가 정확히 한 번만 호출되도록 할 수 있습니다.
    lazy_static! {
        pub static ref SERIAL1: Mutex<SerialPort> = {
            let mut serial_port = unsafe { SerialPort::new(0x3F8) };
            serial_port.init();
            Mutex::new(serial_port)
        };
    }

    #[doc(hidden)]
    pub fn _print(args: ::core::fmt::Arguments) {
        use core::fmt::Write;
        SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
    }

    /// Prints to the host through the serial interface.
    #[macro_export]
    macro_rules! serial_print {
        ($($arg:tt)*) => {
            $crate::serial::_print(format_args!($($arg)*));
        };
    }

    /// Prints to the host through the serial interface, appending a newline.
    #[macro_export]
    macro_rules! serial_println {
        () => ($crate::serial_print!("\n"));
        ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
        ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
            concat!($fmt, "\n"), $($arg)*));
    }

### src/lib.rs

    #![no_std]
    #![cfg_attr(test, no_main)]
    #![feature(custom_test_frameworks)]
    #![test_runner(crate::test_runner)]
    #![reexport_test_harness_main = "test_main"]

    pub mod serial;
    pub mod vga_buffer;

    use core::panic::PanicInfo;

    pub trait Testable {
        fn run(&self) -> ();
    }

    impl<T> Testable for T
    where
        T: Fn(),
    {
        fn run(&self) {
            serial_print!("{}...\t", core::any::type_name::<T>());
            self();
            serial_println!("[ok]");
        }
    }

    pub fn test_runner(tests: &[&dyn Testable]) {
        serial_println!("Running {} tests", tests.len());
        for test in tests {
            test.run();
        }
        exit_qemu(QemuExitCode::Success);
    }

    pub fn test_panic_handler(info: &PanicInfo) -> ! {
        serial_println!("[failed]\n");
        serial_println!("Error: {}\n", info);
        exit_qemu(QemuExitCode::Failed);
        loop {}
    }

    /// Entry point for `cargo test`
    #[cfg(test)]
    #[no_mangle]
    pub extern "C" fn _start() -> ! {
        test_main();
        loop {}
    }

    #[cfg(test)]
    #[panic_handler]
    fn panic(info: &PanicInfo) -> ! {
        test_panic_handler(info)
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

### tests/basic_boot.rs

    #![no_std]
    #![no_main]
    #![feature(custom_test_frameworks)]
    #![reexport_test_harness_main = "test_main"]
    #![test_runner(blog_os::test_runner)]
    // lib.rs에 포함되어 있습니다.

    use core::panic::PanicInfo;

    #[no_mangle] // don't mangle the name of this function
    pub extern "C" fn _start() -> ! {
        test_main();

        loop {}
    }

    #[panic_handler]
    fn panic(info: &PanicInfo) -> ! {
        blog_os::test_panic_handler(info)
    }

    use blog_os::println;

    #[test_case]
    fn test_println() {
        println!("test_println output");
    }

### tests/should_panic.rs

    #![no_std]
    #![no_main]

    use core::panic::PanicInfo;
    use blog_os::{exit_qemu, serial_print, serial_println, QemuExitCode};

    #[no_mangle]
    pub extern "C" fn _start() -> ! {
        should_fail();
        serial_println!("[test did not panic]");
        exit_qemu(QemuExitCode::Failed);
        loop{}
    }

    fn should_fail() {
        serial_print!("should_panic::should_fail...\t");
        assert_eq!(0, 1);
    }

    #[panic_handler]
    fn panic(_info: &PanicInfo) -> ! {
        serial_println!("[ok]");
        exit_qemu(QemuExitCode::Success);
        loop {}
    }