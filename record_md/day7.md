# [Chapter 5 : CPU Exceptions(예외) - 2](https://os.phil-opp.com/cpu-exceptions/)

## 목표

    CPU 예외는 잘못된 메모리 주소에 액세스하거나 0으로 나눌 때와 같이 다양한 잘못된 상황에서 발생합니다.
    이에 대응하기 위해 핸들러 기능을 제공 하는 Interrupt Descriptor Table을 설정해야 합니다.
    Chapter 5의 목표는 커널은 중단점 예외를 포착하고 이후에 정상적인 실행을 재개하는 것입니다.

## IDT(Interrupt Descriptor Table의 유형)

<p>자체 IDT 유형을 생성하는 대신 x86_64 crate의 InterruptDescriptorTable 구조를 사용합니다.</p>

```rs
#[repr(C)]
pub struct InterruptDescriptorTable {
    pub divide_by_zero: Entry<HandlerFunc>,
    pub debug: Entry<HandlerFunc>,
    pub non_maskable_interrupt: Entry<HandlerFunc>,
    pub breakpoint: Entry<HandlerFunc>,
    pub overflow: Entry<HandlerFunc>,
    pub bound_range_exceeded: Entry<HandlerFunc>,
    pub invalid_opcode: Entry<HandlerFunc>,
    pub device_not_available: Entry<HandlerFunc>,
    pub double_fault: Entry<HandlerFuncWithErrCode>,
    pub invalid_tss: Entry<HandlerFuncWithErrCode>,
    pub segment_not_present: Entry<HandlerFuncWithErrCode>,
    pub stack_segment_fault: Entry<HandlerFuncWithErrCode>,
    pub general_protection_fault: Entry<HandlerFuncWithErrCode>,
    pub page_fault: Entry<PageFaultHandlerFunc>,
    pub x87_floating_point: Entry<HandlerFunc>,
    pub alignment_check: Entry<HandlerFuncWithErrCode>,
    pub machine_check: Entry<HandlerFunc>,
    pub simd_floating_point: Entry<HandlerFunc>,
    pub virtualization: Entry<HandlerFunc>,
    pub security_exception: Entry<HandlerFuncWithErrCode>,
    // some fields omitted
}
```

## HandlerFunc Type

```rs
type HandlerFunc = extern "x86-interrupt" fn(_: InterruptStackFrame);
```

### extern keyword

extern 키워드는 호출 규약과 함수를 정의하며 C 코드(<b>extern "C" fn</b>)와 통신하는 데 자주 사용됩니다.

### 어셈블리 용어

<p>
Push(push) : 값을 넣습니다.(Stack Pointer 1 증가)<br>
<br>
Jump(jmp) : 특정 명령어나 주소로 옮깁니다.<br>
<br>
Pop(pop) : 값을 뺍니다.(Stack Pointer 1 감소)<br>
<br>
레지스터 e, r :<br>
e는 x86(32)비트 CPU에서 사용되는 레지스터이고<br>
r은 x64(64)비트 CPU에서 사용되는 레지스터입니다.
</p>

## 인터럽트 호출 규칙

<p>
예외는 함수 호출과 매우 유사합니다.<br>
CPU는 호출된 함수의 첫 번째 명령으로 점프하여 실행합니다.<br>
이후 CPU는 반환 주소로 점프하여 상위 함수의 실행을 계속합니다.<br>
<br>
그러나, 함수 호출은 컴파일러가 삽입한 <b>call</b> 명령에 의해 자발적으로 호출되고 예외는 어떤 명령에서도 발생할 수 있습니다.<br>
이 차이의 결과를 이해하기 위해서는 함수 호출을 좀 더 자세히 살펴볼 필요가 있습니다.<br>
<br>
호출 규칙은 함수 호출의 세부 정보를 지정합니다.<br>
예를 들어, 함수 매개변수가 어디에 위치하는지(예: 레지스터 또는 스택에 위치)와 결과가 어떻게 반환되는지 지정합니다.<br>
x86_64 리눅스에서는 (System VABI에 지정된) C 함수에 다음과 같은 규칙이 적용됩니다.
</p>
<ul>
    <li>레지스터 <b>rdi, rsi, rdx, rcx, r8, r9</b>에서 처음 6개의 정수 인수가 전달됩니다.</li>
    <li>추가 인수가 스택에서 전달됩니다.</li>
    <li>결과가 <b>rax</b> 및 <b>rdx</b>로 반환됩니다.</li>
