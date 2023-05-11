#![allow(unused)]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

use defmt::{println, Format};
use gcode::{BlockBuilder, BlockParser, Decimal};
use pac::gpio::vals::{Moder, Ospeedr, Ot, Pupdr};
use stm32_metapac as pac;

#[inline(never)]
pub fn bench_main(s: &str) -> ! {
    let pin = 0;

    unsafe {
        pac::DBGMCU.cr().modify(|w| {
            w.set_dbg_stop(true);
            w.set_dbg_standby(true);
        });

        pac::RCC.gpioenr().modify(|w| {
            w.set_gpioaen(true);
            w.set_gpioben(true);
            w.set_gpiocen(true);
            w.set_gpioden(true);
            w.set_gpiofen(true);
        });

        pac::GPIOA
            .pupdr()
            .modify(|w| w.set_pupdr(pin, Pupdr::PULLDOWN));
        pac::GPIOA.otyper().modify(|w| w.set_ot(pin, Ot::PUSHPULL));
        pac::GPIOA
            .ospeedr()
            .modify(|w| w.set_ospeedr(pin, Ospeedr::HIGHSPEED));
        pac::GPIOA
            .moder()
            .modify(|w| w.set_moder(pin, Moder::OUTPUT));
    };

    let mut val = true;
    let mut parser = BlockParser::<i32>::default();
    let mut builder = NoOpBlockBuilder;

    loop {
        val = !val;
        unsafe {
            if val {
                pac::GPIOA.bsrr().write(|w| w.set_bs(pin, true));
            } else {
                pac::GPIOA.bsrr().write(|w| w.set_br(pin, true));
            }
        };
        if let Err(e) = parser.try_feed_str(s, &mut builder) {
            println!("{:?}", e);
            break;
        };
    }

    loop {
        cortex_m::asm::nop();
    }
}

#[derive(Debug, Format)]
struct NoOpBlockBuilder;

impl BlockBuilder for NoOpBlockBuilder {
    type Error = ();
    type Significand = i32;

    #[inline(never)]
    fn program_start(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline(never)]
    fn sequence_number(
        &mut self,
        _alignment: bool,
        _number: Decimal<Self::Significand>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline(never)]
    fn g_code(&mut self, _number: Decimal<Self::Significand>) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline(never)]
    fn m_code(&mut self, _number: Decimal<Self::Significand>) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline(never)]
    fn data(
        &mut self,
        _address: char,
        _index: Option<Self::Significand>,
        _number: Decimal<Self::Significand>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline(never)]
    fn end_block(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
