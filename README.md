This is an experimental [G-code](https://en.wikipedia.org/wiki/G-code) parser for resource-constrained embedded devices.
It operates character-by-character to avoid buffering.
It works in a `no_std` environment and does not require an `alloc` crate.
This imposes some limitations on the dialect of G-code it supports.

The reference target is [STM32G031K8](https://www.st.com/en/microcontrollers-microprocessors/stm32g031k8.html).
This device is an ARM Cortex-M0 with 8 KiB of SRAM and 64 KiB of flash program memory.

## Basic Usage

        use gcode::BlockParser;

        let mut parser = BlockParser::<i32>::default();
        let mut builder = ...
        loop {
            let c = read_next_char()?;
            parser.try_feed_char(c, &mut builder)?;
        };

## G-code Language

This parser implements approximately the ISO 6983-1:2009 dialect of G-code.
It does not support parameters, expressions, nor control structures (conditionals and loops.)
It is only a syntax parser and it does not perform any semantic validation.
For example, a word can appear multiple times in one block, and the parser does not enforce modal groups.

Whitespace is defined as space characters (`' '`), tab characters (`\t`), and carriage-return characters (`\r`.)
Whitespace can appear anywhere and is discarded.
For example, these blocks are all equivalent:

        G01X123Y456
        G01 X123 Y456
        G 0 1 X 1 2 3 Y 4 5 6

Blocks are terminated by line feed characters (`\n`.)
Line feeds are not considered whitespace.

Comments start with the `(` character and end with the `)` character.
They may span lines.
They may not be nested.
The content of the comment is discarded.

        G01 (this is a
        comment) X1 Y2

The parser ignores case.
These blocks are equivalent, although the parser will preserve case when calling the `BlockBuilder`.

        g01 X1 y2
        G01 x1 Y2

The parser supports axis and general indexing.
Indexes must be unsigned integers.
Decimals are not allowed here even if the fractional part is zero.

        G01 X5=432

Non-index numbers may have an optional sign.

        G01 X-2.34 Y1=+56.7 Z8.9

The parser normalizes decimal numbers to remove insignificant leading and trailing zeros.
This applies to equally to all numbers: indexes, coordinates, dimensions, feed rates, and G and M codes.
For example, these G codes are all equivalent:

        G92.10
        G92.1
        G092.1
        G092.10

A block is skipped if it starts with a `/` character.
The parser discards everything through the next line feed character (`\n`.)

        /G01 X1 Y2

A block starts with an optional sequence number.
A sequence number starts with the character `N` or a `:` (the "alignment" character.)

        N0001 G01 X1 Y2
        :0002 G01 X3 Y4

The parser also supports the program start character `%`.
It may appear on a line by itself or before any block content (i.e., before a block skip, sequence number, or G-code words.)
Since this is only a syntax parser, the program start character is optional, and it can also be used multiple times in a single program.
(Some G-code programs re-use this character to end a program.)

        %
        G00 X1 Y2 Z3
        G01 X4 Y5 Z6

## Feature Flags

`defmt` - Enable support for the [`defmt`](https://github.com/knurling-rs/defmt) crate.

`mul10_by_shl` - Use binary shift-left operations for checked multiplication by ten.
This is a significant performance increase on some targets.

## Minimum Supported Rust Version

This crate requires several features that are only available on nightly at this time.
