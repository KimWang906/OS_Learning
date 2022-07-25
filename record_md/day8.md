# [Chapter 7 : Double faluts(이중 오류) - 1](https://os.phil-opp.com/double-fault-exceptions/)

## 목표

    Double faults는 간단히 말해 CPU가 예외 처리기를 호출하지 못할 때 발생하는 특별한 예외입니다. 
    해당 예외를 처리하면 시스템 재설정의 원인이 되는 치명적인 삼중 오류를 방지할 수 있습니다.
    모든 경우의 삼중 오류를 방지하기 위해 별도의 커널 스택에서 이중 오류를 확인하도록 인터럽트 스택 테이블을 설정할 것입니다.

## 3중 오류 발생시키기

### 오류 유발 코드

    #[no_mangle]
    pub extern "C" fn _start() -> ! {
        println!("Hello World{}", "!");

        blog_os::init();

        // trigger a page fault
        unsafe {
            *(0xdeadbeef as *mut u64) = 42;
        };

        // as before
        #[cfg(test)]
        test_main();

        println!("It did not crash!");
        loop {}
    }

<p>
<b>unsafe</b>를 사용하여 잘못된 주소 <b>0xdeadbeef</b>에 씁니다.<br>
가상 주소가 Page Table의 실제 주소에 매핑되지 않아 Page Fault가 발생합니다.<br>
IDT에 Page Fault Handler를 등록하지 않아 Double Fault가 발생합니다.<br>
<br>
지금 커널을 시작하면 끝없는 무한 재부팅에 들어가는 것을 볼 수 있습니다.<br>
<br>
무한 재부팅의 이유는 다음과 같습니다.<br>
<br>
<b>1.</b> CPU가 <b>0xdeadbeef<b>에 쓰려고 하면 페이지 오류가 발생합니다.<br>
<b>2.</b> CPU는 IDT의 해당 항목을 보고 Handler 기능이 지정되지 않은 것을 확인했기 때문에 Page Fault Handler를 호출할 수 없으며 Double Fault가 발생합니다.<br>
<b>3.</b> CPU는 Double Fault Handler의 IDT 항목을 살펴보지만 이 항목에서는 Handler 기능도 지정하지 않아 Triple Fault가 발생합니다.<br>
<b>4.</b> Triple Fault는 치명적이고, QEMU(가상환경)는 대부분의 실제 하드웨어처럼 그것에 반응하고 System Reset을 시작합니다.
</p>

### 무한 재부팅 화면

<p align="center"><img src="/record_image/day_8_triple_fault_screen.png"></p>

## 2중 오류의 원인

<p>
특별한 경우를 살펴보기 전에, 우리는 Double Fault의 정확한 원인을 알 필요가 있습니다.<br>
<br>
위에서 우리는 상당히 모호한 정의를 사용했습니다.
</p>

    Double fault는 CPU가 예외 처리기를 호출하지 못할 때 발생하는 특별한 예외입니다.

<p>
호출 실패가 정확히 무엇을 의미할까요?<br>
핸들러가 없다고요? 핸들러가 교체됐다구요?<br>
만약 핸들러가 스스로 예외를 발생시킨다면 어떻게 될까요?<br>
<br>
예를 들어, 다음과 같은 경우 어떻게 될까요?<br>
<br>
<b>1.</b> breakpoint 예외가 발생하지만 해당 핸들러 기능이 Swap Out(종료)되었습니다.<br>
<b>2.</b> Page Fault가 발생했지만 Page Fault Handler가 종료)되었습니다.<br>
<b>3.</b> Handler를 0으로 나누면 breakpoint 예외가 발생하지만 breakpoint handler가 Swap Out(종료)되었습니다.<br>
<b>4.</b> 우리의 커널이 Stack Overflow가 일어나고 Guard Page가 작동합니다.<br>
<br>
다행히도 AMD64 매뉴얼( PDF )에는 정확한 정의가 있습니다.
</p>
<p align="center"><img src="/readme_src/double_fault_list.png"></p>
<p>
이 표의 도움으로 위의 질문 중 처음 세 가지에 답할 수 있습니다.<br>
<br>
<b>1.</b> breakpoint 예외가 발생하고 해당 핸들러 함수가 Swap Out(종료)되면 Page Fault가 발생하고 Page Fault Handler가 호출됩니다.<br>
<b>2.</b> Page Fault가 발생하고 Page Fault Handler가 교체되면 Double Fault가 발생하고 Double Fault Handler가 호출됩니다.<br>
<b>3.</b> Divide-by-zero Handler가 breakpoint 예외를 발생시키면 CPU는 breakpoint Handler를 호출하려고 시도합니다.<br>
   breakpoint Handler가 교체되면 Page Fault가 발생하고 Page Fault Handler가 호출됩니다.
</p>

## Kernel Stack Overflow

<p>
Guard Page는 Stack의 맨 아래에 있는 특별한 Memory Page로 Stack Overflow를 탐지할 수 있습니다.<br>
페이지는 실제 프레임에 매핑되지 않으므로 페이지에 액세스하는 경우 다른 메모리가 자동으로 손상되는 대신 Page Fault가 발생합니다.<br>
부트 로더는 Kernel Stack에 대한 Guard Page를 설정하므로 Stack Overflow로 인해 Page Fault가 발생합니다.<br>
<br>
Page Fault가 발생하면 CPU는 IDT에서 Page Fault Handler를 조회하고 인터럽트 Stack Frame을 Stack으로 넣으려고 합니다.<br>
그러나 현재 Stack Pointer는 여전히 존재하지 않는 Guard Page를 가리킵니다.<br>
따라서 두 번째 Page Fault가 발생하여 Double Fault가 발생합니다.(위 표에 따를 경우에)<br>
<br>
그래서 CPU는 지금 Double Fault Handler를 호출하려고 합니다.<br>
그러나 Double Fault 시 CPU는 예외 Stack Frame도 넣으려고 합니다.<br>
Stack Pointer는 여전히 Guard Page를 가리키기 때문에 세 번째 Page Fault가 발생하여 Triple Fault 및 시스템 재부팅이 발생합니다.<br>
따라서 현재 Double Fault Handler는 이 경우 Triple Fault를 피할 수 없습니다.
</p>

### Stack Overflow Code

<p align="center"><img src="/record_image/day_8_stack_overflow_code.png"></p>
