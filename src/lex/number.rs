use std::collections::VecDeque;
use std::convert::TryInto;
use std::fmt::Debug;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct SizedNum {
    pub signed: bool,
    pub bits: Vec<bool>,
}

impl FromStr for SizedNum {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num: u128 = s.parse()?;
        Ok(num.into())
    }
}

impl SizedNum {
    pub fn zero(signed: bool, size: u8) -> Self {
        if size < 1 || size > 128 {
            panic!("Tried to make SizedNum with size = {}", size)
        }
        Self {
            signed,
            bits: (0..size).map(|_| false).collect(),
        }
    }

    pub fn from_bits(signed: bool, bits: Vec<bool>) -> Self {
        if bits.is_empty() || bits.len() > 128 {
            panic!("Tried to make SizedNum with size = {}", bits.len())
        }
        SizedNum { signed, bits }
    }

    pub fn is_neg(&self) -> bool {
        self.signed && self.bits[0]
    }
}

impl Debug for SizedNum {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let bits = {
            let mut res = String::new();
            for bit in &self.bits {
                res += if *bit { "1" } else { "0" };
            }
            res
        };
        f.debug_struct("SizedNum")
            .field("signed", &self.signed)
            .field("bits", &bits)
            .finish()
    }
}

impl Display for SizedNum {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            Number::from(self.clone()),
            if self.signed { "i" } else { "u" },
            self.bits.len()
        )
    }
}

macro_rules! conv_sized {
    ($t:ty) => {
        impl From<$t> for SizedNum {
            fn from(mut n: $t) -> Self {
                let mut bits = VecDeque::new();
                while n > 0 {
                    bits.push_front(n % 2 == 1);
                    n /= 2;
                }
                Self::from_bits(false, bits.into_iter().collect::<Vec<_>>())
            }
        }
        impl Into<$t> for SizedNum {
            fn into(self) -> $t {
                Number::from(self).try_into().unwrap()
            }
        }
    };
    ($t:ty, signed) => {
        impl From<$t> for SizedNum {
            fn from(mut n: $t) -> Self {
                let mut bits = VecDeque::new();
                bits.push_back(n < 0);
                n = n.abs();
                while n > 0 {
                    bits.insert(1, n % 2 == 1);
                    n /= 2;
                }
                Self::from_bits(true, bits.into_iter().collect::<Vec<_>>())
            }
        }
        impl Into<$t> for SizedNum {
            fn into(self) -> $t {
                Number::from(self).try_into().unwrap()
            }
        }
    };
}

conv_sized!(u8);
conv_sized!(u16);
conv_sized!(u32);
conv_sized!(u64);
conv_sized!(u128);
conv_sized!(i8, signed);
conv_sized!(i16, signed);
conv_sized!(i32, signed);
conv_sized!(i64, signed);
conv_sized!(i128, signed);

impl Into<String> for SizedNum {
    fn into(self) -> String {
        format!(
            "{}{}{}",
            Number::from(self.clone()),
            if self.signed { "u" } else { "i" },
            self.bits.len()
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Number {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    F32(f32),
    F64(f64),
}

impl Into<String> for Number {
    fn into(self) -> String {
        use Number::*;
        match self {
            U8(n) => format!("{}", n),
            U16(n) => format!("{}", n),
            U32(n) => format!("{}", n),
            U64(n) => format!("{}", n),
            U128(n) => format!("{}", n),
            I8(n) => format!("{}", n),
            I16(n) => format!("{}", n),
            I32(n) => format!("{}", n),
            I64(n) => format!("{}", n),
            I128(n) => format!("{}", n),
            F32(n) => format!("{}", n),
            F64(n) => format!("{}", n),
        }
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s: String = (*self).into();
        write!(f, "{}", s)
    }
}

macro_rules! from_sized {
    ($u:ty, $i:ty, $s:ident) => {
        if $s.signed {
            let mut res: $i = 0;
            const TWO: $i = 2;
            for i in 0..($s.bits.len() - 1) {
                let bit = $s.bits[$s.bits.len() - i - 1];
                if bit {
                    res += TWO.pow(i as _);
                }
            }
            if $s.is_neg() {
                res = -res
            }
            Number::from(res)
        } else {
            let mut res: $u = 0;
            const TWO: $u = 2;
            for i in 0..$s.bits.len() {
                let bit = $s.bits[$s.bits.len() - i - 1];
                if bit {
                    res += TWO.pow(i as _);
                }
            }
            Number::from(res)
        }
    };
}

impl From<SizedNum> for Number {
    #[allow(clippy::cognitive_complexity)]
    fn from(n: SizedNum) -> Self {
        match n.bits.len() {
            1...8 => from_sized!(u8, i8, n),
            9...16 => from_sized!(u16, i16, n),
            17...32 => from_sized!(u32, i32, n),
            33...64 => from_sized!(u64, i64, n),
            65...128 => from_sized!(u128, i128, n),
            _ => unreachable!(),
        }
    }
}

macro_rules! conv_num {
    ($num:ty, $match:path) => {
        impl TryInto<$num> for Number {
            type Error = &'static str;
            fn try_into(self) -> Result<$num, Self::Error> {
                if let $match(n) = self {
                    Ok(n)
                } else {
                    Err("Tried to convert Number enum to mismatched type")
                }
            }
        }
        impl From<$num> for Number {
            fn from(n: $num) -> Self {
                $match(n)
            }
        }
    };
}

conv_num!(u8, Number::U8);
conv_num!(u16, Number::U16);
conv_num!(u32, Number::U32);
conv_num!(u64, Number::U64);
conv_num!(u128, Number::U128);
conv_num!(i8, Number::I8);
conv_num!(i16, Number::I16);
conv_num!(i32, Number::I32);
conv_num!(i64, Number::I64);
conv_num!(i128, Number::I128);
conv_num!(f32, Number::F32);
conv_num!(f64, Number::F64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sized() {
        let sized = SizedNum::from(12u8);
        dbg!(&sized);
        let ret: u8 = sized.into();
        assert_eq!(ret, 12u8);

        let sized = SizedNum::from(-64i16);
        dbg!(&sized);
        eprintln!("{}", sized);
        let ret: i8 = sized.into();
        assert_eq!(ret, -64i8);

        let sized = SizedNum::from(256i16);
        dbg!(&sized);
        eprintln!("{}", sized);
        let ret: i16 = sized.into();
        assert_eq!(ret, 256i16);
    }
}
