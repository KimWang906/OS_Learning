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

pub static WRITER: Writer = Writer {
    column_position: 0,
    color_code: ColorCode::new(Color::Yellow, Color::Black),
    buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
};

//최종적으로 화면에 출력되는 문자열
pub fn print_something() {
    /*
        우선 메모리 주소 0xb8000을 가리키는 새로운 Writer 인스턴스를 생성합니다.
        이를 구현한 코드가 다소 난해하게 느껴질 수 있으니 단계별로 나누어 설명드리겠습니다.

        먼저 정수 0xb8000을 읽기/쓰기 모두 가능한 (mutable) 포인터로 타입 변환합니다.
        그 후 * 연산자를 통해 이 포인터를 역참조 (dereference) 하고 &mut를 통해
        즉시 borrow 함으로써 해당 주소에 저장된 값을 변경할 수 있는 레퍼런스 (mutable reference)를 만듭니다.
        여기서 Rust 컴파일러는 포인터의 유효성 및 안전성을 보증할 수 없기에, unsafe 블록을 사용해야만 포인터를 Reference로 변환할 수 있습니다.

        그 다음 Writer 인스턴스에 바이트 b'H'를 적습니다. 
        접두사 b는 ASCII 문자를 나타내는 바이트 상수 (literal) 를 생성합니다.
    */
    use core::fmt::Write;
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    writer.write_byte(b'H');
    writer.write_string("ello! ");
    write!(writer, "The numbers are {} and {}", 42, 1.0/3.0).unwrap();
}

