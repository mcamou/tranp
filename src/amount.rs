use std::{
    cmp::min,
    fmt::Display,
    ops::{Add, Sub},
};

use crate::types::Error;

const DECIMALS: usize = 4;

// I tried using the primitive_fixed_point_decimal and the fixed crates, but they both had problems with
// serde+csv. This is a very-poor-man's version of a fixed decimal.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Hash)]
pub struct Amount(u64);

impl Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = self.into();
        write!(f, "{}", s)
    }
}

impl From<u64> for Amount {
    fn from(value: u64) -> Self {
        Amount(value)
    }
}

impl From<Amount> for u64 {
    fn from(val: Amount) -> Self {
        val.0
    }
}

impl TryFrom<String> for Amount {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let idx = value.find(".").unwrap_or(value.len());
        let (int_part, mut frac_part) = value.split_at(idx);

        frac_part = if frac_part.is_empty() {
            "0000"
        } else {
            &frac_part[1..min(DECIMALS + 1, frac_part.len())]
        };

        let s = format!("{}{:0<width$}", int_part, frac_part, width = DECIMALS);
        s.parse()
            .map_err(|e| {
                Error::Input(format!(
                    "Error while parsing amount {} ({}): {}",
                    value, s, e
                ))
            })
            .map(Amount)
    }
}

impl From<&Amount> for String {
    fn from(value: &Amount) -> Self {
        let frac_part = format!("{:0<width$}", value.0 % 10000, width = DECIMALS).as_str()
            [0..DECIMALS]
            .to_owned();
        format!("{}.{}", value.0 / 10000, frac_part)
    }
}

impl Add for Amount {
    type Output = Amount;

    fn add(self, rhs: Self) -> Self::Output {
        Amount(self.0 + rhs.0)
    }
}

impl Sub for Amount {
    type Output = Amount;

    fn sub(self, rhs: Self) -> Self::Output {
        Amount(self.0 - rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marshal_amount() {
        let sut = Amount(12345678);
        let actual: String = (&sut).into();

        assert_eq!(actual, "1234.5678");
    }

    #[test]
    fn test_unmarshal_amount() {
        let sut = "1234.5678".to_owned();
        let actual: Amount = sut.try_into().expect("Error unmarshalling 1234.5678");

        assert_eq!(actual, Amount(12345678));

        let sut = "1234".to_owned();
        let actual: Amount = sut.try_into().expect("Error unmarshalling 1234");

        assert_eq!(actual, Amount(12340000));

        let sut = "1234.56".to_owned();
        let actual: Amount = sut.try_into().expect("Error unmarshalling 1234.56");

        assert_eq!(actual, Amount(12345600));

        let sut = "1234.5678901".to_owned();
        let actual: Amount = sut.try_into().expect("Error unmarshalling 1234.5678901");

        assert_eq!(actual, Amount(12345678));
    }

    #[test]
    fn test_amount_math() {
        let actual = Amount(123400) + Amount(234500);
        assert_eq!(actual, Amount(357900));

        let actual = Amount(234500) - Amount(123400);
        assert_eq!(actual, Amount(111100));
    }
}
