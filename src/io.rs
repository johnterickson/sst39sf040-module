use f3::hal::stm32f30x::usart1;
use heapless::{Vec, ArrayLength};
use core::str;

pub struct SST39SF040 {
    data_pins: [u8; 8]
}

