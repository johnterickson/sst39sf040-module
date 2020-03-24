use f3::hal::stm32f30x::usart1;
use heapless::{Vec, ArrayLength};
use core::str;

pub struct Serial {
    uart: &'static mut usart1::RegisterBlock,
}

impl Serial {
    pub fn new(uart: &'static mut usart1::RegisterBlock) -> Serial {
        Serial {
            uart,
        }
    }

    pub fn send(&self, value: &str) {
        for character in value.bytes() {
            while self.uart.isr.read().txe().bit_is_clear() {}
            self.uart.tdr.write(|w| w.tdr().bits(u16::from(character)));
        }
    }
    pub fn recv<T: ArrayLength<u8>>(&self, output_buffer: &mut Vec<u8, T>, amount: usize) {
        for _ in 0..amount {
            while self.uart.isr.read().rxne().bit_is_clear() {}
            let byte = self.uart.rdr.read().rdr().bits() as u8;
            if output_buffer.push(byte).is_err() {
                panic!("buffer too small for recv");
            }
        }
    }
}

