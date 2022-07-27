# 최소한의 기능을 담은 Kernel

## 참고 자료

    https://os.phil-opp.com/ko/

## 주의

    저는 현재 Linux OS를 사용하고 있기 때문에 다른 개발환경에서 참고하실 때 어려움이 있을 수 있습니다.

## Recording Notes

[Day 1 Recording note](./record_md/day1.md)

[Day 2 Recording note](./record_md/day2.md)

[Day 3 Recording note](./record_md/day3.md)

[Day 4 Recording note](./record_md/day4.md)

[Day 5 Recording note](./record_md/day5.md)

[Day 6 Recording note](./record_md/day6.md)

[Day 7 Recording note](./record_md/day7.md)

[Day 8 Recording note](./record_md/day8.md)

[Day 9 Recording note](./record_md/day9.md)

[Day 10 Recording note](./record_md/day10.md)

## 컴퓨터의 부팅 과정

    전원이 켜졌을 때 컴퓨터가 맨 처음 하는 일은 바로 마더보드의 롬 (ROM)에 저장된 펌웨어 코드를 실행하는 것입니다. 
    이 코드는 시동 자체 시험을 진행하고, 사용 가능한 램 (RAM)을 확인하며, CPU 및 하드웨어의 초기화 작업을 진행합니다. 
    그 후에는 부팅 가능한 디스크를 감지하고 운영체제 커널을 부팅하기 시작합니다.

    x86 시스템에는 두 가지 펌웨어 표준이 존재합니다.
    하나는 “Basic Input/Output System”(BIOS)이고 다른 하나는 “Unified Extensible Firmware Interface” (UEFI) 입니다. 
    BIOS 표준은 구식 표준이지만, 간단하며 1980년대 이후 출시된 어떤 x86 하드웨어에서도 지원이 잘 됩니다. 
    UEFI는 신식 표준으로서 더 많은 기능들을 갖추었지만, 제대로 설정하고 구동시키기까지의 과정이 더 복잡합니다.

### BIOS 부팅

    UEFI 표준으로 동작하는 최신 기기들도 가상 BIOS를 지원하기에, 존재하는 거의 모든 x86 시스템들이 BIOS 부팅을 지원합니다.
    덕분에 하나의 BIOS 부팅 로직을 구현하면 여태 만들어진 거의 모든 컴퓨터를 부팅시킬 수 있습니다.
    동시에 이 방대한 호환성이 BIOS의 가장 큰 약점이기도 한데,
    그 이유는 1980년대의 구식 부트로더들에 대한 하위 호환성을 유지하기 위해 부팅 전에는 항상 CPU를 16비트 호환 모드 (real mode라고도 불림)로 설정해야 하기 때문입니다.

    이제 BIOS 부팅 과정의 첫 단계부터 살펴보겠습니다.

    여러분이 컴퓨터의 전원을 켜면, 제일 먼저 컴퓨터는 마더보드의 특별한 플래시 메모리로부터 BIOS 이미지를 로드합니다.
    BIOS 이미지는 자가 점검 및 하드웨어 초기화 작업을 처리한 후에 부팅 가능한 디스크가 있는지 탐색합니다.
    부팅 가능한 디스크가 있다면, 제어 흐름은 해당 디스크의 부트로더 (bootloader) 에게 넘겨집니다.
    이 부트로더는 디스크의 가장 앞 주소 영역에 저장되는 512 바이트 크기의 실행 파일입니다.
    대부분의 부트로더들의 경우 로직을 저장하는 데에 512 바이트보다 더 큰 용량이 필요하기에,
    부트로더의 로직을 둘로 쪼개어 첫 단계 로직을 첫 512 바이트 안에 담고, 두 번째 단계 로직은 첫 단계 로직에 의해 로드된 이후 실행됩니다.

    부트로더는 커널 이미지가 디스크의 어느 주소에 저장되어있는지 알아낸 후 메모리에 커널 이미지를 로드해야 합니다.
    그 다음 CPU를 16비트 real mode에서 32비트 protected mode로 전환하고, 
    그 후에 다시 CPU를 64비트 long mode로 전환한 이후부터 64비트 레지스터 및 메인 메모리의 모든 주소를 사용할 수 있게 됩니다. 
    부트로더가 세 번째로 할 일은 BIOS로부터 메모리 매핑 정보 등의 필요한 정보를 알아내어 운영체제 커널에 전달하는 것입니다.

    부트로더를 작성하는 것은 상당히 성가신 작업인데, 그 이유는 어셈블리 코드도 작성해야 하고 
    “A 레지스터에 B 값을 저장하세요” 와 같이 원리를 단번에 이해하기 힘든 작업이 많이 수반되기 때문입니다. 

### Multiboot 표준

    운영체제마다 부트로더 구현 방법이 다르다면 한 운영체제에서 동작하는 부트로더가 다른 운영체제에서는 호환이 되지 않을 것입니다.
    이런 불편한 점을 막기 위해 Free Software Foundation에서 1995년에 Multiboot라는 부트로더 표준을 개발했습니다.
    이 표준은 부트로더와 운영체제 사이의 상호 작용 방식을 정의하였는데, 이 Multiboot 표준에 따르는 부트로더는 Multiboot 표준을 지원하는 어떤 운영체제에서도 동작합니다.
    이 표준을 구현한 대표적인 예로 리눅스 시스템에서 가장 인기 있는 부트로더인 GNU GRUB이 있습니다.

    운영체제 커널이 Multiboot를 지원하게 하려면 커널 파일의 맨 앞에 Multiboot 헤더를 삽입해주면 됩니다.
    이렇게 하면 GRUB에서 운영체제를 부팅하는 것이 매우 쉬워집니다.
    
    하지만 GRUB 및 Multiboot 표준도 몇 가지 문제점들을 안고 있습니다.

    오직 32비트 protected mode만을 지원합니다.
    64비트 long mode를 이용하고 싶다면 CPU 설정을 별도로 변경해주어야 합니다.
    Multiboot 표준 및 GRUB은 부트로더 구현의 단순화를 우선시하여 개발되었기에, 이에 호응하는 커널 측의 구현이 번거로워진다는 단점이 있습니다.
    예를 들어, GRUB이 Multiboot 헤더를 제대로 찾을 수 있으려면 커널 측에서 조정된 기본 페이지 크기 (adjusted default page size)를 링크하는 것이 강제됩니다.
    또한, 부트로더가 커널로 전달하는 부팅 정보는 적절한 추상 레벨에서 표준화된 형태로 전달되는 대신 하드웨어 아키텍처마다 상이한 형태로 제공됩니다.
    GRUB 및 Multiboot 표준에 대한 문서화 작업이 덜 되어 있습니다.
    GRUB이 호스트 시스템에 설치되어 있어야만 커널 파일로부터 부팅 가능한 디스크 이미지를 만들 수 있습니다.
    이 때문에 Windows 및 Mac에서는 부트로더를 개발하는 것이 Linux보다 어렵습니다.

## 출처 :

    https://os.phil-opp.com/ko/minimal-rust-kernel/