</ul>
<p>
Rust는 C ABI를 따르지 않기 때문에,(사실 아직 Rust ABI도 존재하지 않습니다.)<br>
이러한 규칙은 <b>extern "C" fn</b>으로 선언된 함수에만 적용됩니다.<br>
</p>

## 보존 레지스터와 스크래치 레지스터

<p>
호출 규약은 레지스터를 Preserved Register(보존 레지스터)와 Scratch Register(스크래치 레지스터)의 두 부분으로 나뉩니다.<br>
<br>
Preserved Register의 값은 함수 호출에서 변경되지 않은 상태로 유지되어야 합니다.<br>
따라서 호출된 함수("호출자")는 반환하기 전에 원래 값을 복원하는 경우에만 레지스터를 덮어쓸 수 있고, 이러한 레지스터를 "호출자 저장"이라고 합니다.<br>
일반적인 패턴은 이 레지스터들을 함수의 시작 부분에서 스택에 저장하고 반환하기 직전에 복원하는 것입니다.<br>
<br>
반대로 호출된 함수는 Scratch Register를 제한 없이 덮어쓸 수 있고,<br>
호출자가 함수 호출에 걸쳐 스크래치 레지스터의 값을 보존하려면 함수 호출 전에 백업하고 복원해야 합니다.<br>
(예: 스택에 밀어넣기)<br>
그래서 Scratch Register는 호출자에 의해 저장됩니다.
</p>
<p>x86_64에서 C 호출 규약은 다음과 같은 보존 레지스터와 스크래치 레지스터를 지정합니다.</p>
<p align="center"><img src="/readme_src/P_S_Registers.png"></p>
<p>
컴파일러는 이러한 규칙을 알고 있으므로 그에 따라 코드를 생성합니다.<br>
대부분의 함수는 <b>push rbp</b>로 시작하는데, <b>push rbp</b>는 스택에 <b>rbp</b>를 백업하는 것이 그 예시 입니다.
</p>

## 모든 보존 레지스터

<p>
함수 호출과는 달리 모든 명령에서 예외가 발생할 수 있습니다.<br>
대부분의 경우 컴파일 시 생성된 코드가 예외를 발생시킬지 조차 알 수 없습니다.<br>
예를 들어, 컴파일러는 명령이 스택 오버플로 또는 페이지 오류를 유발하는지 알 수 없습니다.<br>
<br>
예외가 언제 발생하는지 모르기 때문에 이전에는 어떤 레지스터도 백업할 수 없습니다.<br>
이는 예외 처리기에 대해 호출자가 저장한 레지스터에 의존하는 호출 규칙을 사용할 수 없음을 의미합니다.<br>
대신, 우리는 모든 레지스터를 보존하는 호출 규약 수단이 필요하다.<br>
x86-interrupt 호출 규약은 해당 조건에 맞는 호출 규약이므로 모든 레지스터 값이 함수 반환 시 원래 값으로 복원되도록 보장합니다.<br>
<br>
모든 레지스터가 함수 입력 시 스택에 저장된다는 의미는 아닙니다.<br>
대신 컴파일러는 함수에 의해 덮어쓰는 레지스터만 백업합니다.<br>
이러한 방식으로 매우 효율적인 코드는 몇 개의 레지스터만 사용하는 짧은 함수들을 위해 생성될 수 있습니다.
</p>

