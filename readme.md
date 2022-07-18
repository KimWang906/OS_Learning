# 최소한의 기능을 담은 Kernel

우리의 호스트 시스템 triple을 위해 컴파일하는 경우,
Rust 컴파일러와 링커는 Linux나 Windows와 같은 운영체제가 있다고 가정하고 또한 운영체제가
C 런타임 시스템을 사용할 것이라고 가정하기 때문에 링커 오류 메세지가 출력된 것입니다
이런 링커 오류를 피하려면 운영체제가 없는 시스템 환경에서 코드가 구동하는 것을 목표로 컴파일해야 합니다
운영체제가 없는 bare metal 시스템 환경의 한 예시로 thumbv7em-none-eabihf target triple이 있습니다
(이는 임베디드 ARM 시스템을 가리킵니다). Target triple의 none은 시스템에 운영체제가 동작하지 않음을 의미하며,
이 target triple의 나머지 부분의 의미는 아직 모르셔도 괜찮습니다
이 시스템 환경에서 구동 가능하도록 컴파일하려면 rustup에서 해당 시스템 환경을 추가해야 합니다

    rustup target add thumbv7em-none-eabihf

위 명령어를 실행하면 해당 시스템을 위한 Rust 표준 라이브러리 및 코어 라이브러리를 설치합니다
이제 해당 target triple을 목표로 하는 freestanding 실행파일을 만들 수 있습니다

    cargo build --target thumbv7em-none-eabihf

--target 인자를 통해 우리가 해당 bare metal 시스템을 목표로 크로스 컴파일할 것이라는 것을 cargo에게 알려줍니다.
목표 시스템 환경에 운영체제가 없는 것을 링커도 알기 때문에
C 런타임을 링크하려고 시도하지 않으며 이제는 링커 에러 없이 빌드가 성공할 것입니다.