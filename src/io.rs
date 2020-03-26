use heapless::{Vec, ArrayLength};
use core::str;
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
        // set addresses as outputs
        sst39.gpioe.moder.write(|w| w.bits(Mode::Write as u32));
        sst39

    }
    pub unsafe fn configure_mode(&self, mode: Mode) {
        self.gpiod.moder.write(|w| w.bits(mode as u32))
    }
    pub unsafe fn set_data(&self, value: u8) {
        // set output register (high bits, gpio 8 - 15) to the given value
        self.gpiod.odr.write(|w| w.bits((value as u32) << 8));
    }
    pub unsafe fn set_address(&self, value: u16) {
        // set
        self.gpioe.odr.write(|w| w.bits(value as u32));
    }
    pub fn read_data(&self) -> u8 {
        // return input register as byte
        self.gpiod.idr.read().bits() as u8
    }
}