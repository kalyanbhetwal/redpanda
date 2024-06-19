target remote :3333
load
monitor arm semihosting enable
b _start
break main