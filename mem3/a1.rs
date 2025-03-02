#![feature(prelude_import)]
#![allow(
    unsafe_code,
    unused,
    non_upper_case_globals,
    non_snake_case,
    non_camel_case_types
)]
#![no_main]
#![no_std]
#[prelude_import]
use core::prelude::rust_2018::*;
#[macro_use]
extern crate core;
extern crate compiler_builtins as _;
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
mod checkpoint {
    #![allow(unsafe_code, non_upper_case_globals)]
    pub mod my_flash {
        #![allow(unsafe_code, unused, non_upper_case_globals)]
        #![no_main]
        #![no_std]
        use core::mem;
        use core::ptr;
        use cortex_m::asm::{nop, self};
        use panic_halt as _;
        use cortex_m_rt::entry;
        use ::core::arch::asm;
        use cortex_m_semihosting::{debug, hprintln};
        use stm32f3xx_hal_v2::{
            self as hal, pac, prelude::*, flash::ACR, pac::Peripherals, pac::FLASH,
        };
        use volatile::Volatile;
        use stm32f3xx_hal_v2::hal::blocking::rng::Read;
        const UNLOCK_KEY1: u32 = 0x4567_0123;
        const UNLOCK_KEY2: u32 = 0xCDEF_89AB;
        pub fn unlock(flash: &mut FLASH) -> bool {
            if flash.cr.read().lock().bit_is_clear() {
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
        pub fn wait_ready(flash: &FLASH) {
            while flash.sr.read().bsy().bit() {}
        }
        pub fn clear_error_flags(flash: &FLASH) {
            if flash.sr.read().wrprterr().bit_is_set() {
                flash.sr.modify(|_, w| w.wrprterr().set_bit());
            }
            if flash.sr.read().pgerr().bit_is_set() {
                flash.sr.modify(|_, w| w.pgerr().set_bit());
            }
        }
        pub fn erase_page(flash: &mut FLASH, page: u32) {
            if flash.sr.read().bsy().bit_is_set() {
                ::cortex_m_semihosting::export::hstdout_str("Flash is busy.\n");
            }
            clear_error_flags(&flash);
            flash.cr.modify(|_, w| w.per().set_bit());
            flash.ar.write(|w| unsafe { w.bits(page as u32) });
            flash.cr.modify(|_, w| w.strt().set_bit());
            while flash.sr.read().bsy().bit_is_set() {}
            while flash.sr.read().bsy().bit_is_set() {}
            flash.cr.modify(|_, w| w.lock().set_bit());
        }
        pub fn write_to_flash(flash: &mut FLASH, addr: u32, data: u32) {
            unlock(flash);
            if flash.sr.read().bsy().bit_is_set() {
                ::cortex_m_semihosting::export::hstdout_str("Flash is busy.\n");
            }
            clear_error_flags(&flash);
            flash.cr.modify(|_, w| w.pg().set_bit());
            unsafe {
                ptr::write_volatile(addr as *mut u16, data as u16);
                ptr::write_volatile(
                    (addr as usize + 2) as *mut u16,
                    (data.wrapping_shr(16)) as u16,
                );
            }
            while flash.sr.read().bsy().bit_is_set() {}
            flash.cr.modify(|_, w| w.lock().set_bit());
            if flash.sr.read().eop().bit_is_set() {
                flash.sr.modify(|_, w| w.eop().set_bit());
            }
            flash.cr.modify(|_, w| w.pg().clear_bit());
        }
    }
    use my_flash::{unlock, wait_ready, clear_error_flags, erase_page, write_to_flash};
    use core::mem;
    use core::ptr;
    use cortex_m::asm::{nop, self};
    use cortex_m_semihosting::hprintln;
    use panic_halt as _;
    use ::core::arch::asm;
    use stm32f3xx_hal_v2::{pac::Peripherals, pac::FLASH};
    use volatile::Volatile;
    pub static mut transcation_log: u32 = 0x60004000;
    pub static mut data_loc: u32 = 0x60005000;
    pub static mut execution_mode: bool = true;
    pub static mut counter: *mut u16 = 0x60003000 as *mut u16;
    pub fn initialization() {
        let dp = Peripherals::take().unwrap();
        dp.RCC.cr.write(|w| w.hsion().set_bit());
        while dp.RCC.cr.read().hsirdy().bit_is_clear() {}
        dp.RCC.cr.modify(|_r, w| w.pllon().clear_bit());
        while dp.RCC.cr.read().pllrdy().bit_is_set() {}
        dp.RCC.cfgr.modify(|_, w| w.pllsrc().hsi_div_prediv());
        dp.RCC.cfgr2.modify(|_, w| w.prediv().div1());
        dp.RCC.cfgr.modify(|_, w| w.pllmul().mul2());
        dp.RCC.cr.modify(|_, w| w.pllon().on());
        while dp.RCC.cr.read().pllrdy().bit_is_clear() {}
        dp.RCC.cfgr.modify(|_, w| { w.hpre().div1().ppre1().div2().ppre2().div1() });
        dp.FLASH.acr.modify(|_, w| w.prftbe().enabled().latency().ws1());
        dp.RCC.cfgr.modify(|_, w| w.sw().pll());
        while dp.RCC.cfgr.read().sw().bits() != 0b10 {}
        while dp.RCC.cfgr.read().sws().bits() != 0b10 {}
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
        let gpiod = dp.GPIOD;
        let gpioe = dp.GPIOE;
        let gpiof = dp.GPIOF;
        let gpiog = dp.GPIOG;
        let gpioh = dp.GPIOH;
        dp.GPIOC.moder.write(|w| unsafe { w.moder10().bits(0b01) });
        dp.GPIOC.moder.write(|w| unsafe { w.moder11().bits(0b01) });
        dp.GPIOC.odr.write(|w| w.odr10().set_bit());
        dp.GPIOA.odr.write(|w| w.odr11().set_bit());
        gpioh.moder.modify(|_, w| { w.moder0().alternate() });
        gpioh.afrl.modify(|_, w| { w.afrl0().af12() });
        gpioh.ospeedr.modify(|_, w| w.ospeedr0().very_high_speed());
        gpioh.moder.modify(|_, w| { w.moder1().alternate() });
        gpioh.afrl.modify(|_, w| { w.afrl1().af12() });
        gpioh.ospeedr.modify(|_, w| w.ospeedr1().very_high_speed());
        gpiof.moder.modify(|_, w| { w.moder2().alternate() });
        gpiof.afrl.modify(|_, w| { w.afrl2().af12() });
        gpiof.ospeedr.modify(|_, w| w.ospeedr2().very_high_speed());
        gpiof.moder.modify(|_, w| { w.moder3().alternate() });
        gpiof.afrl.modify(|_, w| { w.afrl3().af12() });
        gpiof.ospeedr.modify(|_, w| w.ospeedr3().very_high_speed());
        gpiof.moder.modify(|_, w| { w.moder4().alternate() });
        gpiof.afrl.modify(|_, w| { w.afrl4().af12() });
        gpiof.ospeedr.modify(|_, w| w.ospeedr4().very_high_speed());
        gpiof.moder.modify(|_, w| { w.moder5().alternate() });
        gpiof.afrl.modify(|_, w| { w.afrl5().af12() });
        gpiof.ospeedr.modify(|_, w| w.ospeedr5().very_high_speed());
        gpiof.moder.modify(|_, w| { w.moder12().alternate() });
        gpiof.afrh.modify(|_, w| { w.afrh12().af12() });
        gpiof.ospeedr.modify(|_, w| w.ospeedr12().very_high_speed());
        gpiof.moder.modify(|_, w| { w.moder13().alternate() });
        gpiof.afrh.modify(|_, w| { w.afrh13().af12() });
        gpiof.ospeedr.modify(|_, w| w.ospeedr13().very_high_speed());
        gpiof.moder.modify(|_, w| { w.moder14().alternate() });
        gpiof.afrh.modify(|_, w| { w.afrh14().af12() });
        gpiof.ospeedr.modify(|_, w| w.ospeedr14().very_high_speed());
        gpiof.moder.modify(|_, w| { w.moder15().alternate() });
        gpiof.afrh.modify(|_, w| { w.afrh15().af12() });
        gpiof.ospeedr.modify(|_, w| w.ospeedr15().very_high_speed());
        gpiog.moder.modify(|_, w| { w.moder0().alternate() });
        gpiog.afrl.modify(|_, w| { w.afrl0().af12() });
        gpiog.ospeedr.modify(|_, w| w.ospeedr0().very_high_speed());
        gpiog.moder.modify(|_, w| { w.moder1().alternate() });
        gpiog.afrl.modify(|_, w| { w.afrl1().af12() });
        gpiog.ospeedr.modify(|_, w| w.ospeedr1().very_high_speed());
        gpiog.moder.modify(|_, w| { w.moder2().alternate() });
        gpiog.afrl.modify(|_, w| { w.afrl2().af12() });
        gpiog.ospeedr.modify(|_, w| w.ospeedr2().very_high_speed());
        gpiog.moder.modify(|_, w| { w.moder3().alternate() });
        gpiog.afrl.modify(|_, w| { w.afrl3().af12() });
        gpiog.ospeedr.modify(|_, w| w.ospeedr3().very_high_speed());
        gpiog.moder.modify(|_, w| { w.moder4().alternate() });
        gpiog.afrl.modify(|_, w| { w.afrl4().af12() });
        gpiog.ospeedr.modify(|_, w| w.ospeedr4().very_high_speed());
        gpiog.moder.modify(|_, w| { w.moder5().alternate() });
        gpiog.afrl.modify(|_, w| { w.afrl5().af12() });
        gpiog.ospeedr.modify(|_, w| w.ospeedr5().very_high_speed());
        gpiod.moder.modify(|_, w| { w.moder14().alternate() });
        gpiod.afrh.modify(|_, w| { w.afrh14().af12() });
        gpiod.ospeedr.modify(|_, w| w.ospeedr14().very_high_speed());
        gpiod.moder.modify(|_, w| { w.moder15().alternate() });
        gpiod.afrh.modify(|_, w| { w.afrh15().af12() });
        gpiod.ospeedr.modify(|_, w| w.ospeedr15().very_high_speed());
        gpiod.moder.modify(|_, w| { w.moder0().alternate() });
        gpiod.afrl.modify(|_, w| { w.afrl0().af12() });
        gpiod.ospeedr.modify(|_, w| w.ospeedr0().very_high_speed());
        gpiod.moder.modify(|_, w| { w.moder1().alternate() });
        gpiod.afrl.modify(|_, w| { w.afrl1().af12() });
        gpiod.ospeedr.modify(|_, w| w.ospeedr1().very_high_speed());
        gpioe.moder.modify(|_, w| { w.moder7().alternate() });
        gpioe.afrl.modify(|_, w| { w.afrl7().af12() });
        gpioe.ospeedr.modify(|_, w| w.ospeedr7().very_high_speed());
        gpioe.moder.modify(|_, w| { w.moder8().alternate() });
        gpioe.afrh.modify(|_, w| { w.afrh8().af12() });
        gpioe.ospeedr.modify(|_, w| w.ospeedr8().very_high_speed());
        gpioe.moder.modify(|_, w| { w.moder9().alternate() });
        gpioe.afrh.modify(|_, w| { w.afrh9().af12() });
        gpioe.ospeedr.modify(|_, w| w.ospeedr9().very_high_speed());
        gpioe.moder.modify(|_, w| { w.moder10().alternate() });
        gpioe.afrh.modify(|_, w| { w.afrh10().af12() });
        gpioe.ospeedr.modify(|_, w| w.ospeedr10().very_high_speed());
        gpioe.moder.modify(|_, w| { w.moder11().alternate() });
        gpioe.afrh.modify(|_, w| { w.afrh11().af12() });
        gpioe.ospeedr.modify(|_, w| w.ospeedr11().very_high_speed());
        gpioe.moder.modify(|_, w| { w.moder12().alternate() });
        gpioe.afrh.modify(|_, w| { w.afrh12().af12() });
        gpioe.ospeedr.modify(|_, w| w.ospeedr12().very_high_speed());
        gpioe.moder.modify(|_, w| { w.moder13().alternate() });
        gpioe.afrh.modify(|_, w| { w.afrh13().af12() });
        gpioe.ospeedr.modify(|_, w| w.ospeedr13().very_high_speed());
        gpioe.moder.modify(|_, w| { w.moder14().alternate() });
        gpioe.afrh.modify(|_, w| { w.afrh14().af12() });
        gpioe.ospeedr.modify(|_, w| w.ospeedr14().very_high_speed());
        gpioe.moder.modify(|_, w| { w.moder15().alternate() });
        gpioe.afrh.modify(|_, w| { w.afrh15().af12() });
        gpioe.ospeedr.modify(|_, w| w.ospeedr15().very_high_speed());
        gpiod.moder.modify(|_, w| { w.moder8().alternate() });
        gpiod.afrh.modify(|_, w| { w.afrh8().af12() });
        gpiod.ospeedr.modify(|_, w| w.ospeedr8().very_high_speed());
        gpiod.moder.modify(|_, w| { w.moder9().alternate() });
        gpiod.afrh.modify(|_, w| { w.afrh9().af12() });
        gpiod.ospeedr.modify(|_, w| w.ospeedr9().very_high_speed());
        gpiod.moder.modify(|_, w| { w.moder10().alternate() });
        gpiod.afrh.modify(|_, w| { w.afrh10().af12() });
        gpiod.ospeedr.modify(|_, w| w.ospeedr10().very_high_speed());
        gpiod.moder.modify(|_, w| { w.moder7().alternate() });
        gpiod.afrl.modify(|_, w| { w.afrl7().af12() });
        gpiod.ospeedr.modify(|_, w| w.ospeedr7().very_high_speed());
        gpiod.moder.modify(|_, w| { w.moder4().alternate() });
        gpiod.afrl.modify(|_, w| { w.afrl4().af12() });
        gpiod.ospeedr.modify(|_, w| w.ospeedr4().very_high_speed());
        gpiod.moder.modify(|_, w| { w.moder5().alternate() });
        gpiod.afrl.modify(|_, w| { w.afrl5().af12() });
        gpiod.ospeedr.modify(|_, w| w.ospeedr5().very_high_speed());
        unsafe {
            dp.FMC
                .bcr1
                .modify(|_, w| {
                    w.mbken().set_bit();
                    w.mtyp().bits(0b00);
                    w.mwid().bits(0b01);
                    w.bursten().clear_bit();
                    w.wren().clear_bit();
                    w.muxen().clear_bit();
                    w.extmod().clear_bit();
                    w.asyncwait().clear_bit();
                    w
                });
            dp.FMC
                .btr1
                .modify(|_, w| {
                    w.addset().bits(0x1);
                    w.datast().bits(0x90);
                    w.addhld().bits(0x1);
                    w.busturn().bits(0x90);
                    w.clkdiv().bits(0x4);
                    w.datlat().bits(0x90);
                    w.accmod().bits(0x0);
                    w
                });
        }
    }
    pub fn save_variables<T>(mem_loc: *const T, size: usize) {
        unsafe {
            ::cortex_m_semihosting::export::hstdout_fmt(
                format_args!("mem loc {0:?}\n", mem_loc),
            );
            ptr::write(transcation_log as *mut u32, mem_loc as u32);
            transcation_log += 4;
            ptr::write(transcation_log as *mut u32, size as u32);
            transcation_log += 4;
            ptr::write(data_loc as *mut u16, (mem_loc as *const u16) as u16);
            data_loc += 2;
            *counter = *counter + 1;
        }
    }
    pub fn save_variables1<T>(mem_loc: *const T, size: usize) {
        unsafe {
            let mem_loc_u8 = mem_loc as *const u8;
            for i in 0..4 {
                let byte = (mem_loc_u8 as u32 >> (i * 8)) as u8;
                ::cortex_m_semihosting::export::hstdout_fmt(
                    format_args!("bytes {0:0x}\n", byte),
                );
                ptr::write((transcation_log + 2 * i as u32) as *mut u8, byte);
            }
            transcation_log += 2 * 4;
            ptr::write(transcation_log as *mut u8, size as u8);
            transcation_log += 2 * 1;
            for i in 0..size {
                let byte = *mem_loc_u8.add(i);
                ::cortex_m_semihosting::export::hstdout_fmt(
                    format_args!("the logged byte {0}\n", byte),
                );
                ptr::write((transcation_log + 2 * i as u32) as *mut u8, byte);
            }
            transcation_log = transcation_log + 2 * size as u32;
            *counter += 1;
        }
        ::cortex_m_semihosting::export::hstdout_fmt(
            format_args!("Address: {0:p}, Size: {1} bytes\n", mem_loc, size),
        );
    }
    pub fn start_atomic() {
        unsafe {
            execution_mode = false;
        }
    }
    pub fn end_atomic() {
        unsafe {
            transcation_log = 0x60004000;
        }
        unsafe {
            execution_mode = true;
        }
    }
    #[no_mangle]
    pub fn checkpoint(c_type: bool) {
        unsafe {
            asm!("add sp, #256");
        }
        unsafe {
            asm!("pop {{r7}}");
        }
        unsafe {
            asm!("push {{r7}}");
        }
        unsafe {
            asm!("sub sp, #256");
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
            asm!("MOV {0}, r0", out(reg) r0_value);
        }
        unsafe {
            asm!("MOV {0}, r1", out(reg) r1_value);
        }
        unsafe {
            asm!("MOV {0}, r2", out(reg) r2_value);
        }
        unsafe {
            asm!("MOV {0}, r3", out(reg) r3_value);
        }
        unsafe {
            asm!("MOV {0}, r4", out(reg) r4_value);
        }
        unsafe {
            asm!("MOV {0}, r5", out(reg) r5_value);
        }
        unsafe {
            asm!("MOV {0}, r6", out(reg) r6_value);
        }
        unsafe {
            asm!("MOV {0}, r7", out(reg) r7_value);
        }
        unsafe {
            asm!("MOV {0}, r8", out(reg) r8_value);
        }
        unsafe {
            asm!("MOV {0}, r9", out(reg) r9_value);
        }
        unsafe {
            asm!("MOV {0}, r10", out(reg) r10_value);
        }
        unsafe {
            asm!("MOV {0}, r11", out(reg) r11_value);
        }
        unsafe {
            asm!("MOV {0}, r12", out(reg) r12_value);
        }
        unsafe {
            asm!("MOV {0}, r14", out(reg) r14_lr);
        }
        unsafe {
            asm!("MOV {0}, r15", out(reg) r15_pc);
        }
        unsafe {
            asm!("MOV r0, sp");
        }
        unsafe {
            asm!("add r0, #264");
        }
        unsafe {
            asm!("MOV {0}, r0", out(reg) r13_sp);
        }
        unsafe {
            let mut start_address: u32;
            let end_address = r13_sp;
            asm!("movw r0, 0xFFF8\n             movt r0, 0x2000");
            asm!("MOV {0}, r0", out(reg) start_address);
            let stack_size = (start_address - end_address) + 4;
            let mut fram_start_address = Volatile::new(0x6000_9000);
            let mut fram_end_address = Volatile::new(0x6000_FFFF);
            let mut checkpoint_size = Volatile::new(0u32);
            asm::dmb();
            checkpoint_size.write(stack_size + 4 + 16 * 4 + 4 + 4);
            ptr::write_volatile(
                (fram_start_address.read()) as *mut u32,
                checkpoint_size.read() as u32,
            );
            fram_start_address.write(fram_start_address.read() + 4);
            if c_type {
                ptr::write_volatile(
                    fram_start_address.read() as *mut u32,
                    0xDEADBEEF as u32,
                );
            } else {
                ptr::write_volatile(
                    fram_start_address.read() as *mut u32,
                    0x0000_0001 as u32,
                );
            }
            fram_start_address.write(fram_start_address.read() + 4);
            while start_address >= end_address {
                let mut data = Volatile::new(0u32);
                data.write(core::ptr::read_volatile(start_address as *const u32));
                ptr::write_volatile(
                    fram_start_address.read() as *mut u32,
                    data.read() as u32,
                );
                fram_start_address.write(fram_start_address.read() + 1 * 4);
                start_address = start_address - 4;
            }
            ptr::write_volatile(
                (fram_start_address.read()) as *mut u32,
                0xf1f1_f1f1 as u32,
            );
            fram_start_address.write(fram_start_address.read() + 4);
            ptr::write_volatile(fram_start_address.read() as *mut u32, r0_value as u32);
            ptr::write_volatile(
                (fram_start_address.read() + 4) as *mut u32,
                r1_value as u32,
            );
            ptr::write_volatile(
                (fram_start_address.read() + 8) as *mut u32,
                r2_value as u32,
            );
            ptr::write_volatile(
                (fram_start_address.read() + 12) as *mut u32,
                r3_value as u32,
            );
            ptr::write_volatile(
                (fram_start_address.read() + 16) as *mut u32,
                r4_value as u32,
            );
            ptr::write_volatile(
                (fram_start_address.read() + 20) as *mut u32,
                r5_value as u32,
            );
            ptr::write_volatile(
                (fram_start_address.read() + 24) as *mut u32,
                r6_value as u32,
            );
            ptr::write_volatile(
                (fram_start_address.read() + 28) as *mut u32,
                r7_value as u32,
            );
            ptr::write_volatile(
                (fram_start_address.read() + 32) as *mut u32,
                r8_value as u32,
            );
            ptr::write_volatile(
                (fram_start_address.read() + 36) as *mut u32,
                r9_value as u32,
            );
            ptr::write_volatile(
                (fram_start_address.read() + 40) as *mut u32,
                r10_value as u32,
            );
            ptr::write_volatile(
                (fram_start_address.read() + 44) as *mut u32,
                r11_value as u32,
            );
            ptr::write_volatile(
                (fram_start_address.read() + 48) as *mut u32,
                r12_value as u32,
            );
            ptr::write_volatile(
                (fram_start_address.read() + 52) as *mut u32,
                r13_sp as u32,
            );
            ptr::write_volatile(
                (fram_start_address.read() + 56) as *mut u32,
                r14_lr as u32,
            );
            ptr::write_volatile(
                (fram_start_address.read() + 60) as *mut u32,
                r15_pc as u32,
            );
        }
    }
    pub fn erase_all(flash: &mut FLASH) {
        let start_address = 0x0803_0000;
        for i in 0..100 {
            let page = start_address + i * 2 * 1024;
            erase_page(flash, page);
        }
    }
    pub fn restore_globals() {
        unsafe {
            let mut restore_ctr: u16 = 0;
            loop {
                if *counter == restore_ctr {
                    break;
                }
                ptr::write(transcation_log as *mut u32, data_loc as u32);
                transcation_log += 4;
                let size = ptr::read(transcation_log as *mut u32);
                transcation_log += 8;
                data_loc += size;
                restore_ctr = restore_ctr + 1;
            }
        }
    }
    pub fn restore_globals1() {
        unsafe {
            let mut restore_ctr: u16 = 0;
            loop {
                if *counter == restore_ctr {
                    break;
                }
                let mut combined: u32 = 0;
                for i in 0..4 {
                    combined
                        |= (ptr::read((transcation_log + i) as *const u32) << (i * 8));
                }
                let mut size: u8 = ptr::read(transcation_log as *const u8);
                for i in 0..size {
                    ptr::write(
                        (combined + i as u32) as *mut u8,
                        *((transcation_log + i as u32) as *const u8),
                    );
                }
                combined = combined + size as u32;
                restore_ctr += 1;
            }
        }
    }
    pub fn restore() -> bool {
        unsafe {
            let mut fram_start_address = 0x6000_9000;
            let packet_size = ptr::read_volatile(0x6000_9000 as *const u32);
            if packet_size == 0 {
                return false;
            }
            let mut offset: u32 = 0;
            fram_start_address += 4;
            if ptr::read_volatile(fram_start_address as *const u32) == 0xDEAD_BEEF {
                *counter = 0;
            }
            fram_start_address += 4;
            asm!("mov r0, {0}", in (reg) fram_start_address);
            asm!("movw r1, 0xfff8\n        movt r1, 0x02000");
            asm!("msr msp, r1");
            asm!("movw r3, 0xf1f1\n        movt r3, 0xf1f1");
            asm!(
                "1:\n            ldr r1, [r0, #4]\n            cmp r1, r3\n            beq 2f\n            push {{r1}}\n            adds r0, r0, #4\n            b 1b\n            2:"
            );
            asm!("adds r0, r0, #4");
            asm!("adds r0, r0, #4");
            asm!("LDR r1, [r0]");
            asm!("Push {{r1}}");
            asm!("adds r0, r0, #4");
            asm!("LDR r1, [r0]");
            asm!("adds r0, r0, #4");
            asm!("LDR r2, [r0]");
            asm!("adds r0, r0, #4");
            asm!("LDR r3, [r0]");
            asm!("adds r0, r0, #4");
            asm!("LDR r4, [r0]");
            asm!("adds r0, r0, #4");
            asm!("LDR r5, [r0]");
            asm!("adds r0, r0, #4");
            asm!("LDR r6, [r0]");
            asm!("adds r0, r0, #4");
            asm!("LDR r7, [r0]");
            asm!("adds r0, r0, #4");
            asm!("LDR r8, [r0]");
            asm!("adds r0, r0, #4");
            asm!("LDR r9, [r0]");
            asm!("adds r0, r0, #4");
            asm!("LDR r10, [r0]");
            asm!("adds r0, r0, #4");
            asm!("LDR r11, [r0]");
            asm!("adds r0, r0, #4");
            asm!("LDR r12, [r0]");
            asm!("adds r0, r0, #4");
            asm!("adds r0, r0, #4");
            asm!("LDR r14, [r0]");
            asm!("mov pc, r14");
        }
        return true;
    }
    pub fn delete_pg(page: u32) {
        unsafe {
            let mut dp = Peripherals::steal();
            let mut flash = &mut dp.FLASH;
            unlock(&mut flash);
            wait_ready(&flash);
            erase_page(&mut flash, page);
        }
    }
    pub fn delete_all_pg() {
        let start_address = 0x0803_0000;
        unsafe {
            let mut dp = Peripherals::steal();
            let mut flash = &mut dp.FLASH;
            for i in 0..25 {
                let page = start_address + i * 2 * 1024;
                unlock(&mut flash);
                wait_ready(&flash);
                erase_page(&mut flash, page);
            }
        }
    }
}
use checkpoint::{
    checkpoint, restore, delete_pg, delete_all_pg, transcation_log, execution_mode,
    counter, start_atomic, end_atomic, initialization,
};
use instrument::my_proc_macro;
#[link_section = ".fram_section"]
static mut x: u16 = 8;
#[link_section = ".fram_section"]
static mut y: u16 = 3;
#[link_section = ".fram_section"]
static mut z: u16 = 2;
#[link_section = ".fram_section"]
static mut t: u16 = 0xFF;
fn update() {
    let mut ya: u16 = 9;
    start_atomic();
    unsafe {
        save_variables(&x as *const _, core::mem::size_of_val(&x));
    }
    unsafe {
        x = 5;
    }
    save_variables(&ya as *const _, core::mem::size_of_val(&ya));
    ya = ya + 2;
    end_atomic();
}
#[no_mangle]
pub extern "C" fn main() -> ! {
    initialization();
    ::cortex_m_semihosting::export::hstdout_str("reseting the counter \n");
    restore();
    update();
    ::cortex_m_semihosting::export::hstdout_str("reseting the counter \n");
    checkpoint(false);
    unsafe {
        ptr::write_volatile((0x6000_0010) as *mut u16, 0xabcd as u16);
    }
    ::cortex_m_semihosting::export::hstdout_str("reseting the counter at the end \n");
    loop {}
}
