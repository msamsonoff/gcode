#[cfg(feature = "defmt")]
use defmt::Format;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum Sign {
    #[default]
    Positive,
    Negative,
}
