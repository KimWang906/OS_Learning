# [Chapter 8 : Hardware interrupt - 1](https://os.phil-opp.com/hardware-interrupts/)

## 목표

<p>
이번 목표는 Hardware interrupt를 CPU로 올바르게 전달하도록 프로그래밍 가능한 Interrupt controller를 설정합니다.<br>
이러한 Interrupt를 처리하기 위해 Exception handler에 대해 했던 것처럼 Interrupt Descriptor Table에 새 항목을 추가합니다.<br>
주기적인 Timer interrupt를 받는 방법과 키보드에서 입력을 받는 방법을 배웁니다.
</p>

## 개요

<p>
인터럽트는 연결된 하드웨어 장치에서 CPU에 알리는 방법을 제공합니다.<br>
따라서 Kernel이 주기적으로 키보드에서 새 문자를 확인하도록 하는 대신(polling 이라고 하는 프로세스), 키보드는 각 키 누름을 커널에 알릴 수 있습니다.<br>
Kernel은 어떤 일이 발생했을 때만 작동하면 되므로 훨씬 더 효율적입니다.<br>
또한 Kernel이 다음 poll뿐만 아니라 즉시 반응할 수 있기 때문에 더 빠른 반응 시간을 허용합니다.<br>
<br>
모든 하드웨어 장치를 CPU에 직접 연결하는 것은 불가능합니다.<br>
대신 별도의 Interrupt controller가 모든 장치의 Interrupt를 집계한 다음 CPU에 알립니다.
</p>

                         ____________             _____
    Timer ------------> |            |           |     |
    Keyboard ---------> | Interrupt  |---------> | CPU |
    Other Hardware ---> | Controller |           |_____|
    Etc. -------------> |____________|

<p>
대부분의 Interrupt controller는 프로그래밍 가능합니다.<br>
즉, Interrupt에 대해 서로 다른 우선 순위 수준을 지원합니다.<br>
예를 들어, 이를 통해 정확한 시간 기록을 보장하기 위해 Keyboard interrupt보다<br>
Timer interrupt에 더 높은 우선순위를 부여할 수 있습니다.<br>
<br>
Exception와 달리 Hardware interrupt는 비동기적으로 발생합니다.<br>
이는 실행된 코드와 완전히 독립적이며 언제든지 발생할 수 있음을 의미합니다.<br>
따라서 우리는 갑자기 모든 잠재적 동시성 관련 버그와 함께 Kernel에 동시성 형태를 갖게 됩니다.<br>
Rust의 엄격한 소유권 모델은 변경 가능한 전역 상태를 금지하기 때문에 여기에서 도움이 됩니다.<br>
</p>

## 8259 PIC

<p>
Intel 8259 는 1976년에 도입된 PIC(Programmable Interrupt Controller)입니다.<br>
오랫동안 새로운 APIC 로 교체 되었지만 해당 인터페이스는 이전 버전과의 호환성을 위해 현재 시스템에서 여전히 지원됩니다.<br>
<br>
8259에는 8개의 인터럽트 라인과 CPU와 통신하기 위한 여러 라인이 있습니다.<br>
당시의 일반적인 시스템에는 2개의 8259 PIC 인스턴스가 장착되어 있었습니다.<br>
하나는 기본 PIC이고 다른 하나는 기본 PIC의 인터럽트 라인 중 하나에 연결되어 있습니다.<br>
</p>

                         ____________                          ____________
    Real Time Clock --> |            |   Timer -------------> |            |
    ACPI -------------> |            |   Keyboard-----------> |            |      _____
    Available --------> | Secondary  |----------------------> | Primary    |     |     |
    Available --------> | Interrupt  |   Serial Port 2 -----> | Interrupt  |---> | CPU |
    Mouse ------------> | Controller |   Serial Port 1 -----> | Controller |     |_____|
    Co-Processor -----> |            |   Parallel Port 2/3 -> |            |
    Primary ATA ------> |            |   Floppy disk -------> |            |
    Secondary ATA ----> |____________|   Parallel Port 1----> |____________|

<p>
이 그래픽은 인터럽트 라인의 일반적인 할당을 보여줍니다.<br>
15개 라인의 대부분이 고정 매핑을 가지고 있음을 알 수 있습니다.<br>
예를 들어, 보조 PIC의 라인 4는 마우스에 할당됩니다.<br>
<br>
각 컨트롤러는 2개의 I/O 포트 , 1개의 "명령" 포트 및 1개의 "데이터" 포트 를 통해 구성할 수 있습니다.<br>
기본 컨트롤러의 경우 이러한 포트는 <b>0x20</b>(명령) 및 <b>0x21</b>(데이터)입니다.<br>
보조 컨트롤러의 경우 <b>0xa0</b>(명령) 및 <b>0xa1</b>(데이터)입니다.
</p>

PIC 구성 방법에 대한 자세한 내용은 [osdev.org](https://wiki.osdev.org/8259_PIC)의 내용을 참조하세요.

### Qemu 화면

<p align="center"><img src="/readme_src/qemu-hardware-timer-dots.gif"></p>

## Source code

### [Interrupt](/src/interrupts.rs)

### [lib](/src/lib.rs)