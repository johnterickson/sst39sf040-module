use heapless::{Vec, ArrayLength};
use core::str;
use cortex_m::asm::nop;
use f3;

pub struct SST39SF040<'a> {
    pub gpiod: &'a f3::hal::stm32f30x::GPIOD,
    pub gpioe: &'a f3::hal::stm32f30x::GPIOE
}

pub enum Mode {
    Write = 0x55555555,
    Read = 0x0
}

impl SST39SF040<'_> {
    pub unsafe fn new<'a>(gpiod: &'a f3::hal::stm32f30x::GPIOD, gpioe: &'a f3::hal::stm32f30x::GPIOE) -> SST39SF040<'a> {
        let sst39 = SST39SF040 {
            gpiod,
            gpioe,
        };
        sst39.configure_data_mode(Mode::Write);
        sst39.gpioe.moder.write(|w| w.bits(Mode::Write as u32));
        sst39.set_read_pin(true);
        sst39.set_write_pin(true);
        sst39

    }
    pub unsafe fn configure_data_mode(&self, mode: Mode) {
        self.gpiod.moder.write(|w| w.bits(mode as u32));
    }
    pub unsafe fn set_data_pins(&self, value: u8) {
        /* set output register (high bits, gpio 8 - 15) to the given value. in order to preserve
        the contents of the lower half of the address (gpio 0 - 7), i shift the given byte over to
        the higher half of the gpio and read the lower half of the output register, or-ing them
        together in order to preserve the lower address bus and set the higher half of the gpio. */
        self.gpiod.odr.modify(|r, w| w.bits(((r.bits() as u16 & 0xff) | (((value as u16) << 8) as u16)) as u32));
    }
    pub unsafe fn set_address_pins(&self, value: u16) {
        // set high bits
        self.gpioe.odr.modify(|r, w| w.bits(r.bits() & 0xff | (value & 0xff00) as u32));
        // set low bits, similar operation to the in set_data but this time we preserve high gpio
        self.gpiod.odr.modify(|r, w| w.bits(((r.bits() as u16 & 0xff00) | (value & 0xff)) as u32));
    }
    pub fn read_data_pins(&self) -> u8 {
        // return input register as byte
        ((self.gpiod.idr.read().bits() & 0xff00) >> 8) as u8
    }
    pub unsafe fn set_write_pin(&self, state: bool) {
        self.gpioe.odr.modify(|r, w| w.bits(r.bits() | (((state as u8) << 6) as u32) ));
    }
    pub unsafe fn set_read_pin(&self, state: bool) {
        self.gpioe.odr.modify(|r, w| w.bits(r.bits() | (((state as u8) << 3) as u32) ));
    }
    pub unsafe fn read_byte(&self, address: u16) -> u8 {
        self.set_address_pins(address);
        self.set_write_pin(true);
        self.set_read_pin(false);
        self.configure_data_mode(Mode::Read);
        // TODO: fix busy loop and replace with time delay one
        // roughly 720 clock cycles at least, so ~1ms
        for _ in 0..59000 {
            nop();
        }
        self.read_data_pins()
    }
}