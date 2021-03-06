use heapless::{Vec, ArrayLength};
use core::str;
use cortex_m::asm::nop;
use f3::hal::stm32f30x::TIM6;

use cortex_m_semihosting::hio;
use core::fmt::Write;

pub struct SST39SF040<'a> {
    pub gpiod: &'a f3::hal::stm32f30x::GPIOD,
    pub gpioe: &'a f3::hal::stm32f30x::GPIOE,
}

pub enum Mode {
    Write = 0x5555,
    Read = 0x0
}

impl SST39SF040<'_> {
    pub unsafe fn sleep(ms: u16) {
        let tim6 = TIM6::ptr();
        // set arr to the khz we'd like to wait for
        (*tim6).arr.write(|w| w.arr().bits(ms));
        // enable the counter
        (*tim6).cr1.modify(|_, w| w.cen().set_bit());
        // wait for alarm to go off
        while !(*tim6).sr.read().uif().bit_is_set() {}
        // remove alarm
        (*tim6).sr.modify(|_, w| w.uif().clear_bit());
    }
    pub unsafe fn new<'a>(gpiod: &'a f3::hal::stm32f30x::GPIOD, gpioe: &'a f3::hal::stm32f30x::GPIOE) -> SST39SF040<'a> {
        let sst39 = SST39SF040 {
            gpiod,
            gpioe,
        };
        sst39.gpioe.moder.write(|w| w.bits(0x55555555));
        sst39.gpiod.moder.write(|w| w.bits(0x55555555));
        sst39.set_read_pin(true);
        sst39.set_write_pin(true);
        // exit software id mode
        // sst39.set_out_byte(0xf0, 0x0);
        SST39SF040::sleep(5000);
        sst39

    }
    pub unsafe fn configure_data_mode(&self, mode: Mode) {
        self.gpiod.moder.modify(|r, w| w.bits(r.bits() & 0xffff | ((mode as u32) << 16) as u32));
    }
    pub unsafe fn set_data_pins(&self, value: u8) {
        /* set output register (high bits, gpio 8 - 15) to the given value. in order to preserve
        the contents of the lower half of the address (gpio 0 - 7), i shift the given byte over to
        the higher half of the gpio and read the lower half of the output register, or-ing them
        together in order to preserve the lower address bus and set the higher half of the gpio. */
        self.gpiod.odr.modify(|r, w| w.bits(((r.bits() as u16 & 0xff) | ((value as u16) << 8)) as u32));
    }
    pub unsafe fn set_address_pins(&self, value: u16) {
        // set high bits
        self.gpioe.odr.modify(|r, w| w.bits(r.bits() & 0xff | (value & 0xff00) as u32));
        // set low bits, similar operation to the in set_data but this time we preserve high gpio
        self.gpiod.odr.modify(|r, w| w.bits(((r.bits() as u16 & 0xff00) | (value & 0xff)) as u32));
    }
    pub unsafe fn read_data_pins(&self) -> u8 {
        // return input register as byte
        self.configure_data_mode(Mode::Read);
        ((self.gpiod.idr.read().bits() & 0xff00) >> 8) as u8
    }
    pub unsafe fn set_write_pin(&self, state: bool) {
        self.gpioe.odr.modify(|r, w| w.bits(r.bits() | (((state as u8) << 6) as u32) ));
    }
    pub unsafe fn set_read_pin(&self, state: bool) {
        self.gpioe.odr.modify(|r, w| w.bits(r.bits() | (((state as u8) << 3) as u32) ));
    }
    pub unsafe fn read_byte(&self, address: u16) -> u8 {
        self.set_write_pin(true);
        self.set_read_pin(true);
        self.set_address_pins(address);
        writeln!(hio::hstdout().unwrap(), "pre_read: {:x}", self.read_data_pins());
        self.set_read_pin(false);
        SST39SF040::sleep(100);
        let byte = self.read_data_pins();
        writeln!(hio::hstdout().unwrap(), "post_read: {:x}", byte);
        self.set_read_pin(true);
        byte
    }
    pub unsafe fn set_out_byte(&self, byte: u8, address: u16) {
        self.configure_data_mode(Mode::Write);
        self.set_address_pins(address);
        self.set_data_pins(byte);
        self.set_write_pin(false);
        SST39SF040::sleep(1000);
        self.set_write_pin(true);
    }
    pub unsafe fn erase_chip(&self) {
        self.set_out_byte(0xaa, 0x5555);
        self.set_out_byte(0x55, 0x2aaa);
        self.set_out_byte(0x80, 0x5555);
        self.set_out_byte(0xaa, 0x5555);
        self.set_out_byte(0x55, 0x2aaa);
        self.set_out_byte(0x10, 0x5555);
        SST39SF040::sleep(2000);
    }
    pub unsafe fn write_byte(&self, byte: u8, address: u16) {
        self.set_out_byte(0xaa, 0x5555);
        writeln!(hio::hstdout().unwrap(), "wrote 0xaa to 0x5555");
        self.set_out_byte(0x55, 0x2aaa);
        writeln!(hio::hstdout().unwrap(), "wrote 0x55 to 0x2aaa");
        self.set_out_byte(0xa0, 0x5555);
        writeln!(hio::hstdout().unwrap(), "wrote 0xa0 to 0x5555");
        SST39SF040::sleep(1000);
        self.set_out_byte(byte, address);
        writeln!(hio::hstdout().unwrap(), "wrote {:x} to {:x}", self.gpiod.odr.read().bits(), address);
        self.set_data_pins(0);
    }
}