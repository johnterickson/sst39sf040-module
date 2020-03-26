#![no_main]
#![no_std]
/*
pin mapping:
    DATA_PINS

*/
extern crate cortex_m_semihosting;
extern crate panic_semihosting;

use cortex_m_rt::entry;
use heapless::{
    consts::{
        U8,
        U128
    },
    Vec
};
use core::str::from_utf8;


use f3::{
    hal::{
        prelude::*,
        stm32f30x::{self, USART1, usart1},
    },
    led::Leds,
};
pub use cortex_m::{asm::bkpt, iprint, iprintln, peripheral::ITM};

mod serial;
mod io;

use cortex_m_semihosting::hio;
use core::fmt::Write;
use core::borrow::Borrow;
use numtoa::NumToA;

fn query_ok(conn: &serial::Serial) {
    let mut recv_buffer: Vec<u8, U8> = Vec::new();
    conn.recv(&mut recv_buffer, 8);
    if from_utf8(&recv_buffer).unwrap() == "UART_OK?" {
        conn.send("UART_OK!");
    } else {
        conn.send("UNRECOG?");
    }
}

#[entry]
unsafe fn main() -> ! {
    let mut stdout = hio::hstdout().unwrap();

    // fetch peripherals singleton
    let stm32f3_peripherals = stm32f30x::Peripherals::take().unwrap();
    // set bit in ahbenr for power to gpiod/e, so our gpiod/e pins can have power
    // doing it up here at the beginning since RCC is borrowed at the constraint
    stm32f3_peripherals.RCC.ahbenr.modify(|_, w| w.iopden().set_bit().iopeen().set_bit());
    // get flash from peripherals
    let mut flash = stm32f3_peripherals.FLASH.constrain();
    let mut rcc = stm32f3_peripherals.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    // serial things
    let mut gpioa = stm32f3_peripherals.GPIOA.split(&mut rcc.ahb); // fetch pinout

    // set both tx and rx as alt func 7, by modifying mode register and the alt func register (high)
    let tx = gpioa.pa9.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    let rx = gpioa.pa10.into_af7(&mut gpioa.moder, &mut gpioa.afrh);

    // use hal serial to configure
    f3::hal::serial::Serial::usart1(stm32f3_peripherals.USART1, (tx, rx), 115_200.bps(), clocks, &mut rcc.apb2);
    let usart: &mut usart1::RegisterBlock = &mut *(USART1::ptr() as *mut _);
    let conn = serial::Serial::new(usart); // construct my singleton from registers of usart (registerblock)

    // write 1 to data register in the last bit
    let mut sst39 = io::SST39SF040::new(&stm32f3_peripherals.GPIOD, &stm32f3_peripherals.GPIOE);
    sst39.configure_mode(io::Mode::Write);
    sst39.set_data(0xff);
    sst39.set_address(0xffff);
    let x = sst39.gpioe.moder.read().bits();
    let mut buf = [0u8; 32];
    x.numtoa(16, &mut buf);
    writeln!(stdout, "{}", from_utf8(&buf).unwrap());
    loop {
        query_ok(&conn);
        loop {
            let mut command: Vec<u8, U8> = Vec::new();
            conn.recv(&mut command, 8);
        }
    }
}