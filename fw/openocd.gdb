source gdb-svd.py
svd stm32f303.svd
target extended-remote : 3333
set print asm-demangle on
set backtrace limit 32
break DefaultHandler
break HardFault
break rust_begin_unwind
break main
monitor arm semihosting enable
load
stepi

