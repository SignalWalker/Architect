use std::convert::TryInto;
use std::str::FromStr;

pub enum Keyword {
    Use,
    As,
    Class,
    Type,
    Super,
    Enum,
}

impl Into<&'static str> for Keyword {
    fn into(self) -> &'static str {
        use Keyword::*;
        match self {
            Use => "use",
            As => "as",
            Class => "class",
            Type => "type",
            Super => "super",
            Enum => "enum",
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum StaticType {
    Any,
    Str,
    Selector,
    Unsigned(usize),
    Signed(usize),
    F32,
    F64,
}

impl TryInto<&'static str> for StaticType {
    type Error = ();
    fn try_into(self) -> Result<&'static str, Self::Error> {
        use StaticType::*;
        match self {
            Any => Ok("Any"),
            Str => Ok("String"),
            Selector => Ok("Selector"),
            F32 => Ok("f32"),
            F64 => Ok("f64"),
            _ => Err(()),
        }
    }
}

impl Into<String> for StaticType {
    fn into(self) -> String {
        use StaticType::*;
        match self {
            Unsigned(s) => format!("u{}", s),
            Signed(s) => format!("i{}", s),
            _ => String::from_str(self.try_into().unwrap()).unwrap(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Integer {
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
}

macro_rules! into_num {
    ($num:ty, $match:path) => {
        impl TryInto<$num> for Integer {
            type Error = ::std::convert::Infallible;
            fn try_into(self) -> Result<$num, Self::Error> {
                if let $match(n) = self {
                    Ok(n)
                } else {
                    unimplemented!("Into Num: {}", stringify!($num));
                }
            }
        }
    };
}

into_num!(u8, Integer::U8);
into_num!(u16, Integer::U16);
into_num!(u32, Integer::U32);
into_num!(u64, Integer::U64);
into_num!(u128, Integer::U128);
into_num!(i8, Integer::I8);
into_num!(i16, Integer::I16);
into_num!(i32, Integer::I32);
into_num!(i64, Integer::I64);
into_num!(i128, Integer::I128);
