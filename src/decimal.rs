use core::fmt::Debug;

use crate::sign::Sign;
use crate::significand::{Significand, SignificandExt};

#[cfg(feature = "defmt")]
use defmt::Format;

/// A decimal number in the format `significand × 10`<sup>`-negative_exponent`</sup>.
///
/// The significand is sometimes called the "mantissa".
///
/// The sign of the [`Decimal`] is encoded in the significand. If the underlying implementation of
/// [`Significand`] supports negative numbers the significand will be negative whenever the
/// [`Decimal`] is less than zero. There is, however, no provision for distinguishing between
/// positive and negative zero.
///
/// The negative exponent is the number of the significand's decimal digits that appear right of
/// the decimal point. It is stored an unsigned positive number.
///
/// Decimal numbers processed by this crate correspond to coordinates, distances, and feed rates
/// for CNC machines with particular ("reasonable") physical characteristics. Negative exponents
/// are expected to be small, typically single digits. For example, if the base unit is meters, a
/// distance of `25μm` would be stored as a significand of `25` and a negative exponent of `6`.
///
/// `25μm = 0.000025m = 25×10`<sup>`-6`</sup>`m`.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub struct Decimal<S>
where
    S: Significand,
{
    significand: S,
    negative_exponent: u32,
}

impl<S> Decimal<S>
where
    S: Significand,
{
    /// Creates a new [`Decimal`] with the specified significand and negative exponent.
    pub const fn new(significand: S, negative_exponent: u32) -> Self {
        Self {
            significand,
            negative_exponent,
        }
    }

    /// Returns the significand of the [`Decimal`] number.
    pub const fn significand(&self) -> S {
        self.significand
    }

    /// Returns the negative exponent of the [`Decimal`] number.
    pub const fn negative_exponent(&self) -> u32 {
        self.negative_exponent
    }
}

impl<S> Eq for Decimal<S> where S: Eq + Significand {}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub struct DecimalParser<S>
where
    S: Significand,
{
    state: State,
    sign: Sign,
    significand: S,
    negative_exponent: u32,
    trailing_zeros_plus_one: u32,
}

impl<S> Default for DecimalParser<S>
where
    S: Significand,
{
    fn default() -> Self {
        Self {
            state: State::Start,
            sign: Sign::default(),
            significand: S::default(),
            negative_exponent: 0,
            trailing_zeros_plus_one: 1,
        }
    }
}