## 인터럽트 스택 프레임

<p>
정상 함수 호출 시(call 명령 사용) CPU는 목표 함수로 이동하기 전에 반환 주소를 넣습니다.<br>
(ret 명령을 사용하여) 함수 반환 시 CPU는 이 반환 주소를 팝업하고 이 주소로 점프합니다.<br>
<br>
따라서 일반 함수 호출의 스택 프레임은 다음과 같습니다.
</p>
<p align="center"><img src="/readme_src/function-stack-frame.svg"></p>
<p>
그러나 예외 및 인터럽트 핸들러의 경우, 반환 주소를 푸시하는 것만으로는 충분하지 않을 수 있는데<br>
인터럽트 핸들러는 종종 다른 컨텍스트(Stack Pointer, CPU flags 등)에서 실행되기 때문입니다.<br>
<br>
대신 인터럽트가 발생하면 CPU는 다음 단계를 수행합니다.
</p>
<p>
<b>1. Stack Pointer 정렬</b><br>
인터럽트는 어떤 명령에서도 발생할 수 있고, Stack Pointer도 값을 가질 수 있습니다.<br>
그러나 일부 CPU 명령(예: 일부 SSE 명령) Stack Pointer가 16바이트 경계에 정렬되어야 하므로 CPU는 인터럽트 직후에 정렬을 수행합니다.<br>
<br>
<b>2. Switching stacks(경우에 따라)</b><br>
스택 스위치는 사용자 모드 프로그램에서 CPU 예외가 발생하는 경우와 같이 CPU 권한 수준이 변경될 때 발생합니다.<br>
인터럽트 스택 테이블(Interrupt Stack Table)을 사용하여 특정 인터럽트를 위한 스택 스위치를 구성할 수도 있습니다.<br>
<br>
<b>3. 이전 Stack Pointer 밀어넣기</b><br>
CPU는 인터럽트가 발생한 시점(정렬 전)에 Stack Pointer(<b>rsp</b>)와 Stack Segment(<b>ss</b>) 레지스터의 값을 넣습니다.<br>
이렇게 하면 인터럽트 핸들러에서 돌아올 때 원래 Stack Pointer를 복원할 수 있습니다.<br>
<br>
<b>4. RPLAGS 레지스터 밀어넣기 및 업데이트</b><br>
RFLAGS 레지스터에는 다양한 제어 및 상태 비트가 포함되어 있습니다.<br>
Interrupt Entry에서 CPU는 일부 비트를 변경하고 이전 값을 넣습니다.<br>
<br>
<b>5. Instruction Pointer 넣기</b><br>
인터럽트 핸들러 기능으로 이동하기 전에 CPU는 명령 포인터(<b>rip</b>)와 코드 세그먼트(<b>cs</b>)를 넣습니다.<br>
이는 정상 함수 호출의 반환 주소 푸시와 같습니다.<br>
<br>
<b>6. 오류 코드 넣기(일부 예외)</b><br>
페이지 장애와 같은 일부 특정 예외의 경우 CPU는 예외의 원인을 설명하는 오류 코드를 넣습니다.<br>
<br>
<b>7. Interrupt Handler 호출</b><br>
CPU는 IDT의 해당 필드에서 인터럽트 핸들러 함수의 주소와 세그먼트 설명자를 읽습니다.<br>
그런 다음 값을 <b>rip</b> 및 <b>cs</b> 레지스터에 로드하여 이 핸들러를 호출합니다.
</p>

