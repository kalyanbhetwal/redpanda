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
use checkpoint::{checkpoint, restore, delete_pg, delete_all_pg, transcation_log, execution_mode, counter,start_atomic, end_atomic};
use instrument::my_proc_macro;

#[link_section = ".fram_section"]
static mut x:u16 = 1;
#[link_section = ".fram_section"]
static mut y:u16 = 3;
#[link_section = ".fram_section"]
static mut z:u16 = 2;
#[link_section = ".fram_section"]
static mut t:u16 = 5; //change to assign a random number

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

    dp.RCC.ahbenr.modify(|_, w| w.iopden().set_bit());
    dp.RCC.ahbenr.modify(|_, w| w.iopeen().set_bit());
    dp.RCC.ahbenr.modify(|_, w| w.iopfen().set_bit());
    dp.RCC.ahbenr.modify(|_, w| w.iopgen().set_bit());
    dp.RCC.ahbenr.modify(|_, w| w.iophen().set_bit());  
    dp.RCC.ahbenr.modify(|_, w| w.sramen().set_bit());  
    dp.RCC.ahbenr.modify(|_, w| w.flitfen().set_bit());  
    dp.RCC.ahbenr.modify(|_, w| w.fmcen().set_bit());  


    dp.RCC.apb2enr.modify(|_, w| w.syscfgen().set_bit());
    dp.RCC.apb1enr.modify(|_, w| w.pwren().set_bit());

  let mut gpiod = dp.GPIOD;
  let mut gpioe = dp.GPIOE;
  let mut gpiof = dp.GPIOF;
  let mut gpiog = dp.GPIOG;
  let mut gpioh = dp.GPIOH;

    gpioh.moder.modify(|_, w| {w.moder0().alternate()});
    gpioh.afrl.modify(|_, w| {  w.afrl0().af12()});
    gpioh.ospeedr.modify(|_, w| w.ospeedr0().very_high_speed());


    gpioh.moder.modify(|_, w| {w.moder1().alternate()});
    gpioh.afrl.modify(|_, w| {  w.afrl1().af12()});
    gpioh.ospeedr.modify(|_, w| w.ospeedr1().very_high_speed());


    gpiof.moder.modify(|_, w| {w.moder2().alternate()});
    gpiof.afrl.modify(|_, w| {  w.afrl2().af12()});
    gpiof.ospeedr.modify(|_, w| w.ospeedr2().very_high_speed());


    gpiof.moder.modify(|_, w| {w.moder3().alternate()});
    gpiof.afrl.modify(|_, w| {  w.afrl3().af12()});
    gpiof.ospeedr.modify(|_, w| w.ospeedr3().very_high_speed());

    gpiof.moder.modify(|_, w| {w.moder4().alternate()});
    gpiof.afrl.modify(|_, w| {  w.afrl4().af12()});
    gpiof.ospeedr.modify(|_, w| w.ospeedr4().very_high_speed());


    gpiof.moder.modify(|_, w| {w.moder5().alternate()});
    gpiof.afrl.modify(|_, w| {  w.afrl5().af12()});
    gpiof.ospeedr.modify(|_, w| w.ospeedr5().very_high_speed());

        
        // pf.pf12.into_af12(&mut pf.moder, &mut pf.afrh); //FMC_A6
        // pf.pf13.into_af12(&mut pf.moder, &mut pf.afrh); //FMC_A7
        // pf.pf14.into_af12(&mut pf.moder, &mut pf.afrh); //FMC_A8
        // pf.pf15.into_af12(&mut pf.moder, &mut pf.afrh); //FMC_A9

    gpiof.moder.modify(|_, w| {w.moder12().alternate()});
    gpiof.afrh.modify(|_, w| {  w.afrh12().af12()});
    gpiof.ospeedr.modify(|_, w| w.ospeedr12().very_high_speed());


    gpiof.moder.modify(|_, w| {w.moder13().alternate()});
    gpiof.afrh.modify(|_, w| {  w.afrh13().af12()});
    gpiof.ospeedr.modify(|_, w| w.ospeedr13().very_high_speed());


    gpiof.moder.modify(|_, w| {w.moder14().alternate()});
    gpiof.afrh.modify(|_, w| {  w.afrh14().af12()});
    gpiof.ospeedr.modify(|_, w| w.ospeedr14().very_high_speed());


    gpiof.moder.modify(|_, w| {w.moder15().alternate()});
    gpiof.afrh.modify(|_, w| {  w.afrh15().af12()});
    gpiof.ospeedr.modify(|_, w| w.ospeedr15().very_high_speed());

  // pg.pg0.into_af12(&mut pg.moder, &mut pg.afrl); //FMC_A10
    // pg.pg1.into_af12(&mut pg.moder, &mut pg.afrl); //FMC_A11
    // pg.pg2.into_af12(&mut pg.moder, &mut pg.afrl); //FMC_A12
    // pg.pg3.into_af12(&mut pg.moder, &mut pg.afrl); //FMC_A13
    // pg.pg4.into_af12(&mut pg.moder, &mut pg.afrl); //FMC_A14

    gpiog.moder.modify(|_, w| {w.moder0().alternate()});
    gpiog.afrl.modify(|_, w| {  w.afrl0().af12()});
    gpiog.ospeedr.modify(|_, w| w.ospeedr0().very_high_speed());

    
    gpiog.moder.modify(|_, w| {w.moder1().alternate()});
    gpiog.afrl.modify(|_, w| {  w.afrl1().af12()});
    gpiog.ospeedr.modify(|_, w| w.ospeedr1().very_high_speed());

    
    gpiog.moder.modify(|_, w| {w.moder2().alternate()});
    gpiog.afrl.modify(|_, w| {  w.afrl2().af12()});
    gpiog.ospeedr.modify(|_, w| w.ospeedr2().very_high_speed());

    
    gpiog.moder.modify(|_, w| {w.moder3().alternate()});
    gpiog.afrl.modify(|_, w| {  w.afrl3().af12()});
    gpiog.ospeedr.modify(|_, w| w.ospeedr3().very_high_speed());

    
    gpiog.moder.modify(|_, w| {w.moder4().alternate()});
    gpiog.afrl.modify(|_, w| {  w.afrl4().af12()});
    gpiog.ospeedr.modify(|_, w| w.ospeedr4().very_high_speed());

    gpiod.moder.modify(|_, w| {w.moder14().alternate()});
    gpiod.afrh.modify(|_, w| {  w.afrh14().af12()});
    gpiod.ospeedr.modify(|_, w| w.ospeedr14().very_high_speed());

    gpiod.moder.modify(|_, w| {w.moder15().alternate()});
    gpiod.afrh.modify(|_, w| {  w.afrh15().af12()});
    gpiod.ospeedr.modify(|_, w| w.ospeedr15().very_high_speed());

    gpiod.moder.modify(|_, w| {w.moder0().alternate()});
    gpiod.afrl.modify(|_, w| {  w.afrl0().af12()});
    gpiod.ospeedr.modify(|_, w| w.ospeedr0().very_high_speed());


    gpiod.moder.modify(|_, w| {w.moder1().alternate()});
    gpiod.afrl.modify(|_, w| {  w.afrl1().af12()});
    gpiod.ospeedr.modify(|_, w| w.ospeedr1().very_high_speed());

    gpioe.moder.modify(|_, w| {w.moder7().alternate()});
    gpioe.afrl.modify(|_, w| {  w.afrl7().af12()});
    gpioe.ospeedr.modify(|_, w| w.ospeedr7().very_high_speed());

    gpioe.moder.modify(|_, w| {w.moder8().alternate()});
    gpioe.afrh.modify(|_, w| {  w.afrh8().af12()});
    gpioe.ospeedr.modify(|_, w| w.ospeedr8().very_high_speed());

    gpioe.moder.modify(|_, w| {w.moder9().alternate()});
    gpioe.afrh.modify(|_, w| {  w.afrh9().af12()});
    gpioe.ospeedr.modify(|_, w| w.ospeedr9().very_high_speed());


    gpioe.moder.modify(|_, w| {w.moder10().alternate()});
    gpioe.afrh.modify(|_, w| {  w.afrh10().af12()});
    gpioe.ospeedr.modify(|_, w| w.ospeedr10().very_high_speed());


    gpiod.moder.modify(|_, w| {w.moder7().alternate()});
    gpiod.afrl.modify(|_, w| {  w.afrl7().af12()});
    gpiod.ospeedr.modify(|_, w| w.ospeedr7().very_high_speed());


    gpiod.moder.modify(|_, w| {w.moder4().alternate()});
    gpiod.afrl.modify(|_, w| {  w.afrl4().af12()});
    gpiod.ospeedr.modify(|_, w| w.ospeedr4().very_high_speed());


    gpiod.moder.modify(|_, w| {w.moder5().alternate()});
    gpiod.afrl.modify(|_, w| {  w.afrl5().af12()});
    gpiod.ospeedr.modify(|_, w| w.ospeedr5().very_high_speed());


   
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
    hprintln!("resting the counter ");
    //unsafe{ptr::write(counter as *mut u8 ,0);}
    
    // if unsafe{execution_mode}{
    //     checkpoint(false);
    // }
    
    // exit QEMU
    // NOTE do not run this on hardware; it can corrupt OpenOCD state
    //debug::exit(debug::EXIT_SUCCESS);

   loop {}
}
