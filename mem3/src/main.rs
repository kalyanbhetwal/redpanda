#![allow(unsafe_code,unused, non_upper_case_globals,non_snake_case, non_camel_case_types )]
#![no_main]
#![no_std]
use core::mem;
use core::ptr;
use crate::ptr::addr_of;
use checkpoint::save_variables;
use cortex_m::asm::nop;
use panic_halt as _;
use cortex_m::peripheral::SCB;
use cortex_m_rt::entry;
use cortex_m::interrupt;
use cortex_m_semihosting::hprintln;
use stm32f3xx_hal_v2::{pac::Peripherals, pac::Interrupt};
use volatile::Volatile;
use cortex_m::peripheral::NVIC;

mod checkpoint;
use checkpoint::{checkpoint, restore, delete_pg, delete_all_pg, transcation_log, execution_mode, counter,start_atomic, end_atomic, initialization};
use instrument::my_proc_macro;

#[link_section = ".fram_section"]
static mut x:u16 = 8;
#[link_section = ".fram_section"]
static mut y:u16 = 3;
#[link_section = ".fram_section"]
static mut z:u16 = 2;
#[link_section = ".fram_section"]
static mut t:u16 = 0xFF; //change to assign a random number

// #[link_section = ".fram_section"]
// static mut rnd_array:[u16;5] = [10,12,14,15,2];

// fn test_checkpoint(){
//     unsafe {
//         asm!("mov r0, #10
//               mov r1, #20
//               mov r2, #30
//               mov r3, #40
//               mov r4, #50
//               mov r5, #20
//               mov r6, #30
//               mov r7, #40
//               mov r8, #50
//         "); 
//         }
//     checkpoint(false);
//     unsafe {
//         asm!("add r0, r1"); 
//         }
// }

#[my_proc_macro]
fn update(){
    let mut ya:u16 = 12;
    start_atomic();
    //unsafe{save_variables(addr_of!(x), 4);}
    unsafe{x = 5;}
    ya = ya + 2;
    end_atomic();
}

#[no_mangle]
pub extern "C" fn main() -> ! {
    //delete_pg(0x0803_0000 as u32);  //0x0807_F800
   
    initialization();
    hprintln!("reseting the counter ");
    //unsafe{ptr::write_volatile(  0x6000_9000 as *mut u32, 0);} //this line is for deleting the start_address
   //restore();
    // unsafe{rnd_array[4] = 1;}
    //restore();
    update();
    hprintln!("reseting the counter ");
    //unsafe{ptr::write(counter as *mut u8 ,0);}


    checkpoint(false);
    
    // if unsafe{execution_mode}{
    //     checkpoint(false);
    // }
    
    // exit QEMU
    // NOTE do not run this on hardware; it can corrupt OpenOCD state
    //debug::exit(debug::EXIT_SUCCESS);

    //let mut ans = [0;60];
    //let mut ans;
    unsafe {
        ptr::write_volatile((0x6000_0010) as *mut u16, 0xabcd as u16); 

        // ans =  ptr::read_volatile((0x6000_0000) as *mut u16); 

        // hprintln!("Value at index {:?}", ans).unwrap();
    }


    // let mut ans;
    // unsafe{
    //     for i in (0..=1000).step_by(4){
    //        // ptr::write_volatile((0x6000_0000 + i) as *mut u8, 0xDD as u8); 

    //         ans = unsafe { ptr::read_volatile((0x6000_0000 +i) as *mut u32) };
    //         hprintln!("Value at index {}: {}", i, ans).unwrap();
    //     }  
    //    // hprintln!("Value at index {:?}", ans).unwrap();

    //   }

    hprintln!("reseting the counter at the end ");

   loop {}
}