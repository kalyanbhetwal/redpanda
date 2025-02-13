
#![allow(unsafe_code, non_upper_case_globals)]
pub mod my_flash;
use my_flash::{unlock, wait_ready, clear_error_flags, erase_page, write_to_flash};

use core::mem;
use core::ptr;
use cortex_m::asm::{nop, self};
use cortex_m_semihosting::hprintln;
use panic_halt as _;

use::core::arch::asm;
use stm32f3xx_hal_v2::{pac::Peripherals, pac::FLASH};
use volatile::Volatile;

pub static mut transcation_log: u32 = 0x60004000; 
pub static mut data_loc: u32 = 0x60005000; 
pub static mut execution_mode: bool = true;  //1. true is jit 2.false is static 
pub static mut counter: *mut u16= 0x60003000 as *mut u16;


pub fn initialization(){
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
    dp.RCC.cfgr.modify(|_, w| w.pllmul().mul2());  //changed from x9 to x2

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

   dp.RCC.ahbenr.modify(|_, w| w.iopcen().set_bit());

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

 let  gpiod = dp.GPIOD;
 let  gpioe = dp.GPIOE;
 let  gpiof = dp.GPIOF;
 let  gpiog = dp.GPIOG;
 let  gpioh = dp.GPIOH;

 dp.GPIOC.moder.write(|w| unsafe { w.moder10().bits(0b01) });
 dp.GPIOC.moder.write(|w| unsafe { w.moder11().bits(0b01) });
 dp.GPIOC.odr.write(|w| w.odr10().set_bit());
 dp.GPIOA.odr.write(|w| w.odr11().set_bit());

//  dp.GPIOC.moder.modify(|_, w| w.moder11().bits(0b01)); // Set MODER11[1:0] = 01
//  dp.GPIOC.bsrr.write(|w| w.bs11().set_bit());


//  dp.GPIOC.moder.modify(|_, w| w.moder10().bits(0b01)); // Set MODER10[1:0] = 01
//  dp.GPIOC.bsrr.write(|w| w.bs10().set_bit());

 //    PH0   ------> FMC_A0
   gpioh.moder.modify(|_, w| {w.moder0().alternate()});
   gpioh.afrl.modify(|_, w| {  w.afrl0().af12()});
   gpioh.ospeedr.modify(|_, w| w.ospeedr0().very_high_speed());

// PH1   ------> FMC_A1
   gpioh.moder.modify(|_, w| {w.moder1().alternate()});
   gpioh.afrl.modify(|_, w| {  w.afrl1().af12()});
   gpioh.ospeedr.modify(|_, w| w.ospeedr1().very_high_speed());

//  PF2   ------> FMC_A2
   gpiof.moder.modify(|_, w| {w.moder2().alternate()});
   gpiof.afrl.modify(|_, w| {  w.afrl2().af12()});
   gpiof.ospeedr.modify(|_, w| w.ospeedr2().very_high_speed());

//   PF3   ------> FMC_A3
   gpiof.moder.modify(|_, w| {w.moder3().alternate()});
   gpiof.afrl.modify(|_, w| {  w.afrl3().af12()});
   gpiof.ospeedr.modify(|_, w| w.ospeedr3().very_high_speed());

   //   PF4   ------> FMC_A4
   gpiof.moder.modify(|_, w| {w.moder4().alternate()});
   gpiof.afrl.modify(|_, w| {  w.afrl4().af12()});
   gpiof.ospeedr.modify(|_, w| w.ospeedr4().very_high_speed());

// PF5   ------> FMC_A5
   gpiof.moder.modify(|_, w| {w.moder5().alternate()});
   gpiof.afrl.modify(|_, w| {  w.afrl5().af12()});
   gpiof.ospeedr.modify(|_, w| w.ospeedr5().very_high_speed());


   //    PF12   ------> FMC_A6
   gpiof.moder.modify(|_, w| {w.moder12().alternate()});
   gpiof.afrh.modify(|_, w| {  w.afrh12().af12()});
   gpiof.ospeedr.modify(|_, w| w.ospeedr12().very_high_speed());

//   PF13   ------> FMC_A7
   gpiof.moder.modify(|_, w| {w.moder13().alternate()});
   gpiof.afrh.modify(|_, w| {  w.afrh13().af12()});
   gpiof.ospeedr.modify(|_, w| w.ospeedr13().very_high_speed());

//   PF14   ------> FMC_A8
   gpiof.moder.modify(|_, w| {w.moder14().alternate()});
   gpiof.afrh.modify(|_, w| {  w.afrh14().af12()});
   gpiof.ospeedr.modify(|_, w| w.ospeedr14().very_high_speed());

   //PF15   ------> FMC_A9
   gpiof.moder.modify(|_, w| {w.moder15().alternate()});
   gpiof.afrh.modify(|_, w| {  w.afrh15().af12()});
   gpiof.ospeedr.modify(|_, w| w.ospeedr15().very_high_speed());

   // PG0   ------> FMC_A10
   gpiog.moder.modify(|_, w| {w.moder0().alternate()});
   gpiog.afrl.modify(|_, w| {  w.afrl0().af12()});
   gpiog.ospeedr.modify(|_, w| w.ospeedr0().very_high_speed());

   //  PG1   ------> FMC_A11
   gpiog.moder.modify(|_, w| {w.moder1().alternate()});
   gpiog.afrl.modify(|_, w| {  w.afrl1().af12()});
   gpiog.ospeedr.modify(|_, w| w.ospeedr1().very_high_speed());

   //  PG2   ------> FMC_A12
   gpiog.moder.modify(|_, w| {w.moder2().alternate()});
   gpiog.afrl.modify(|_, w| {  w.afrl2().af12()});
   gpiog.ospeedr.modify(|_, w| w.ospeedr2().very_high_speed());

   //    PG3   ------> FMC_A13
   gpiog.moder.modify(|_, w| {w.moder3().alternate()});
   gpiog.afrl.modify(|_, w| {  w.afrl3().af12()});
   gpiog.ospeedr.modify(|_, w| w.ospeedr3().very_high_speed());

   //   PG4   ------> FMC_A14
   gpiog.moder.modify(|_, w| {w.moder4().alternate()});
   gpiog.afrl.modify(|_, w| {  w.afrl4().af12()});
   gpiog.ospeedr.modify(|_, w| w.ospeedr4().very_high_speed());

   
    //PG5   ------> FMC_A15
    gpiog.moder.modify(|_, w| {w.moder5().alternate()});
    gpiog.afrl.modify(|_, w| {  w.afrl5().af12()});
    gpiog.ospeedr.modify(|_, w| w.ospeedr5().very_high_speed());


    //  PD14   ------> FMC_D0
   gpiod.moder.modify(|_, w| {w.moder14().alternate()});
   gpiod.afrh.modify(|_, w| {  w.afrh14().af12()});
   gpiod.ospeedr.modify(|_, w| w.ospeedr14().very_high_speed());

   //  PD15   ------> FMC_D1
   gpiod.moder.modify(|_, w| {w.moder15().alternate()});
   gpiod.afrh.modify(|_, w| {  w.afrh15().af12()});
   gpiod.ospeedr.modify(|_, w| w.ospeedr15().very_high_speed());

   // PD0   ------> FMC_D2
   gpiod.moder.modify(|_, w| {w.moder0().alternate()});
   gpiod.afrl.modify(|_, w| {  w.afrl0().af12()});
   gpiod.ospeedr.modify(|_, w| w.ospeedr0().very_high_speed());

   // PD1   ------> FMC_D3
   gpiod.moder.modify(|_, w| {w.moder1().alternate()});
   gpiod.afrl.modify(|_, w| {  w.afrl1().af12()});
   gpiod.ospeedr.modify(|_, w| w.ospeedr1().very_high_speed());

   //PE7   ------> FMC_D4
   gpioe.moder.modify(|_, w| {w.moder7().alternate()});
   gpioe.afrl.modify(|_, w| {  w.afrl7().af12()});
   gpioe.ospeedr.modify(|_, w| w.ospeedr7().very_high_speed());

   //PE8   ------> FMC_D5
   gpioe.moder.modify(|_, w| {w.moder8().alternate()});
   gpioe.afrh.modify(|_, w| {  w.afrh8().af12()});
   gpioe.ospeedr.modify(|_, w| w.ospeedr8().very_high_speed());

   // PE9   ------> FMC_D6
   gpioe.moder.modify(|_, w| {w.moder9().alternate()});
   gpioe.afrh.modify(|_, w| {  w.afrh9().af12()});
   gpioe.ospeedr.modify(|_, w| w.ospeedr9().very_high_speed());

   //PE10   ------> FMC_D7
   gpioe.moder.modify(|_, w| {w.moder10().alternate()});
   gpioe.afrh.modify(|_, w| {  w.afrh10().af12()});
   gpioe.ospeedr.modify(|_, w| w.ospeedr10().very_high_speed());

   //PE11   ------> FMC_D8
   gpioe.moder.modify(|_, w| {w.moder11().alternate()});
   gpioe.afrh.modify(|_, w| {  w.afrh11().af12()});
   gpioe.ospeedr.modify(|_, w| w.ospeedr11().very_high_speed());


   //PE12   ------> FMC_D9
   gpioe.moder.modify(|_, w| {w.moder12().alternate()});
   gpioe.afrh.modify(|_, w| {  w.afrh12().af12()});
   gpioe.ospeedr.modify(|_, w| w.ospeedr12().very_high_speed());

   //PE13   ------> FMC_D10
   gpioe.moder.modify(|_, w| {w.moder13().alternate()});
   gpioe.afrh.modify(|_, w| {  w.afrh13().af12()});
   gpioe.ospeedr.modify(|_, w| w.ospeedr13().very_high_speed());

   //PE14   ------> FMC_D11
   gpioe.moder.modify(|_, w| {w.moder14().alternate()});
   gpioe.afrh.modify(|_, w| {  w.afrh14().af12()});
   gpioe.ospeedr.modify(|_, w| w.ospeedr14().very_high_speed());

   //PE15   ------> FMC_D12
   gpioe.moder.modify(|_, w| {w.moder15().alternate()});
   gpioe.afrh.modify(|_, w| {  w.afrh15().af12()});
   gpioe.ospeedr.modify(|_, w| w.ospeedr15().very_high_speed());

   //PD8   ------> FMC_D13
   gpiod.moder.modify(|_, w| {w.moder8().alternate()});
   gpiod.afrh.modify(|_, w| {  w.afrh8().af12()});
   gpiod.ospeedr.modify(|_, w| w.ospeedr8().very_high_speed());

   //PD9   ------> FMC_D14
   gpiod.moder.modify(|_, w| {w.moder9().alternate()});
   gpiod.afrh.modify(|_, w| {  w.afrh9().af12()});
   gpiod.ospeedr.modify(|_, w| w.ospeedr9().very_high_speed());

   //PD10   ------> FMC_D15
   gpiod.moder.modify(|_, w| {w.moder10().alternate()});
   gpiod.afrh.modify(|_, w| {  w.afrh10().af12()});
   gpiod.ospeedr.modify(|_, w| w.ospeedr10().very_high_speed());


   // PD4   ------> FMC_NOE
   // PD5   ------> FMC_NWE
   // PD7   ------> FMC_NE1

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
          w.mwid().bits(0b01); // 16-bit width
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
          w.clkdiv().bits(0x4);
          //data latency
          w.datlat().bits(0x0);
          //access mode
          w.accmod().bits(0x0);
  
          w
      });
  }
}

