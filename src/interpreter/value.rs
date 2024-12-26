use anyhow::Result;
use std::{
    fmt::Display,
    ops::{Add, Neg, Sub},
};

use super::token::FullUnit;

pub struct Value {
    value: f64,
    unit: Option<FullUnit>,
}

impl Value {
    pub fn new(value: f64, unit: Option<FullUnit>) -> Self {
        Self { value, unit }
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn unit(&self) -> Option<FullUnit> {
        self.unit
    }

    pub fn convert_to(self, unit: FullUnit) -> Self {
        if self.unit == Some(unit) {
            return self;
        }

        let our_unit = match self.unit {
            Some(u) => u,
            None => return Self::new(self.value, Some(unit)),
        };

        let multiplier = u64::from(our_unit) as f64 / u64::from(unit) as f64;
        let value = self.value * multiplier;

        Self::new(value, Some(unit))
    }

    /// Returns the result of multiplying `self` by `rhs`, but only if one or both of the two values are
    /// unitless.
    pub fn try_mul(&self, rhs: Self) -> Result<Self> {
        if self.unit.is_some() && rhs.unit.is_some() {
            anyhow::bail!("Cannot multiply two values with units");
        }

        let unit = self.unit.or(rhs.unit);
        Ok(Self::new(self.value * rhs.value, unit))
    }

    /// Returns the result of dividing `self` by `rhs`, but only if the `rhs` or both of the two values are
    /// unitless.
    pub fn try_div(&self, rhs: Self) -> Result<Self> {
        if rhs.unit.is_some() {
            anyhow::bail!("Cannot divide by a value with a unit");
        }

        Ok(Self::new(self.value / rhs.value, self.unit))
    }
}

impl Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.unit == rhs.unit {
            return Self::new(self.value.sub(rhs.value), self.unit);
        }

        let (left, right) = if let (Some(left), Some(right)) = (self.unit, rhs.unit) {
            (left, right)
        } else {
            let unit = self.unit.or(rhs.unit);
            return Self::new(self.value.sub(rhs.value), unit);
        };

        let precise = std::cmp::min(left, right);
        let value = self
            .convert_to(precise)
            .value
            .sub(rhs.convert_to(precise).value);

        Self::new(value, Some(precise))
    }
}

impl Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.unit == rhs.unit {
            return Self::new(self.value.add(rhs.value), self.unit);
        }

        let (left, right) = if let (Some(left), Some(right)) = (self.unit, rhs.unit) {
            (left, right)
        } else {
            let unit = self.unit.or(rhs.unit);
            return Self::new(self.value.add(rhs.value), unit);
        };

        let precise = std::cmp::min(left, right);
        let value = self
            .convert_to(precise)
            .value
            .add(rhs.convert_to(precise).value);

        Self::new(value, Some(precise))
    }
}

impl Neg for Value {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.value, self.unit)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.unit {
            Some(unit) => write!(f, "{}{}", self.value, unit),
            None => write!(f, "{}", self.value),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::interpreter::{token::Unit, unit_prefix::UnitPrefix};

    #[test]
    fn test_value_display() {
        let value = Value::new(42.0, Some(FullUnit::new(UnitPrefix::Kilo, Unit::Byte)));
        assert_eq!(format!("{}", value), "42kB");

        let value = Value::new(42.0, None);
        assert_eq!(format!("{}", value), "42");
    }

    #[test]
    fn test_value_convert_to() {
        let value = Value::new(42.0, Some(FullUnit::new(UnitPrefix::Kilo, Unit::Byte)));
        let new_value = value.convert_to(FullUnit::new(UnitPrefix::Mega, Unit::Byte));
        assert_eq!(new_value.value(), 0.042);
        assert_eq!(
            new_value.unit(),
            Some(FullUnit::new(UnitPrefix::Mega, Unit::Byte))
        );

        let value = Value::new(42.0, None);
        let new_value = value.convert_to(FullUnit::new(UnitPrefix::Mega, Unit::Byte));
        assert_eq!(new_value.value(), 42.0);
        assert_eq!(
            new_value.unit(),
            Some(FullUnit::new(UnitPrefix::Mega, Unit::Byte))
        );
    }

    #[test]
    fn test_value_try_mul() {
        let value = Value::new(42.0, Some(FullUnit::new(UnitPrefix::Kilo, Unit::Byte)));
        let new_value = value.try_mul(Value::new(2.0, None)).unwrap();
        assert_eq!(new_value.value(), 84.0);
        assert_eq!(
            new_value.unit(),
            Some(FullUnit::new(UnitPrefix::Kilo, Unit::Byte))
        );

        let value = Value::new(42.0, None);
        let new_value = value
            .try_mul(Value::new(
                2.0,
                Some(FullUnit::new(UnitPrefix::Mega, Unit::Byte)),
            ))
            .unwrap();
        assert_eq!(new_value.value(), 84.0);
        assert_eq!(
            new_value.unit(),
            Some(FullUnit::new(UnitPrefix::Mega, Unit::Byte))
        );
    }

    #[test]
    fn test_value_try_div() {
        let value = Value::new(42.0, Some(FullUnit::new(UnitPrefix::Kilo, Unit::Byte)));
        let new_value = value.try_div(Value::new(2.0, None)).unwrap();
        assert_eq!(new_value.value(), 21.0);
        assert_eq!(
            new_value.unit(),
            Some(FullUnit::new(UnitPrefix::Kilo, Unit::Byte))
        );
    }
}
