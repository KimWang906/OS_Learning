use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::println;
/*
    CPU가 우리의 새로운 인터럽트 기술자 테이블을 사용하기 위해서는 lidt 명령을 사용하여 로드해야 합니다.
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
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX); // new
        }
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
    처리기는 짧은 오류 메시지를 출력하고 예외 스택 프레임을 덤프하고, 이중 오류 처리기의 오류 코드는 항상 0이므로 인쇄할 필요가 없습니다.
    중단점 처리기의 한 가지 차이점은 이중 오류 처리기가 프로그램의 실행 순서를 변경하여 다른 명령을 실행 수 있도록 하는 것입니다.
    그 이유는 x86_64 아키텍처가 이중 장애 예외로부터의 반환을 허용하지 않기 때문입니다.
*/

// 호출 규약과 함수를 정의합니다(x86-interrupt, double_fault_handler)
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}