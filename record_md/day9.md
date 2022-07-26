# [Chapter 7 : Double faluts(이중 오류) - 2](https://os.phil-opp.com/double-fault-exceptions/)

## Switching Stacks

x86_64 아키텍처는 예외가 발생할 때 미리 정의된 정상 스택으로 전환할 수 있습니다.
이 스위치는 하드웨어에서 발생하므로 CPU가 예외 Stack Frame을 넣기 전에 수행될 수 있습니다.

다음 코드는 Rust와 유사한 언어에서 구성된 테이블입니다.

    struct InterruptStackTable {
        stack_pointers: [Option<StackPointer>; 7],
    }

각 예외 Handler에 대해 IST에서 해당 IDT 항목의 stack_pointers 필드를 통해 Stack을 선택할 수 있습니다.
예를 들어, IST의 첫 번째 Stack을 Double Fault Handler에 사용할 수 있습니다.
그런 다음 Double Fault가 발생할 때마다 CPU가 자동으로 이 Stack으로 전환됩니다.
이 스위치는 어떤 것도 넣기 전에 발생하므로 Triple Fault를 방지할 수 있습니다.

## Global Descriptor Table

GDT(Global Descriptor Table)는 페이징이 사실상의 표준이 되기 전에 메모리 분할에 사용되었던 기술입니다.
커널/사용자 모드 구성이나 TSS 로딩과 같은 다양한 것들을 위해 64비트 모드에서도 여전히 필요합니다.

### GDT 구현

    use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};

    lazy_static! {
        static ref GDT: GlobalDescriptorTable = {
            let mut gdt = GlobalDescriptorTable::new();
            gdt.add_entry(Descriptor::kernel_code_segment());
            gdt.add_entry(Descriptor::tss_segment(&TSS));
            gdt
        };
    }

GDT가 로드되었지만 여전히 Stack Overflow에서 무한 루프가 발생합니다.

### 마지막 단계

<p>
문제는 세그먼트와 TSS 레지스터에 여전히 이전 GDT의 값이 포함되어 있기 때문에 GDT 세그먼트가 아직 활성화되어 있지 않다는 것입니다.<br>
또한 새 Stack을 사용하도록 이중 오류 IDT 항목을 수정해야 합니다.<br>
<br>
요약하면 다음을 수행해야 합니다.<br>
<br>
1. Code Segment Register를 다시 로드합니다.<br>
<br>
우리는 GDT를 변경했기 때문에 Code Segment Register인 <b>cs</b>를 다시 로드해야 합니다.<br>
이전 Segment Selector가 이제 다른 GDT descriptor(예: TSS descriptor)를 가리킬 수 있기 때문에 이 작업이 필요합니다.<br>
<br>
2. TSS 로드를 로드합니다.<br>
TSS 선택기가 포함된 GDT를 로드했지만 CPU에 해당 TSS를 사용해야 한다고 알려야 합니다.<br>
<br>
3. IDT 항목을 업데이트합니다.<br>
TSS가 로드되는 즉시 CPU가 유효한 IST(Interrupt Stack Table)에 액세스할 수 있습니다.<br>
그런 다음 CPU에 Double Fault IDT 항목을 수정하여 새로운 Double Fault Stack을 사용해야 한다고 말할 수 있습니다.
</p>

### GDT Source Code

    //gdt.rs
    use x86_64::VirtAddr;
    use x86_64::structures::tss::TaskStateSegment;
    use lazy_static::lazy_static;

    pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

    /*  처음 두 단계에서는 gdt::init 함수의 code_selector 및 tss_selector 변수에 Access해야 합니다.
        새로운 Selector Struct를 통해 static의 일부를 구성함으로써 이를 달성할 수 있습니다.
    */
    lazy_static! {
        static ref TSS: TaskStateSegment = {
            let mut tss = TaskStateSegment::new();
            tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
                const STACK_SIZE: usize = 4096 * 5;
                static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

                let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
                let stack_end = stack_start + STACK_SIZE;
                stack_end
            };
            tss
        };
    }

    use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};
    use x86_64::structures::gdt::SegmentSelector;

    lazy_static! {
        static ref GDT: (GlobalDescriptorTable, Selectors) = {
            let mut gdt = GlobalDescriptorTable::new();
            let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
            let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
            (gdt, Selectors { code_selector, tss_selector })
        };
    }

    struct Selectors {
        code_selector: SegmentSelector,
        tss_selector: SegmentSelector,
    }

    // 이제 선택기를 사용하여 cs Segment Register를 다시 로드하고 다음 TSS를 로드할 수 있습니다.
    pub fn init() {
        use x86_64::instructions::tables::load_tss;
        use x86_64::instructions::segmentation::{CS, Segment};
        
        GDT.0.load();
        unsafe {
            CS::set_reg(GDT.1.code_selector);
            load_tss(GDT.1.tss_selector);
        }
    }

    //interrupt.rs
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

## 결과

### Qemu 화면

<p align="center"><img src="/record_image/day_9_stack_overflow_result_code.png"></p>

### 테스트케이스 결과

<p align="center"><img src="/record_image/day_9_result_console.png"></p>

## Source code

### [Interrupt](/src/interrupts.rs)

### [lib](/src/lib.rs)

### [main](/src/main.rs)

### [gdt](/src/gdt.rs)

### [testcase](/src/stack_overflow.rs)
