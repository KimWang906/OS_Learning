# Chapter 4 : Testing - 1

    요약 : 4일차에는 Qemu를 백그라운드에서 실행하게끔 만들었고, 직렬포트를 이용한 기본 드라이버를 제작하였습니다.

## 실행 결과화면

<img src="/record_image/day_4_console_ok.png"><br>
<img src="/record_image/day_4_console_fail.png">

## Source code

<img src="/record_image/day_4_source_code.png"><br>

    4일차에서 배운 가장 중요하다고 생각하는 코드입니다.
    serial 모듈에 있는 serial_println을 이용하여 패닉이 생길 경우 panic() 함수를 호출해
    Console에 메세지를 출력할 수 있게 합니다. 이로 인해 테스트 중 Qemu 화면을 보지 않아도 결과를 알 수 있고
    향후 테스트케이스를 출력할 때도 사용할 수 있습니다.