impl<S> DecimalParser<S>
where
    S: Significand,
{
    pub fn try_feed(&mut self, c: char) -> Result<(), Error> {
        match (&self.state, c) {
            (State::Start, '+') => {
                self.state = State::Sign;
                Ok(())
            }
            (State::Start, '-') => {
                self.state = State::Sign;
                self.sign = Sign::Negative;
                Ok(())
            }
            (State::Start | State::Sign, '.') => {
                self.state = State::LeadingDecimal;
                Ok(())
            }
            (State::Integer, '.') => {
                self.state = State::Fraction;
                Ok(())
            }
            (State::Start | State::Sign | State::Integer, '0'..='9') => {
                if '0' == c && self.significand.is_zero() {
                    self.state = State::Integer;
                    Ok(())
                } else {
                    let significand = self.significand.checked_append_digit(1, c, self.sign);
                    if let Some(significand) = significand {
                        self.state = State::Integer;
                        self.significand = significand;
                        Ok(())
                    } else {
                        Err(Error::Capacity)
                    }
                }
            }
            (State::LeadingDecimal | State::Fraction, '0') => {
                if let Some(trailing_zeroes) = self.trailing_zeros_plus_one.checked_add(1) {
                    self.state = State::Fraction;
                    self.trailing_zeros_plus_one = trailing_zeroes;
                    Ok(())
                } else {
                    Err(Error::Capacity)
                }
            }
            (State::LeadingDecimal | State::Fraction, '1'..='9') => {
                let significand = self.significand.checked_append_digit(
                    self.trailing_zeros_plus_one,
                    c,
                    self.sign,
                );
                let negative_exponent = self
                    .negative_exponent
                    .checked_add(self.trailing_zeros_plus_one);
                if let (Some(significand), Some(negative_exponent)) =
                    (significand, negative_exponent)
                {
                    self.state = State::Fraction;
                    self.significand = significand;
                    self.negative_exponent = negative_exponent;
                    self.trailing_zeros_plus_one = 1;
                    Ok(())
                } else {
                    Err(Error::Capacity)
                }
            }
            _ => Err(Error::InvalidCharacter),
        }
    }

    pub const fn try_end(&self) -> Result<Decimal<S>, Error> {
        if matches!(self.state, State::Integer | State::Fraction) {
            let number = Decimal {
                significand: self.significand,
                negative_exponent: self.negative_exponent,
            };
            Ok(number)
        } else {
            Err(Error::Incomplete)
        }
    }

    #[cfg(test)]
    fn try_feed_str<T>(&mut self, s: T) -> Result<(), Error>
    where
        T: AsRef<str>,
    {
        let s = s.as_ref();
        for c in s.chars() {
            self.try_feed(c)?;
        }
        Ok(())
    }

    #[cfg(test)]
    fn try_feed_str_end<T>(mut self, s: T) -> Result<Decimal<S>, Error>
    where
        T: AsRef<str>,
    {
        self.try_feed_str(s)?;
        self.try_end()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum Error {
    Capacity,
    Incomplete,
    InvalidCharacter,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(Format))]
enum State {
    Start,
    Sign,
    LeadingDecimal,
    Integer,
    Fraction,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn significand() {
        let decimal = Decimal::new(946_178_989, 5);
        let significand = decimal.significand();
        assert_eq!(significand, 946_178_989);
    }

    #[test]
    fn negative_exponent() {
        let decimal = Decimal::new(679_503_158, 4);
        let negative_exponent = decimal.negative_exponent();
        assert_eq!(negative_exponent, 4);
    }

    #[test]
    fn capacity_integer() {
        let parser: DecimalParser<i32> = DecimalParser::default();
        let result = parser.try_feed_str_end("2147483648");
        assert_eq!(result, Err(Error::Capacity));
    }

    #[test]
    fn capacity_fraction() {
        let parser: DecimalParser<i32> = DecimalParser::default();
        let result = parser.try_feed_str_end("21474.83648");
        assert_eq!(result, Err(Error::Capacity));
    }

    #[test]
    fn capacity_trailing_zeros() {
        let parser = DecimalParser::<i32> {
            state: State::Fraction,
            sign: Sign::default(),
            significand: Default::default(),
            negative_exponent: 0,
            trailing_zeros_plus_one: u32::MAX,
        };
        let result = parser.try_feed_str_end("0");
        assert_eq!(result, Err(Error::Capacity));
    }

    #[test]
    fn invalid_character() {
        let parser: DecimalParser<i32> = DecimalParser::default();
        let result = parser.try_feed_str_end("4904-3957");
        assert_eq!(result, Err(Error::InvalidCharacter));
    }

    //
    // unsigned
    //

    #[test]
    fn unsigned_incomplete() {
        let parser: DecimalParser<i32> = DecimalParser::default();
        let result = parser.try_feed_str_end("");
        assert_eq!(result, Err(Error::Incomplete));
    }

    #[test]
    fn unsigned_zero() {
        let parser: DecimalParser<i32> = DecimalParser::default();
        let result = parser.try_feed_str_end("0");
        assert_eq!(result, Ok(Decimal::new(0, 0)));
    }

    #[test]
    fn unsigned_zero_point() {
        let parser: DecimalParser<i32> = DecimalParser::default();
        let result = parser.try_feed_str_end("0.");
        assert_eq!(result, Ok(Decimal::new(0, 0)));
    }

    #[test]
    fn unsigned_point_zero() {
        let parser: DecimalParser<i32> = DecimalParser::default();
        let result = parser.try_feed_str_end(".0");
        assert_eq!(result, Ok(Decimal::new(0, 0)));
    }

    #[test]
    fn unsigned_zero_point_zero() {
        let parser: DecimalParser<i32> = DecimalParser::default();
        let result = parser.try_feed_str_end("0.0");
        assert_eq!(result, Ok(Decimal::new(0, 0)));
    }

    #[test]
    fn unsigned_one() {
        let parser: DecimalParser<i32> = DecimalParser::default();
        let result = parser.try_feed_str_end("2");
        assert_eq!(result, Ok(Decimal::new(2, 0)));
    }

    #[test]
    fn unsigned_two() {
        let parser: DecimalParser<i32> = DecimalParser::default();
        let result = parser.try_feed_str_end("8.5");
        assert_eq!(result, Ok(Decimal::new(85, 1)));
    }

    #[test]
    fn unsigned_many_point() {
        let parser: DecimalParser<i32> = DecimalParser::default();
        let result = parser.try_feed_str_end("514159813.");
        assert_eq!(result, Ok(Decimal::new(514_159_813, 0)));
    }

    #[test]
    fn unsigned_many_point_many() {
        let parser: DecimalParser<i32> = DecimalParser::default();
        let result = parser.try_feed_str_end("148.452384");
        assert_eq!(result, Ok(Decimal::new(148_452_384, 6)));
    }

    #[test]
    fn unsigned_point_many() {
        let parser: DecimalParser<i32> = DecimalParser::default();
        let result = parser.try_feed_str_end(".799001184");
        assert_eq!(result, Ok(Decimal::new(799_001_184, 9)));
    }

    //
    // positive
    //

    #[test]
    fn positive_incomplete() {
        let parser: DecimalParser<i32> = DecimalParser::default();
        let result = parser.try_feed_str_end("+");
        assert_eq!(result, Err(Error::Incomplete));
    }

    #[test]
    fn positive_zero() {
        let parser: DecimalParser<i32> = DecimalParser::default();
        let result = parser.try_feed_str_end("+0");
        assert_eq!(result, Ok(Decimal::new(0, 0)));
    }

    #[test]
    fn positive_zero_point() {
        let parser: DecimalParser<i32> = DecimalParser::default();
        let result = parser.try_feed_str_end("+0.");
        assert_eq!(result, Ok(Decimal::new(0, 0)));
    }

    #[test]
    fn positive_point_zero() {
        let parser: DecimalParser<i32> = DecimalParser::default();
        let result = parser.try_feed_str_end("+.0");
        assert_eq!(result, Ok(Decimal::new(0, 0)));
    }

    #[test]
    fn positive_zero_point_zero() {
        let parser: DecimalParser<i32> = DecimalParser::default();
        let result = parser.try_feed_str_end("+0.0");
        assert_eq!(result, Ok(Decimal::new(0, 0)));
    }

    //
    // negative
    //

    #[test]
    fn negative_incomplete() {
        let parser: DecimalParser<i32> = DecimalParser::default();
        let result = parser.try_feed_str_end("-");
        assert_eq!(result, Err(Error::Incomplete));
    }

    #[test]
    fn negative_zero() {
        let parser: DecimalParser<i32> = DecimalParser::default();
        let result = parser.try_feed_str_end("-0");
        assert_eq!(result, Ok(Decimal::new(0, 0)));
    }

    #[test]
    fn negative_zero_point() {
        let parser: DecimalParser<i32> = DecimalParser::default();
        let result = parser.try_feed_str_end("-0.");
        assert_eq!(result, Ok(Decimal::new(0, 0)));
    }

    #[test]
    fn negative_point_zero() {
        let parser: DecimalParser<i32> = DecimalParser::default();
        let result = parser.try_feed_str_end("-.0");
        assert_eq!(result, Ok(Decimal::new(0, 0)));
    }

    #[test]
    fn negative_zero_point_zero() {
        let parser: DecimalParser<i32> = DecimalParser::default();
        let result = parser.try_feed_str_end("-0.0");
        assert_eq!(result, Ok(Decimal::new(0, 0)));
    }
}
