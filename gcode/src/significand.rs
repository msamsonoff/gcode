use crate::sign::Sign;

// It would be nice to conditionally include defmt::Format here. See RFC 3399 for cfg-attributes on
// where clauses:
// https://github.com/rust-lang/rfcs/pull/3399

/// The trait for numeric types that can be used to store the significand of a [`Decimal`] number.
///
/// [`Decimal`]: crate::Decimal
pub trait Significand
where
    Self: Clone + Copy + Default,
{
    /// Returns `true` if the number is zero.
    fn is_zero(&self) -> bool;

    /// Checked multiplication by a power of ten. Computes `self × 10`<sup>`exp`</sup>, returning
    /// `None` if overflow occurred.
    fn checked_shl10(self, exp: u32) -> Option<Self>;

    /// Checked addition with an unsigned integer. Computes `self + rhs`, returning `None` if
    /// overflow occurred.
    fn checked_add_unsigned(self, rhs: u32) -> Option<Self>;

    /// Checked subtraction with an unsigned integer. Computes `self - rhs`, returning `None` if
    /// overflow occurred.
    fn checked_sub_unsigned(self, rhs: u32) -> Option<Self>;
}

impl Significand for i32 {
    fn is_zero(&self) -> bool {
        0 == *self
    }

    fn checked_shl10(self, exp: u32) -> Option<Self> {
        cfg_if::cfg_if! {
            if #[cfg(not(feature = "mul10_by_shl"))] {

                10_i32.checked_pow(exp)?.checked_mul(self)

            } else {

                // This is SIGNIFICANTLY faster on ARM Cortex-M0. The overall performance of the
                // G-code parser is around 5% to 40% faster, depending on the density of numeric
                // data in the file.
                //
                // ARM does not set the V (overflow) flag when the MUL instruction overflows (e.g.,
                // see the [ARMv6-M Architecture Reference Manual][1] A.6.7.44 MUL, which reads
                // "APSR.V unchanged".) The compiler implements `checked_mul` by using the
                // `__aeabi_lmul` function to perform a widened 64-bit multiplication. This has
                // both the overhead of the function call itself (it does not get inlined, even
                // with LTO) and dozens of arithmetic instructions, most of which end up computing
                // bits that are ultimately thrown away. It is expensive.
                //
                // [1]: https://developer.arm.com/documentation/ddi0419/latest/
                //
                // y = x * 10
                // y = x * (8 + 2)
                // y = (x * 8) + (x * 2)
                // y = (x << 3) + (x << 1)

                let mut acc = self;
                let mut exp = exp;
                while exp > 0 {
                    let x8 = acc.checked_shl(3)?;
                    let x2 = acc.checked_shl(1)?;
                    acc = x8.checked_add(x2)?;
                    exp -= 1;
                };
                Some(acc)

            }
        }
    }

    fn checked_add_unsigned(self, rhs: u32) -> Option<Self> {
        <Self>::checked_add_unsigned(self, rhs)
    }

    fn checked_sub_unsigned(self, rhs: u32) -> Option<Self> {
        <Self>::checked_sub_unsigned(self, rhs)
    }
}

pub trait SignificandExt
where
    Self: Sized,
{
    fn checked_append_digit(&self, exp: u32, digit: char, sign: Sign) -> Option<Self>;
}

impl<S> SignificandExt for S
where
    S: Significand,
{
    fn checked_append_digit(&self, exp: u32, digit: char, sign: Sign) -> Option<Self> {
        // NOTE: This was a constant until #[feature(const_convert)] was removed.
        // https://github.com/rust-lang/rust/issues/88674
        // https://github.com/rust-lang/rust/issues/110395
        let zero: u32 = '0'.into();

        let digit = u32::try_from(digit).ok()?.checked_sub(zero)?;
        if (0..=9).contains(&digit) {
            let significand = self.checked_shl10(exp)?;
            match sign {
                Sign::Positive => significand.checked_add_unsigned(digit),
                Sign::Negative => significand.checked_sub_unsigned(digit),
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //
    // Significand for i32
    //

    #[test]
    fn i32_is_zero_true() {
        assert!(0_i32.is_zero());
    }

    #[test]
    fn i32_is_zero_false() {
        assert!(!9_i32.is_zero());
    }

    #[test]
    fn i32_checked_shl10() {
        assert_eq!(Significand::checked_shl10(6_i32, 4), Some(60000));
    }

    #[test]
    fn i32_checked_add_unsigned() {
        assert_eq!(Significand::checked_add_unsigned(7_i32, 8), Some(15));
    }

    #[test]
    fn i32_checked_add_unsigned_none() {
        assert_eq!(Significand::checked_add_unsigned(i32::MAX, 1), None);
    }

    #[test]
    fn i32_checked_sub_unsigned() {
        assert_eq!(Significand::checked_sub_unsigned(-9_i32, 3), Some(-12));
    }

    #[test]
    fn i32_checked_sub_unsigned_none() {
        assert_eq!(Significand::checked_sub_unsigned(i32::MIN, 1), None);
    }

    //
    // SignificandExt for i32
    //

    #[test]
    fn checked_append_digit_none() {
        let left = 3.checked_append_digit(1, 'a', Sign::Positive);
        assert_eq!(left, None);
    }

    #[test]
    fn checked_append_digit_positive() {
        let left = 8.checked_append_digit(2, '9', Sign::Positive);
        assert_eq!(left, Some(809));
    }

    #[test]
    fn checked_append_digit_negative() {
        let left = (-2).checked_append_digit(3, '5', Sign::Negative);
        assert_eq!(left, Some(-2005));
    }
}
