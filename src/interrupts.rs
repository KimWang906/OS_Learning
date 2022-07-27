use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::println;
use crate::print;
/*
    CPU가 우리의 새로운 Interrupt Descriptor Table을 사용하기 위해서는 lidt 명령을 사용하여 로드해야 합니다.
    x86_64의 InterruptDescriptorTable 구조는 이를 위한 로드 메서드 함수를 제공합니다.
*/

/*
    로드 메서드는 프로그램의 전체 런타임에 유효한 참조인 &'static self'를 예상합니다.
    그 이유는 CPU가 우리가 다른 IDT를 로드할 때까지 모든 인터럽트에서 이 테이블에 액세스하기 때문입니다.
    따라서 static보다 짧은 수명을 사용하면 use-after-free 버그가 발생할 수 있습니다.
    pub fn init_idt() {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.load();
    }
*/

/*
이 문제를 해결하기 위해서, 'static lifetime이 있는 곳에 idt를 저장해야 합니다.
이를 위해 Box를 사용하여 힙에 IDT를 할당하고 'static reference로 변환할 수 있지만 OS 커널을 작성하고 있으므로 힙이 없습니다.

대안으로 IDT를 static으로 저장할 수 있습니다.

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub fn init_idt() {
    IDT.breakpoint.set_handler_fn(breakpoint_handler);
    IDT.load();
}
이 수정은 오류 없이 컴파일되지만 Rust의 표현과는 거리가 멉니다.
static mut는 데이터 경쟁에 매우 취약하므로 액세스할 때마다 unsafe블록이 필요합니다.
*/
use lazy_static::lazy_static;
use crate::gdt;

/*
    set_cs를 사용하여 코드 세그먼트 레지스터를 다시 로드하고 load_tss를 사용하여 TSS를 로드합니다. 
    함수가 안전하지 않은 것으로 확인되었으므로, 함수를 호출하려면 unsafe 블록이 필요합니다.
    잘못된 Selector를 로드하여 메모리 안전을 해칠 수 있기 때문입니다.
*/

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()]
                .set_handler_fn(timer_interrupt_handler);
        idt
    };
}


pub fn init_idt() {
    IDT.load();
}

// 호출 규약과 함수를 정의합니다(x86-interrupt, breakpoint_handler)
extern "x86-interrupt" fn breakpoint_handler( 
    stack_frame: InterruptStackFrame)
{
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

/*
    처리기는 짧은 오류 메시지를 출력하고 예외 Stack Frame을 Dump하고, double fault handler의 오류 코드는 항상 0이므로 인쇄할 필요가 없습니다.
    breakpoint handler의 한 가지 차이점은 double fault handler가 프로그램의 실행 순서를 변경하여 다른 명령을 실행 수 있도록 하는 것입니다.
    그 이유는 x86_64 아키텍처가 double fault exception로부터의 반환을 허용하지 않기 때문입니다.
*/

// 호출 규약과 함수를 정의합니다(x86-interrupt, double_fault_handler)
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

// 호출 규약과 함수를 정의합니다(x86-interrupt, timer_interrupt_handler)
/*
    extern "x86-interrupt" fn timer_interrupt_handler(
        _stack_frame: InterruptStackFrame)
    {
        print!(".");
    }

    타이머 인터럽트 핸들러에서 화면에 점을 인쇄합니다.
    타이머 인터럽트가 주기적으로 발생하기 때문에 각 타이머 틱에 점이 표시될 것으로 예상됩니다.
    그러나 실행하면 단일 점만 인쇄되는 것을 볼 수 있습니다.

    그 이유는 PIC가 인터럽트 핸들러로부터 명시적인 "인터럽트 종료"(EOI) 신호를 기다리고 있기 때문입니다.
    이 신호는 컨트롤러에 인터럽트가 처리되었고 시스템이 다음 인터럽트를 수신할 준비가 되었음을 알려줍니다.
    그래서 PIC는 여전히 첫 번째 타이머 인터럽트를 처리하느라 바쁘다고 생각하고 다음 인터럽트를 보내기 전에 EOI 신호를 기다립니다.

    EOI를 보내기 위해 정적 PICS구조체를 다시 사용합니다.

    extern "x86-interrupt" fn timer_interrupt_handler(
        _stack_frame: InterruptStackFrame)
    {
        print!(".");

        unsafe {
            PICS.lock()
                .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
        }
    }

    notify_end_of_interrupt는 기본 또는 보조 PIC가 인터럽트를 전송했는지 확인한 다음 명령 및 데이터 포트를 사용하여 각 컨트롤러에 EOI 신호를 전송합니다.
    보조 PIC가 인터럽트를 전송한 경우 보조 PIC가 기본 PIC의 입력 라인에 연결되어 있으므로 두 PIC에 모두 알림을 보내야 합니다.
*/
extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame)
{
    print!(".");

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

use pic8259::ChainedPics;
use spin;

/*
    PIC의 기본 구성은 CPU에 0–15 범위의 Interrupt vector number를 보내기 때문에 사용할 수 없습니다.
    현재 저 범위의 number는 이미 CPU exception에 의해 점유되고 있습니다.
    예를 들어 숫자 8은 double fault에 해당합니다.
    이 겹치는 문제를 해결하려면 PIC 인터럽트를 다른 번호로 다시 매핑해야 합니다.
    실제 범위는 예외와 겹치지 않는 한 중요하지 않지만 일반적으로 32-47 범위가 선택됩니다.
    이는 32개의 예외 슬롯 이후 첫 번째 여유 번호이기 때문입니다.
*/

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/*
    위에서 언급한 바와 같이 사진의 오프셋을 32-47 범위로 설정합니다.
    Chained Pics 구조를 Mutex로 감싸면 다음 단계에서 필요한 (잠금 방법을 통해) 안전한 가변 액세스를 얻을 수 있습니다.
    그러나 잘못된 오프셋으로 인해 정의되지 않은 동작이 발생할 수 있으므로 ChainedPics::new 함수는 안전하지 않습니다.
*/

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}