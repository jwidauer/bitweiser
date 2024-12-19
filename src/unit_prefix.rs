use core::fmt;
use std::fmt::{Display, Formatter};

const KILO: u64 = 1_000u64.pow(1);
const MEGA: u64 = 1_000u64.pow(2);
const GIGA: u64 = 1_000u64.pow(3);
const TERA: u64 = 1_000u64.pow(4);
const PETA: u64 = 1_000u64.pow(5);
const EXA: u64 = 1_000u64.pow(6);
const KIBI: u64 = 1024u64.pow(1);
const MEBI: u64 = 1024u64.pow(2);
const GIBI: u64 = 1024u64.pow(3);
const TEBI: u64 = 1024u64.pow(4);
const PEBI: u64 = 1024u64.pow(5);
const EXBI: u64 = 1024u64.pow(6);

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum UnitPrefix {
    None,
    Kilo,
    Mega,
    Giga,
    Tera,
    Peta,
    Exa,
    Kibi,
    Mebi,
    Gibi,
    Tebi,
    Pebi,
    Exbi,
}

impl UnitPrefix {
    pub(crate) fn dec_from_num(num: u64) -> Self {
        match num {
            0..KILO => Self::None,
            KILO..MEGA => Self::Kilo,
            MEGA..GIGA => Self::Mega,
            GIGA..TERA => Self::Giga,
            TERA..PETA => Self::Tera,
            PETA..EXA => Self::Peta,
            EXA..=u64::MAX => Self::Exa,
        }
    }

    pub(crate) fn bin_from_num(num: u64) -> Self {
        match num {
            0..KIBI => Self::None,
            KIBI..MEBI => Self::Kibi,
            MEBI..GIBI => Self::Mebi,
            GIBI..TEBI => Self::Gibi,
            TEBI..PEBI => Self::Tebi,
            PEBI..EXBI => Self::Pebi,
            EXBI..=u64::MAX => Self::Exbi,
        }
    }

    pub(crate) fn len(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Kilo => 1,
            Self::Mega => 1,
            Self::Giga => 1,
            Self::Tera => 1,
            Self::Peta => 1,
            Self::Exa => 1,
            Self::Kibi => 2,
            Self::Mebi => 2,
            Self::Gibi => 2,
            Self::Tebi => 2,
            Self::Pebi => 2,
            Self::Exbi => 2,
        }
    }
}

impl TryFrom<&str> for UnitPrefix {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.to_lowercase();
        let val = match value.as_str() {
            "" => Self::None,
            "k" => Self::Kilo,
            "m" => Self::Mega,
            "g" => Self::Giga,
            "t" => Self::Tera,
            "p" => Self::Peta,
            "e" => Self::Exa,
            "ki" => Self::Kibi,
            "mi" => Self::Mebi,
            "gi" => Self::Gibi,
            "ti" => Self::Tebi,
            "pi" => Self::Pebi,
            "ei" => Self::Exbi,
            _ => anyhow::bail!("Invalid unit prefix: {}", value),
        };
        Ok(val)
    }
}

impl TryFrom<u64> for UnitPrefix {
    type Error = anyhow::Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        let val = match value {
            1 => Self::None,
            KILO => Self::Kilo,
            MEGA => Self::Mega,
            GIGA => Self::Giga,
            TERA => Self::Tera,
            PETA => Self::Peta,
            EXA => Self::Exa,
            KIBI => Self::Kibi,
            MEBI => Self::Mebi,
            GIBI => Self::Gibi,
            TEBI => Self::Tebi,
            PEBI => Self::Pebi,
            EXBI => Self::Exbi,
            _ => anyhow::bail!("Invalid unit prefix: {}", value),
        };
        Ok(val)
    }
}

impl From<UnitPrefix> for u64 {
    fn from(value: UnitPrefix) -> u64 {
        type UP = UnitPrefix;
        match value {
            UP::None => 1,
            UP::Kilo => KILO,
            UP::Mega => MEGA,
            UP::Giga => GIGA,
            UP::Tera => TERA,
            UP::Peta => PETA,
            UP::Exa => EXA,
            UP::Kibi => KIBI,
            UP::Mebi => MEBI,
            UP::Gibi => GIBI,
            UP::Tebi => TEBI,
            UP::Pebi => PEBI,
            UP::Exbi => EXBI,
        }
    }
}

impl Display for UnitPrefix {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        type UP = UnitPrefix;
        match self {
            UP::None => write!(f, ""),
            UP::Kilo => write!(f, "K"),
            UP::Mega => write!(f, "M"),
            UP::Giga => write!(f, "G"),
            UP::Tera => write!(f, "T"),
            UP::Peta => write!(f, "P"),
            UP::Exa => write!(f, "E"),
            UP::Kibi => write!(f, "Ki"),
            UP::Mebi => write!(f, "Mi"),
            UP::Gibi => write!(f, "Gi"),
            UP::Tebi => write!(f, "Ti"),
            UP::Pebi => write!(f, "Pi"),
            UP::Exbi => write!(f, "Ei"),
        }
    }
}
