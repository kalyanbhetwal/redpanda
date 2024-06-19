#![allow(unsafe_code, unused)]
#![no_main]
#![no_std]

use core::{borrow::BorrowMut, fmt::Result};
use core::ptr;

use cortex_m::asm::{nop, self};
use panic_halt as _;

use cortex_m_rt::entry;
use stm32f3xx_hal_v2::flash::FlashExt;
use::core::arch::asm;
use cortex_m_semihosting::{debug, hprintln};
use stm32f3xx_hal_v2::{flash::ACR, pac::Peripherals, pac::FLASH};

use volatile::Volatile;
use core::sync::atomic::{compiler_fence, Ordering};



const UNLOCK_KEY1: u32 = 0x4567_0123;
const UNLOCK_KEY2: u32 = 0xCDEF_89AB;

fn unlock(flash: &mut FLASH) ->bool{

    if flash.cr.read().lock().bit_is_clear(){
        return true;
    }

    flash.keyr.write(|w| unsafe { w.bits(UNLOCK_KEY1) });
    flash.keyr.write(|w| unsafe { w.bits(UNLOCK_KEY2) });

    if flash.cr.read().lock().bit_is_clear() {
        return true;
    } else {
        return false;
    }
}

fn wait_ready(flash: &FLASH) {
    while flash.sr.read().bsy().bit() {}
}

fn clear_error_flags(flash: &FLASH) {
    if flash.sr.read().wrprterr().bit_is_set() {
        flash.sr.modify(|_, w| w.wrprterr().set_bit());
    }
    if flash.sr.read().pgerr().bit_is_set() {
        flash.sr.modify(|_, w| w.pgerr().set_bit());
    }
}

fn erase_page(flash: &mut FLASH, page: u32){

    // 1. Check that no Flash memory operation is ongoing by checking the BSY bit in the Flash
    // status register (FLASH_SR).
    if flash.sr.read().bsy().bit_is_set() {
        hprintln!("Flash is busy.");
     }

    // 2. Check and clear all error programming flags due to a previous programming. If not,
     // PGSERR is set.
    clear_error_flags(&flash);

    // 3. Set the PER bit and select the page you wish to erase (PNB). For dual bank variants:
     //  - with the associated bank(BKER) in the Flash control register (FLASH_CR).

     flash.cr.modify(|_, w| w.per().set_bit());

     // Program the FLASH_CR register
     // regs.ar.modify(|_, w| w.far().bits(page as u8));
     flash.ar.write(|w| unsafe { w.bits(page as u32) });


     // 4. Set the STRT bit in the FLASH_CR register.
     flash.cr.modify(|_, w| w.strt().set_bit());

    // 5. Wait for the BSY bit to be cleared in the FLASH_SR register.
      
    while flash.sr.read().bsy().bit_is_set() {}

    // 6. lock the flash
    while flash.sr.read().bsy().bit_is_set() {}
    flash.cr.modify(|_, w| w.lock().set_bit());

}

fn write_to_flash(flash: &mut FLASH, addr: u32, data: u32) {
        unlock(flash);

        // 1. Check that no Flash memory operation is ongoing by checking the BSY bit in the Flash
        if flash.sr.read().bsy().bit_is_set() {
            hprintln!("Flash is busy.");
        }
         
        clear_error_flags(&flash);
       // 2. Set the PG bit in the Flash control register (FLASH_CR).
       flash.cr.modify(|_, w| w.pg().set_bit());


        // 3. Perform the data write (half-word) at the desired address.
        unsafe{
                ptr::write_volatile(addr as *mut u16, data as u16);
                ptr::write_volatile((addr as usize + 2) as * mut u16, (data.wrapping_shr(16)) as u16);
        }
        asm::dsb();
        // 4. Wait for the BSY bit to be cleared in the FLASH_SR register.

        while flash.sr.read().bsy().bit_is_set() {}
        // 5. lock the flash
        flash.cr.modify(|_, w| w.lock().set_bit());
        

        // 6. Check that EOP flag is set in the FLASH_SR register (meaning that the programming
        // operation has succeed), and clear it by software.
        if flash.sr.read().eop().bit_is_set() {
            flash.sr.modify(|_, w| w.eop().set_bit()); // Clear
        }

         // 6. Clear the PG bit in the FLASH_CR register if there no more programming request
        // anymore.
        flash.cr.modify(|_, w| w.pg().clear_bit());

}


fn erase_all(flash: &mut FLASH){
    let start_address = 0x0801_0000;

    for i in 0..255{
        let page = start_address + i * 2*1024;
         erase_page(flash,  page);
    }

}

fn delete_pg(page: u32){
    let dp = Peripherals::take().unwrap();
    let mut flash= dp.FLASH;
    unlock(& mut flash); 
    wait_ready(&flash);
    erase_page(&mut flash,  page);
}

#[no_mangle]
pub extern "C" fn main() -> ! {
  
 //delete_pg(0x0801_0000 as u32);

 let dp = Peripherals::take().unwrap();
 let mut flash= dp.FLASH;
 unlock(& mut flash); 
 wait_ready(&flash);
 unsafe {
    asm!("mov r0, #10
          mov r1, #20
    "); 


 let mut addr = volatile::ReadOnly::new(0x0801_0F10);

 let mut data = 40;


 write_to_flash(&mut flash, addr.read(), data);

    
    }
    unsafe {
        asm!("add r0, r1"); 
        }
  

    // exit QEMU
    // NOTE do not run this on hardware; it can corrupt OpenOCD state
    //debug::exit(debug::EXIT_SUCCESS);

   loop {}
}
