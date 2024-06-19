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
use checkpoint::{checkpoint, restore, delete_pg, delete_all_pg, transcation_log, execution_mode,start_atomic, end_atomic};
use instrument::my_proc_macro;

#[link_section = ".fram_section"]
static mut x:u8 = 1;
#[link_section = ".fram_section"]
static mut y:u8 = 3;
#[link_section = ".fram_section"]
static mut z:u8 = 2;
#[link_section = ".fram_section"]
static mut t:u8 = 5; //change to assign a random number

#[link_section = ".fram_section"]
static mut rnd_array:[u16;5] = [10,12,14,15,2];

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

fn initialization(){
    unsafe{execution_mode = false;}
    let dp  = Peripherals::take().unwrap();
    
     //enable HSI
    dp.RCC.cr.write(|w| w.hsion().set_bit());
    while dp.RCC.cr.read().hsirdy().bit_is_clear() {}
 
     //configure PLL
     // Step 1: Disable the PLL by setting PLLON to 0
     dp.RCC.cr.modify(|_r, w| w.pllon().clear_bit());
 
     // Step 2: Wait until PLLRDY is cleared
     while dp.RCC.cr.read().pllrdy().bit_is_set() {}
 
     // Step 3: Change the desired parameter
     // For example, modify PLL multiplier (PLLMUL)
 
     dp.RCC.cfgr.modify(|_, w| w.pllsrc().hsi_div_prediv());
 
     // Set PLL Prediv to /1
     dp.RCC.cfgr2.modify(|_, w| w.prediv().div1());
 
     // Set PLL MUL to x9
     dp.RCC.cfgr.modify(|_, w| w.pllmul().mul9());
 
     // Step 4: Enable the PLL again by setting PLLON to 1
    // dp.RCC.cr.modify(|_r, w| w.pllon().set_bit());
 
     dp.RCC.cr.modify(|_, w| w.pllon().on());
 
     while dp.RCC.cr.read().pllrdy().bit_is_clear(){}
 
        // Configure prescalar values for HCLK, PCLK1, and PCLK2
    dp.RCC.cfgr.modify(|_, w| {
         w.hpre().div1() // HCLK prescaler: no division
         .ppre1().div2() // PCLK1 prescaler: divide by 2
         .ppre2().div1() // PCLK2 prescaler: no division
     });
 
 
     // Enable FLASH Prefetch Buffer and set Flash Latency (required for high speed)
     // was crashing just because this was missing
     dp.FLASH.acr
         .modify(|_, w| w.prftbe().enabled().latency().ws1());
 
      // Select PLL as system clock source
      dp.RCC.cfgr.modify(|_, w| w.sw().pll());
 
      while dp.RCC.cfgr.read().sw().bits() != 0b10 {}
 
       // Wait for system clock to stabilize
       while dp.RCC.cfgr.read().sws().bits() != 0b10 {}
 
    //   dp.RCC.ahbenr.modify(|_, w| w.iopden().set_bit());
    //   dp.RCC.ahbenr.modify(|_, w| w.iopeen().set_bit());
    //   dp.RCC.ahbenr.modify(|_, w| w.iopfen().set_bit());
    //   dp.RCC.ahbenr.modify(|_, w| w.iopgen().set_bit());
    //   dp.RCC.ahbenr.modify(|_, w| w.iophen().set_bit());  
    //   dp.RCC.ahbenr.modify(|_, w| w.sramen().set_bit());  
    //   dp.RCC.ahbenr.modify(|_, w| w.flitfen().set_bit());  
    //   dp.RCC.ahbenr.modify(|_, w| w.fmcen().set_bit());  

      dp.RCC.ahbenr.write(|w| unsafe{w.bits(0xf10034)});
 
      dp.RCC.apb2enr.modify(|_, w| w.syscfgen().set_bit());
      dp.RCC.apb1enr.modify(|_, w| w.pwren().set_bit());
   
      dp.GPIOD.moder.write(|w| unsafe{w.bits(0xa0008a0a)});
      dp.GPIOD.ospeedr.write(|w| unsafe { w.bits(0xf000cf0f) });
      dp.GPIOD.afrl.write(|w| unsafe { w.bits(0xc0cc00cc) });
      dp.GPIOD.afrh.write(|w| unsafe { w.bits(0xcc000000) });

   
      dp.GPIOE.moder.write(|w| unsafe{w.bits(0x2a8000)});
      dp.GPIOE.ospeedr.write(|w| unsafe { w.bits(0xff000ff0) });
      dp.GPIOE.afrl.write(|w| unsafe { w.bits(0xc0000000) });
      dp.GPIOE.afrh.write(|w| unsafe { w.bits(0xccc) });
   
      dp.GPIOF.moder.write(|w| unsafe{w.bits(0xaa000aa0)});
      dp.GPIOF.ospeedr.write(|w| unsafe { w.bits(0x3fc000) });
      dp.GPIOF.afrl.write(|w| unsafe { w.bits(0xcccc00) });
      dp.GPIOF.afrh.write(|w| unsafe { w.bits(0xccc) });
   
      dp.GPIOG.moder.write(|w| unsafe{w.bits(0x2aa)});
      dp.GPIOG.ospeedr.write(|w| unsafe { w.bits(0x3ff) });
      dp.GPIOG.afrl.write(|w| unsafe { w.bits(0xccccc) });
   
      dp.GPIOH.moder.write(|w| unsafe{w.bits(0xa)});
      dp.GPIOH.ospeedr.write(|w| unsafe { w.bits(0xf) });
      dp.GPIOH.afrl.write(|w| unsafe { w.bits(0xcc) });

   
     // Configure FMC for SRAM memory(in our case F-RAM)
       unsafe{
           dp.FMC.bcr1.modify(|_, w| {
           w.mbken().set_bit(); // Enable FRAM bank 1
           w.mtyp().bits(0b00); // FRAM memory type
           w.mwid().bits(0b00); // 8-bit width
           w.bursten().clear_bit(); //disable brust access mode
           w.wren().clear_bit(); // wrap disable
           w.muxen().clear_bit(); // Non-multiplexed
           w.extmod().clear_bit(); // extended mode
           w.asyncwait().clear_bit(); //disable async wait
           w
        });
   
        /*
           Timing.AddressSetupTime = 1;
           Timing.AddressHoldTime = 1;
           Timing.DataSetupTime = 5;
           Timing.BusTurnAroundDuration = 0;
           Timing.CLKDivision = 0;
           Timing.DataLatency = 0;
           Timing.AccessMode = FMC_ACCESS_MODE_A;
      */
        dp.FMC.btr1.modify(|_,w|  {
          // Set address setup time to 1 cycle
           w.addset().bits(0x1);
           // Set data setup time to 5 cycle
           w.datast().bits(0x5);
           // address hold time
           w.addhld().bits(0x1);
           // bus turn around
           w.busturn().bits(0x0);
           // clock division
           w.clkdiv().bits(0x0);
           //data latency
           w.datlat().bits(0x0);
           //access mode
           w.accmod().bits(0x0);
   
           w
       });
   }
   
unsafe{
    //let dp = Peripherals::steal(); //take().unwrap();

    // Enable the clock for GPIOA and SYSCFG
    dp.RCC.ahbenr.modify(|_, w| w.iopaen().set_bit());
   // dp.RCC.apb2enr.modify(|_, w| w.syscfgen().set_bit()); //already done above

    // Configure PA0 as input
    dp.GPIOA.moder.modify(|_, w| w.moder0().input());
    dp.GPIOA.pupdr.modify(|_, w| w.pupdr0().pull_up());

    dp.SYSCFG.exticr1.modify(|_, w| w.exti0().pa0());

    // Configure EXTI0 for falling edge trigger and enable it
    dp.EXTI.imr1.modify(|_, w| w.mr0().set_bit());
    dp.EXTI.ftsr1.modify(|_, w| w.tr0().set_bit());
    }
    
    // Enable EXTI0 interrupt in the NVIC
   // unsafe { NVIC::unmask(Interrupt::EXTI0) };
   unsafe{execution_mode = true;}

}

#[my_proc_macro]
fn update(){
    let mut ya:u8 = 2;
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
    unsafe{rnd_array[4] = 1;}
    update();
    if unsafe{execution_mode}{
        checkpoint(false);
    }
  
    // exit QEMU
    // NOTE do not run this on hardware; it can corrupt OpenOCD state
    //debug::exit(debug::EXIT_SUCCESS);

   loop {}
}
