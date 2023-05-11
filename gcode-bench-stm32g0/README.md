The benchmark hardware is a [NUCLEO-G031K8](https://www.st.com/en/evaluation-tools/nucleo-g031k8.html) development board.
It uses a [STM32G031K8](https://www.st.com/en/microcontrollers-microprocessors/stm32g031k8.html) processor.
It runs at 16 MHz after reset.
(The benchmark does not reconfigure the core clock.)

This requires [`probe-run`](https://github.com/knurling-rs/probe-run).

        $ cargo install probe-run

There are two benchmark G-code files.
Each file is approximately 16 KiB in length.

*   `max.gcode` - This file is 16,822 bytes long and mostly uses 10 digit numbers (`u32::MAX`.)

*   `min.gcode` - This file is 16,458 bytes long and mostly uses 1 digit numbers (`0`.)

There is an associated binary for each file.

        $ cargo run --release --bin bench_min --features mul10_by_shl
        $ cargo run --release --bin bench_max --features mul10_by_shl

The benchmark program sets the `A0` pin low, parses a G-code file, sets the pin high, and parses the file again.
It repeats this indefinitely and produces a square wave on pin `A0`.
The time to parse the file is 1/2 the period of the square wave.
We can calculate the number of clock cycles required to process each byte based on this time, the length of the file, and the core clock frequency.

## Baseline Results

*   `min.gcode` with `mul10_by_shl` - 169,582.69 byte/s, 94.35 cycles/byte

    ![benchmark results for min.gcode](./baseline-min.png)

*   `max.gcode` with `mul10_by_shl` - 122,386.32 bytes/s, 130.73 cycles/byte

    ![benchmark results for max.gcode](./baseline-max.png)