pub fn save_variables<T>(mem_loc: *const T, size: usize) {
    unsafe{
        hprintln!("mem loc {:?}", mem_loc);
        ptr::write(transcation_log as *mut u32 , mem_loc as u32);
        transcation_log += 4;
        ptr::write(transcation_log as *mut u32 , size as u32);
        transcation_log += 4;
        ptr::write( data_loc as *mut u16 , (mem_loc as * const u16) as u16);
        data_loc+= 2;
        *counter = *counter + 1;
    }

}

pub fn save_variables1<T>(mem_loc: *const T, size: usize) {
    unsafe{
        let mem_loc_u8 = mem_loc as *const u8;
        for i in 0..4 {
            let byte = (mem_loc_u8 as u32 >> (i * 8)) as u8; // Extract the byte at position i
            hprintln!("bytes {:0x}", byte);
            ptr::write((transcation_log+2 *i as u32) as *mut u8 , byte);
        }
        transcation_log += 2*4;

        ptr::write(transcation_log as *mut u8 , size as u8);

        transcation_log += 2*1; //adding 2 because of issues in fram where only even address are being written

        for i in 0..size{
            let byte = *mem_loc_u8.add(i); 
            hprintln!("the logged byte {}", byte);
            ptr::write( (transcation_log+2*i as u32) as *mut u8 , byte);   
        }
        transcation_log =  transcation_log + 2*size as u32;
        *counter +=1;

        // let mut a = ptr::read(counter as *const u8);
        // hprintln!("the read value before logging {}",a );
        // a = a + 1;
        // ptr::write(counter as *mut u8, a);
        // let b = ptr::read(counter as *const u8 );
        // hprintln!("After logging counter is {}", b);
    }
    hprintln!("Address: {:p}, Size: {} bytes", mem_loc, size);
}