<p>Interrupt Stack Frame은 다음과 같습니다.</p>
<p align="center"><img src="/readme_src/exception-stack-frame.svg"></p>
<b>x86_64</b> crate에서 인터럽트 스택 프레임은 <b><a href="https://docs.rs/x86_64/0.14.2/x86_64/structures/idt/struct.InterruptStackFrame.html">InterruptStackFrame</a></b> 구조로 표현됩니다.
<b>&mut</b>로 인터럽트 핸들러에 전달되며 예외 원인에 대한 추가 정보를 검색하는 데 사용할 수 있습니다.
일부 예외만 오류 코드를 넣기 때문에 구조에 오류 코드 필드가 없습니다.
이러한 예외는 별도의 <b><a href="https://docs.rs/x86_64/0.14.2/x86_64/structures/idt/type.HandlerFuncWithErrCode.html">HandlerFuncWithErrCode</a></b> 함수 유형을 사용하며 여기에는 <b>error_code</b> 인수가 추가됩니다.

## 비하인드

x86 인터럽트 호출 규칙은 예외 처리 프로세스의 거의 모든 복잡한 세부 사항을 숨기는 강력한 추상화입니다.
하지만 때때로 프로세스에서 무슨 일이 일어나고 있는지 아는 것은 유용합니다.

다음은 x86 인터럽트 호출 규칙이 처리하는 사항의 간략한 개요입니다.

<b>인수 검색</b>
대부분의 호출 규칙은 인수가 레지스터에서 전달될 것으로 예상합니다.
예외 처리기는 레지스터 값을 스택에 백업하기 전에 덮어쓰지 않아야 하므로 이 작업을 수행할 수 없습니다.
대신, <b>x86-interrupt</b> 호출 규약은 인수들이 이미 특정 오프셋의 스택에 있다는 것을 인식합니다.

<b>iretq를 사용하여 Return</b>
인터럽트 스택 프레임은 일반 함수 호출의 스택 프레임과 완전히 다르기 때문에,
일반 ret 명령을 통해 핸들러 함수에서 돌아올 수 없습니다.
대신 iretq 명령을 사용해야 합니다.

<b>오류 코드 처리</b>
일부 예외를 위해 푸시되는 오류 코드는 상황을 훨씬 더 복잡하게 만듭니다.
스택 정렬이 변경되며(다음 Pointer 참조), 반환하기 전에 스택에서 제거해야 합니다.
<b>x86-interrupt</b> 호출 규칙은 모든 복잡성을 처리합니다.
그러나 어떤 핸들러 함수가 어떤 예외에 사용되는지 모르기 때문에 함수 인수의 수에서 그 정보를 추론해야 합니다.
이는 프로그래머가 각 예외에 대해 올바른 기능 유형을 사용할 책임이 있음을 의미합니다.
다행히 <b>x86_64</b> create로 정의된 <b>InterruptDescriptorTable</b> 유형은 올바른 함수 유형이 사용되도록 보장합니다.

<b>Stack 정렬</b>
16바이트 스택 정렬을 요구하는 일부 명령어(특히 SSE 명령어)가 있습니다.
CPU는 예외가 발생할 때마다 이 정렬을 보장하지만 일부 예외의 경우 나중에 오류 코드를 넣으면 다시 삭제합니다.
<b>x86-interrupt</b> 호출 규약은 이 경우에 스택을 재배치함으로써 이것을 처리합니다.

### [breakpoint](https://eli.thegreenplace.net/2011/01/27/how-debuggers-work-part-2-breakpoints)

breakpoint 예외는 일반적으로 디버거에서 사용됩니다.
사용자가 breakpoint를 설정하면 디버거가 해당 명령을 해당 명령으로 덮어쓰므로 <b>int3</b> CPU가 해당 행에 도달하면 중단점 예외가 발생합니다.
사용자가 프로그램을 계속하기를 원하면 디버거는 <b>int3</b> 명령을 원래 명령으로 다시 교체하고 프로그램을 계속합니다.

## 실행 결과

### Qemu

<p align="center"><img src="/record_image/day_7_interrupt_breakpoint.png"></p>

### Result

<p align="center"><img src="/record_image/day_7_interrupt_result.png"></p>

## Source code

### [Interrupt](/src/interrupts.rs)

### [lib](/src/lib.rs)

### [main](/src/main.rs)
