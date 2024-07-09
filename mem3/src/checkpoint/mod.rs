
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
pub static mut execution_mode: bool = true;  //1. true is jit 2.flase is static 
pub static mut counter: *mut u8= 0x60003002 as *mut u8;



pub fn save_variables<T>(mem_loc: *const T, size: usize) {
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
            "add sp, #280"
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
            "sub sp, #280"
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
            "add r0, #288",
        );
    }
    unsafe {
        asm!(
            "MOV {0}, r0",
            out(reg) r13_sp
        );
    }
    unsafe{

        let  dp = Peripherals::steal();
        let mut flash= dp.FLASH;
        unlock(&mut flash);
        wait_ready(&flash);

   
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
         let mut flash_start_address = Volatile::new(0x0803_0000);
         let mut flash_end_address = Volatile::new(0x0807_FFFF);    

        let mut checkpoint_size= Volatile::new(0u32);
        asm::dmb();
        // 1. stack size
        // 2. 4 bytes -> 0xf1f1_f1f1 (end of stack in the frame magic number)
        // 3. 16 * 4 -> all the cpu registers
        // 4. 4 bytes -> size of frame
        // 5. 4 bytes -> 0xDEADBEEF (magic number to indicate the static checkpoint)
        checkpoint_size.write(stack_size+4+16*4 +4 +4);
        asm::dmb();

        loop{
            let mut offset = ptr::read_volatile(flash_start_address.read() as *const u32);
            if offset == 0xffff_ffff{
                break;
            }
            flash_start_address.write(flash_start_address.read() + offset); 
            if flash_start_address.read() + checkpoint_size.read() >= flash_end_address.read() {
               erase_all(&mut flash);
               flash_start_address = Volatile::new(0x0803_0000);
               break;
            }
        }
        asm::dmb();
        //write the size of packet at the begining of the packet
        write_to_flash(&mut flash,  (flash_start_address.read()) as u32, checkpoint_size.read() as u32); 
        flash_start_address.write(flash_start_address.read()+4);
        asm::dmb();
           // Code that involves Flash write
    //      if offset == 0xffff_ffff {
    //   // stack_size + 4(0xffff_ffff to signal end of stack) + 16*4(store registers) + 4 (size of a packet)
    //         write_to_flash(&mut flash,  flash_start_address as u32, checkpoint_size+1-1  as u32);
    //         flash_start_address = flash_start_address + 4;
    //      }
    //      else{
    //         flash_start_address = flash_start_address + offset; 
    //         let flash_end_address = 0x0807_FFFF-1+1;
    //         if flash_end_address - flash_start_address < checkpoint_size {
    //             //clear flash
    //             //set start address and offset
    //             erase_all(&mut flash);
    //             flash_start_address = 0x0801_0004;
    //             offset = 0;
    //         }
    //         write_to_flash(&mut flash,  0x0801_0000 as u32, offset+checkpoint_size+1-1  as u32);
    //      }
    asm::dmb(); 
        if c_type {
            //write at the begining of checkpoint fram so magic number indicate jit or static checkpoint
            write_to_flash(&mut flash,  flash_start_address.read() as u32, 0xDEADBEEF as u32);
        }
        else{
            write_to_flash(&mut flash,  flash_start_address.read() as u32,  0x0000_0001 as u32);
        }
         while start_address >= end_address{
            let mut data = Volatile::new(0u32);
            data.write(core::ptr::read_volatile(start_address as * const u32));
            write_to_flash(&mut flash,  flash_start_address.read() as u32, data.read() as u32);
            flash_start_address.write(flash_start_address.read() +1* 4);
            // Move to the next address based on the size of the type
            start_address = start_address-4;
            
        }
        asm::dmb();
    asm::dmb();
    //mark the end of the stack
    write_to_flash(&mut flash,  (flash_start_address.read()) as u32, 0xf1f1_f1f1 as u32);
    flash_start_address.write(flash_start_address.read() + 4);
    asm::dmb();

    // for i in 0..15{
    //     write_to_flash(&mut flash,  0x0800_9060 as u32, r0_value as u32);
    //       flash_start_address = flash_start_address + 4;
    // }

    write_to_flash(&mut flash,  flash_start_address.read() as u32, r0_value as u32);
    write_to_flash(&mut flash,  flash_start_address.read()+4 as u32, r1_value as u32);
    write_to_flash(&mut flash,  flash_start_address.read()+8 as u32, r2_value as u32);
    write_to_flash(&mut flash,  flash_start_address.read()+12 as u32, r3_value as u32);
    write_to_flash(&mut flash,  flash_start_address.read()+16 as u32, r4_value as u32);
    write_to_flash(&mut flash,  flash_start_address.read()+20 as u32, r5_value as u32);
    write_to_flash(&mut flash,  flash_start_address.read()+24 as u32, r6_value as u32);
    write_to_flash(&mut flash,  flash_start_address.read()+28 as u32, r7_value as u32);
    write_to_flash(&mut flash,  flash_start_address.read()+32 as u32, r8_value as u32);
    write_to_flash(&mut flash,  flash_start_address.read()+36 as u32, r9_value as u32);
    write_to_flash(&mut flash,  flash_start_address.read()+40 as u32, r10_value as u32);
    write_to_flash(&mut flash,  flash_start_address.read()+44 as u32, r11_value as u32);
    write_to_flash(&mut flash,  flash_start_address.read()+48 as u32, r12_value as u32);
    write_to_flash(&mut flash,  flash_start_address.read()+52 as u32, r13_sp as u32);
    write_to_flash(&mut flash,  flash_start_address.read()+56 as u32, r14_lr as u32);
    write_to_flash(&mut flash,  flash_start_address.read()+60 as u32, r15_pc as u32);
    drop(flash);
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
        let mut restore_ctr:u8 = 0;
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

            let end = ptr::read(combined as *const u8);
            restore_ctr += 1;

  
            // if end == 0xFB{
            //     break;
            // }
        }
    }
}
pub fn restore()->bool{
    unsafe {
        let mut flash_start_address = 0x0803_0000;
        let packet_size = ptr::read_volatile(0x0803_0000 as *const u32);
        //let r0_flash = ptr::read_volatile(0x0800_9060 as *const u32);
        if packet_size == 0xffff_ffff {
            return false
        }
        if  ptr::read_volatile((flash_start_address + packet_size) as *const u32)==0xffff_ffff{
            return  false;
        }

        let mut offset:u32 = 0;
        // think about multiple conditions where it could break
        //1. There could multiple failed checkpoints before a successful checkpoint.
        //2. The last checkpoint could be a failed(incomplete) checkpoint.
        loop{
            
            offset = ptr::read_volatile(flash_start_address  as *const u32);
  
            if  ptr::read_volatile((flash_start_address + offset) as *const u32) == 0xffff_ffff{
                break;
            }
    
            flash_start_address+=offset;
        }
        flash_start_address+=4;

        if  ptr::read_volatile(flash_start_address as *const u32) == 0xDEAD_BEEF{
            restore_globals();
            //ptr::write(counter as *mut u8,0);
            *counter = 0;
        }

        // let mut end_address = 0x0801_0004 + packet_size;
        // let recent_frame_size: u32 = ptr::read_volatile(end_address as *const u32);
        // let mut recent_frame_start_address = end_address - recent_frame_size;

        flash_start_address+=4;

        asm!(
            "mov r0, {0}",
            in(reg) flash_start_address
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

        asm!("POP {{r0}}");
        
        asm!("adds sp, sp, #56");
        asm!("adds sp, sp, #8");

        // asm!("POP {{r0, r1, r2, r3, r12, lr}}");
        // asm!("LDMIA sp!, {{pc, xPSR}}");

        asm!("POP {{r0, r1, r2, r3}}");
        asm!("adds sp, sp, #4");
        asm!("POP {{r4}}");
        asm!("adds sp, sp, #16"); //stack alignment issue
        asm!("adds sp, sp, #64"); //stack alignment issue


       // asm!("POP {{r4, r5, r6, r7}}");
       // asm!("MSR xPSR, r7");
        asm!("mov pc, r4");    // pc is r15
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