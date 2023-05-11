#![no_main]
#![no_std]

#[cortex_m_rt::entry]
fn main() -> ! {
    let s = include_str!("../min.gcode");
    gcode_bench_stm32g0::bench_main(s);
}