pub fn start_atomic(){
    //checkpoint(true);
    //undo or redo updates
    //memcopy some variables
           //unsafe{ptr::write(transcation_log as *mut u8, 1);} //still debating this
    // unsafe {
    //     let mut step = 0;
    //     for (mem_loc, size) in mem_locs.iter().zip(sizes.iter()) {
            
    //         for i in 0..4 {
    //             let byte = (*mem_loc >> (i * 8)) as u8; // Extract the byte at position i
    //             ptr::write((transcation_log+i as u32) as *mut u8 , byte);
    //         }

    //         transcation_log += 4;

    //         ptr::write((transcation_log) as *mut u8 , *size as u8);

    //         let byte_ptr = *mem_loc as *const u8;    
    //         for i in 0..*size{
    //             ptr::write( (transcation_log+i as u32) as *mut u8 , *byte_ptr.add(i));   
    //         }
    //         step = step + *size;
    //         transcation_log =  transcation_log + *size as u32;
    //         }
    //         ptr::write( transcation_log as *mut u8 ,0xFB);   // mark end of the transcation
    //         transcation_log = 0x60004000;

    //         }  
    unsafe{execution_mode = false;}
}


pub fn end_atomic(){
    unsafe {transcation_log = 0x60004000;}
    unsafe {execution_mode = true;}

}

#[no_mangle]
pub fn checkpoint(c_type:bool){

    unsafe {
        asm!(
            "add sp, #256"
        );
    }
    unsafe {
        asm!(
            "pop {{r7}}"
        );
    }
    unsafe {
        asm!(
            "push {{r7}}"
        );
    }
    unsafe {
        asm!(
            "sub sp, #256"
        );
    }

    let r0_value: u32;
    let r1_value: u32;
    let r2_value: u32;
    let r3_value: u32;
    let r4_value: u32;
    let r5_value: u32;
    let r6_value: u32;
    let r7_value: u32;
    let r8_value: u32;
    let r9_value: u32;
    let r10_value: u32;
    let r11_value: u32;
    let r12_value: u32;
    let r13_sp: u32;
    let r14_lr: u32;
    let r15_pc: u32;

    unsafe {
        asm!(
            "MOV {0}, r0",
            out(reg) r0_value
        );
    }

    unsafe {
        asm!(
            "MOV {0}, r1",
            out(reg) r1_value
        );
    }

    unsafe {
        asm!(
            "MOV {0}, r2",
            out(reg) r2_value
        );
    }
    unsafe {
        asm!(
            "MOV {0}, r3",
            out(reg) r3_value
        );
    }

    unsafe {
        asm!(
            "MOV {0}, r4",
            out(reg) r4_value
        );
    }

    unsafe {
        asm!(
            "MOV {0}, r5",
            out(reg) r5_value
        );
    }

    unsafe {
        asm!(
            "MOV {0}, r6",
            out(reg) r6_value
        );
    }

    unsafe {
        asm!(
            "MOV {0}, r7",
            out(reg) r7_value
        );
    }

    unsafe {
        asm!(
            "MOV {0}, r8",
            out(reg) r8_value
        );
    }

    unsafe {
        asm!(
            "MOV {0}, r9",
            out(reg) r9_value
        );
    }

    unsafe {
        asm!(
            "MOV {0}, r10",
            out(reg) r10_value
        );
    }
    unsafe {
        asm!(
            "MOV {0}, r11",
            out(reg) r11_value
        );
    }


    unsafe {
        asm!(
            "MOV {0}, r12",
            out(reg) r12_value
        );
    }
 
    unsafe {
        asm!(
            "MOV {0}, r14",
            out(reg) r14_lr
        );
    }
    unsafe {
        asm!(
            "MOV {0}, r15",
            out(reg) r15_pc
        );
    }

  //  r14_lr = lr;

    unsafe {
        asm!(
            "MOV r0, sp",
        );
    }
    // have to be extra careful for the sp value
    unsafe {
        asm!(
            "add r0, #264",
        );
    }
    unsafe {
        asm!(
            "MOV {0}, r0",
            out(reg) r13_sp
        );
    }
    unsafe{

        // let  dp = Peripherals::steal();
        // let mut flash= dp.FLASH;
        // unlock(&mut flash);
        // wait_ready(&flash);

   
        //let  start_address: u32 = 0x2000_fffc as u32;
        let mut start_address:u32;
        let  end_address = r13_sp;
        asm!("movw r0, 0xFFF8
             movt r0, 0x2000");

         asm!(
             "MOV {0}, r0",
             out(reg) start_address
         );

         let stack_size = (start_address - end_address) + 4;
        // leaving first xyz K for program i.e start at 0x0801_0000
         let mut fram_start_address = Volatile::new(0x6000_9000);
         let mut fram_end_address = Volatile::new(0x6000_FFFF);    

        let mut checkpoint_size= Volatile::new(0u32);
        asm::dmb();
        // 1. stack size
        // 2. 4 bytes -> 0xf1f1_f1f1 (end of stack in the frame magic number)
        // 3. 16 * 4 -> all the cpu registers
        // 4. 4 bytes -> size of frame
        // 5. 4 bytes -> 0xDEADBEEF (magic number to indicate the static checkpoint)
        checkpoint_size.write(stack_size+4+16*4 +4 +4);
        //asm::dmb();

        // loop{
        //     let mut offset = ptr::read_volatile(fram_start_address.read() as *const u32);
        //     if offset == 0{
        //         break;
        //     }
        //     fram_start_address.write(fram_start_address.read() + offset); 
        //     if fram_start_address.read() + checkpoint_size.read() >= fram_end_address.read() {
        //        //erase_all(&mut flash);
        //        fram_start_address = Volatile::new(0x0800_7000);
        //        break;
        //     }
        // }
        //asm::dmb();
        //write the size of packet at the begining of the packet
         ptr::write_volatile(  (fram_start_address.read()) as *mut u32, checkpoint_size.read() as u32); 
        fram_start_address.write(fram_start_address.read()+4);
        if c_type {
            //write at the begining of checkpoint fram so magic number indicate jit or static checkpoint
             ptr::write_volatile(  fram_start_address.read() as *mut u32, 0xDEADBEEF as u32);
        }
        else{
             ptr::write_volatile(  fram_start_address.read() as *mut u32,  0x0000_0001 as u32);
        }
        fram_start_address.write(fram_start_address.read()+4);
         while start_address >= end_address{
            let mut data = Volatile::new(0u32);
            data.write(core::ptr::read_volatile(start_address as * const u32));
             ptr::write_volatile(  fram_start_address.read() as *mut u32, data.read() as u32);
            fram_start_address.write(fram_start_address.read() +1* 4);
            // Move to the next address based on the size of the type
            start_address = start_address-4;
            
        }

    //mark the end of the stack
     ptr::write_volatile(  (fram_start_address.read()) as *mut u32, 0xf1f1_f1f1 as u32);
    fram_start_address.write(fram_start_address.read() + 4);


    // for i in 0..15{
    //      ptr::write_volatile(  0x0800_9060 as u32, r0_value as u32);
    //       fram_start_address = fram_start_address + 4;
    // }

     ptr::write_volatile(  fram_start_address.read() as *mut  u32, r0_value as u32);
     ptr::write_volatile(  (fram_start_address.read()+4) as  *mut u32, r1_value as u32);
     ptr::write_volatile( (fram_start_address.read()+8) as *mut u32, r2_value as u32);
     ptr::write_volatile( (fram_start_address.read()+12) as *mut u32, r3_value as u32);
     ptr::write_volatile( (fram_start_address.read()+16)  as *mut u32, r4_value as u32);
     ptr::write_volatile( (fram_start_address.read()+20) as *mut u32, r5_value as u32);
     ptr::write_volatile( (fram_start_address.read()+24) as *mut u32, r6_value as u32);
     ptr::write_volatile( (fram_start_address.read()+28) as *mut u32, r7_value as u32);
     ptr::write_volatile( (fram_start_address.read()+32) as *mut u32, r8_value as u32);
     ptr::write_volatile( (fram_start_address.read()+36) as *mut u32, r9_value as u32);
     ptr::write_volatile( (fram_start_address.read()+40) as *mut u32, r10_value as u32);
     ptr::write_volatile( (fram_start_address.read()+44) as *mut u32, r11_value as u32);
     ptr::write_volatile( (fram_start_address.read()+48) as *mut u32, r12_value as u32);
     ptr::write_volatile( (fram_start_address.read()+52) as *mut u32, r13_sp as u32);
     ptr::write_volatile( (fram_start_address.read()+56) as *mut u32, r14_lr as u32);
     ptr::write_volatile( (fram_start_address.read()+60) as *mut  u32, r15_pc as u32);
   
    }     
}

pub fn erase_all(flash: &mut FLASH){
    let start_address = 0x0803_0000;

    for i in 0..100{
        let page = start_address + i * 2*1024;
         erase_page(flash,  page);
    }

}

pub fn restore_globals(){
    unsafe{
        let mut restore_ctr: u16 = 0;
        loop {
            if *counter == restore_ctr {
                break;
            }

           ptr::write(transcation_log as *mut u32, transcation_log + 8);
           transcation_log += 12;

           restore_ctr = restore_ctr + 1; 
        }

    }
}   

pub fn restore_globals1(){
    unsafe{
        let mut restore_ctr:u16 = 0;
        loop {
            if *counter == restore_ctr {
                break;
            }

            let mut combined:u32 = 0;
            for i in 0..4 {
                combined |= (ptr::read((transcation_log + i) as *const u32) << (i * 8));
            
            }
            
            let mut size:u8 = ptr::read( transcation_log as *const u8);
        
            for i in 0..size{
                ptr::write((combined + i as u32) as *mut u8,*((transcation_log + i as u32) as *const u8));
            }
            combined =  combined + size as u32;

            //let end = ptr::read(combined as *const u8);
            restore_ctr += 1;

  
            // if end == 0xFB{
            //     break;
            // }
        }
    }
}
pub fn restore()->bool{
    unsafe {
        let mut fram_start_address = 0x6000_9000;
        let packet_size = ptr::read_volatile(0x6000_9000 as *const u32);
        //let r0_flash = ptr::read_volatile(0x0800_9060 as *const u32);
        if packet_size == 0 { //0xffff_ffff
            return false
        }
        // if  ptr::read_volatile((fram_start_address + packet_size) as *const u32)== 0{
        //     return  false;
        // }

        let mut offset:u32 = 0;
        // think about multiple conditions where it could break
        //1. There could multiple failed checkpoints before a successful checkpoint.
        //2. The last checkpoint could be a failed(incomplete) checkpoint.
        // loop{
            
        //     offset = ptr::read_volatile(fram_start_address  as *const u32);
  
        //     if  ptr::read_volatile((fram_start_address + offset) as *const u32) == 0{
        //         break;
        //     }
    
        //     fram_start_address+=offset;
        // }
        fram_start_address+=4;

        if  ptr::read_volatile(fram_start_address as *const u32) == 0xDEAD_BEEF{
            //restore_globals();
            //ptr::write(counter as *mut u8,0);
            *counter = 0;
        }

        // let mut end_address = 0x0801_0004 + packet_size;
        // let recent_frame_size: u32 = ptr::read_volatile(end_address as *const u32);
        // let mut recent_frame_start_address = end_address - recent_frame_size;

        fram_start_address+=4;

        asm!(
            "mov r0, {0}",
            in(reg) fram_start_address
        );

        //set sp to 0x0200_fffc
        asm!("movw r1, 0xfff8
        movt r1, 0x02000");
        asm!("msr msp, r1");

        asm!("movw r3, 0xf1f1
        movt r3, 0xf1f1");
    
        asm!("1:
            ldr r1, [r0, #4]
            cmp r1, r3
            beq 2f
            push {{r1}}
            adds r0, r0, #4
            b 1b
            2:");     

        asm!("adds r0, r0, #4");
        asm!("adds r0, r0, #4");

        asm!( "LDR r1, [r0]");
        asm!("Push {{r1}}");

        asm!("adds r0, r0, #4");
        asm!( "LDR r1, [r0]");

        asm!("adds r0, r0, #4");
        asm!( "LDR r2, [r0]");

        asm!("adds r0, r0, #4");
        asm!( "LDR r3, [r0]");

        asm!("adds r0, r0, #4");
        asm!( "LDR r4, [r0]");

        asm!("adds r0, r0, #4");
        asm!( "LDR r5, [r0]");

        asm!("adds r0, r0, #4");
        asm!( "LDR r6, [r0]");

        asm!("adds r0, r0, #4");
        asm!( "LDR r7, [r0]");

        asm!("adds r0, r0, #4");
        asm!( "LDR r8, [r0]");

        asm!("adds r0, r0, #4");
        asm!( "LDR r9, [r0]");

        asm!("adds r0, r0, #4");
        asm!( "LDR r10, [r0]");

        asm!("adds r0, r0, #4");
        asm!( "LDR r11, [r0]");

        asm!("adds r0, r0, #4");
        asm!( "LDR r12, [r0]");

        asm!("adds r0, r0, #4");
        //asm!( "LDR r13, [r0]"); //no need to do this

        asm!("adds r0, r0, #4");
        asm!( "LDR r14, [r0]");
        asm!( "mov pc, r14");  // pc --> r15

       // asm!("POP {{r0}}");
        
    //     asm!("adds sp, sp, #56");
    //     asm!("adds sp, sp, #8");

    //     // asm!("POP {{r0, r1, r2, r3, r12, lr}}");
    //     // asm!("LDMIA sp!, {{pc, xPSR}}");

    //     asm!("POP {{r0, r1, r2, r3}}");
    //     asm!("adds sp, sp, #4");
    //     asm!("POP {{r4}}");
    //     asm!("adds sp, sp, #16"); //stack alignment issue
    //     asm!("adds sp, sp, #64"); //stack alignment issue


    //    // asm!("POP {{r4, r5, r6, r7}}");
    //    // asm!("MSR xPSR, r7");
    //     asm!("mov pc, r4");    // pc is r15
        //asm!("mov r15, r14"); // I am writing my own function to handle interrupt 


    }
    return true;
}

pub fn delete_pg(page: u32){
    unsafe{
    let mut dp = Peripherals::steal();
    let mut flash= &mut dp.FLASH;
    unlock(&mut flash); 
    wait_ready(&flash);
    erase_page(&mut flash,  page);
    }
}
pub fn delete_all_pg(){
    let start_address = 0x0803_0000;
    unsafe{
        let mut dp = Peripherals::steal();
        let mut flash= &mut dp.FLASH;
        for i in 0..25{
            let page = start_address + i * 2*1024;
            unlock(&mut flash); 
            wait_ready(&flash);
            erase_page(&mut flash,  page);
        }
       // drop(flash);
    }

}