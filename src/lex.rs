use crate::lex::number::SizedNum;
use nom::character::is_alphanumeric;
use nom::character::streaming::digit1;
use nom::number::streaming::double;
use nom::number::streaming::float;
// use strum::EnumDiscriminants;
use strum_macros::EnumDiscriminants;

use nom::{complete, many1, named, one_of, take_while1, ws, IResult};
use std::convert::TryFrom;
use std::fmt::Debug;
use std::fmt::Display;

pub mod number;

#[derive(Debug, Hash, Copy, Clone, PartialEq, Eq)]
pub enum DelimToken {
    Paren,
    Bracket,
    Curly,
}

#[derive(Clone, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(Hash), name(TokenType))]
pub enum Token<'s> {
    Ident(&'s [u8]), // alphanumeric1
    Path(&'s [u8]),
    Use,
    As,
    Any,
    Super,
    Class,
    Type,
    Enum,
    Int(SizedNum),
    F32(f32),
    F64(f64),
    Slash, // char()
    OpenDelim(DelimToken),
    CloseDelim(DelimToken),
    DQuote,
    SQuote,
    Colon,
    Semi,
    Comma,
    Period,
    Dollar,
    Star,
    Eq,
    Lt,
    Gt,
    Tilde,
}

impl Into<String> for Token<'_> {
    fn into(self) -> String {
        use DelimToken::*;
        use Token::*;
        match self {
            Use => "use".into(),
            As => "as".into(),
            Any => "Any".into(),
            Super => "super".into(),
            Class => "class".into(),
            Type => "type".into(),
            Enum => "enum".into(),
            Slash => "/".into(),
            DQuote => "\"".into(),
            SQuote => "\'".into(),
            Colon => ":".into(),
            Semi => ";".into(),
            Comma => ",".into(),
            Dollar => "$".into(),
            Star => "*".into(),
            Eq => "=".into(),
            Lt => "<".into(),
            Gt => ">".into(),
            Tilde => "~".into(),
            Period => ".".into(),
            OpenDelim(o) => match o {
                Paren => "(".into(),
                Bracket => "[".into(),
                Curly => "{".into(),
            },
            CloseDelim(c) => match c {
                Paren => ")".into(),
                Bracket => "]".into(),
                Curly => "}".into(),
            },
            Int(i) => i.to_string(),
            F32(f) => f.to_string(),
            F64(f) => f.to_string(),
            Ident(i) | Path(i) => String::from_utf8(i.to_vec()).unwrap(),
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s: String = self.clone().into();
        write!(f, "{}", s)
    }
}

impl Debug for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl TryFrom<char> for Token<'_> {
    type Error = String;
    fn try_from(c: char) -> Result<Self, Self::Error> {
        use DelimToken::*;
        use Token::*;
        match c {
            '/' => Ok(Slash),
            '(' => Ok(OpenDelim(Paren)),
            '[' => Ok(OpenDelim(Bracket)),
            '{' => Ok(OpenDelim(Curly)),
            ')' => Ok(CloseDelim(Paren)),
            ']' => Ok(CloseDelim(Bracket)),
            '}' => Ok(CloseDelim(Curly)),
            '"' => Ok(DQuote),
            '\'' => Ok(SQuote),
            ':' => Ok(Colon),
            ';' => Ok(Semi),
            ',' => Ok(Comma),
            '$' => Ok(Dollar),
            '*' => Ok(Star),
            '=' => Ok(Eq),
            '<' => Ok(Lt),
            '>' => Ok(Gt),
            '~' => Ok(Tilde),
            '.' => Ok(Period),
            _ => Err(format!("{} is not a special token character.", c)),
        }
    }
}

named!(recognize_special(&[u8]) -> char, one_of!("/([{}])\"\':;,.$*=<>~"));
named!(recognize_ident(&[u8]) -> &[u8], take_while1!(|c| {c == b'_' || is_alphanumeric(c)}));

macro_rules! rec_key {
    ($k:literal, $t:path, $i:ident) => {{
        let res = nom::bytes::streaming::tag::<_, _, (&[u8], nom::error::ErrorKind)>($k)($i);
        match res {
            Ok((rem, _k)) => return Ok((rem, $t)),
            Err(e) => e,
        }
    }};
}

pub fn recognize_keyword<'s>(input: &'s [u8]) -> IResult<&'s [u8], Token<'s>> {
    use Token::*;
    rec_key!("use ", Use, input);
    rec_key!("as ", As, input);
    rec_key!("Any", Any, input);
    rec_key!("super ", Super, input);
    rec_key!("class ", Class, input);
    rec_key!("type ", Type, input);
    let err = rec_key!("enum ", Enum, input);
    Err(err)
}

pub fn int_literal<'s>(input: &'s [u8]) -> IResult<&'s [u8], Token<'s>> {
    match digit1(input) {
        Ok((rem, num)) => Ok((
            rem,
            Token::Int(String::from_utf8(num.to_vec()).unwrap().parse().unwrap()),
        )),
        Err(e) => Err(e),
    }
}

pub fn float_literal<'s>(input: &'s [u8]) -> IResult<&'s [u8], Token<'s>> {
    match float::<&[u8], ()>(input) {
        Ok((rem, num)) => Ok((rem, Token::F32(num))),
        Err(_) => match double(input) {
            Ok((rem, num)) => Ok((rem, Token::F64(num))),
            Err(e) => Err(e),
        },
    }
}

pub fn recognize_number<'s>(input: &'s [u8]) -> IResult<&'s [u8], Token<'s>> {
    // basically all of this is because ints are also recognized as floats
    let float = float_literal(input);
    match float {
        Ok((f_rem, _)) => {
            let int = int_literal(input);
            match int {
                Ok((i_rem, _)) => {
                    if i_rem.len() == f_rem.len() {
                        int
                    } else {
                        float
                    }
                }
                _ => float,
            }
        }
        Err(_) => float,
    }
}

pub fn recognize_token<'s>(input: &'s [u8]) -> IResult<&'s [u8], Token<'s>> {
    let special = recognize_special(input);
    if let Ok((rem, c)) = special {
        return Ok((rem, Token::try_from(c).unwrap()));
    }
    let number = recognize_number(input);
    if let Ok(n) = number {
        return Ok(n);
    }
    let key = recognize_keyword(input);
    if let Ok((rem, key)) = key {
        return Ok((rem, key));
    }
    let ident = recognize_ident(input);
    if let Ok((rem, id)) = ident {
        return Ok((rem, Token::Ident(id)));
    }
    match ident {
        Err(e) => Err(e),
        _ => unreachable!(),
    }
}

named!(pub lex(&[u8]) -> Token, complete!(ws!(recognize_token)));
named!(pub multilex(&[u8]) -> Vec<Token>, many1!(lex));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn special() {
        assert_eq!(recognize_special(b"{"), Ok((b"" as _, '{')));
        assert_eq!(recognize_special(b"."), Ok((b"" as _, '.')));
    }

    #[test]
    fn ident() {
        let ident = b"_ident;";
        assert_eq!(
            recognize_ident(ident),
            Ok((b";" as _, &ident[0..ident.len() - 1]))
        );
    }

    #[test]
    fn int() {
        let num = b"12345;";
        assert_eq!(
            int_literal(num),
            Ok((b";" as _, Token::Int(12345u16.into())))
        );
    }

    #[test]
    fn float() {
        let num = b"12345.6789;";
        assert_eq!(
            float_literal(num),
            Ok((b";" as _, Token::F32(12345.6789f32)))
        );
    }

    #[test]
    fn double() {
        let num = b"9876543210123456789.9876543210123456789;";
        assert_eq!(
            float_literal(num),
            Ok((
                b";" as _,
                Token::F64(9876543210123456789.9876543210123456789f64)
            ))
        );
    }

    #[test]
    fn token() {
        //use std::str::from_utf8;
        let special = b"{";
        assert_eq!(
            recognize_token(special),
            Ok((b"" as _, Token::OpenDelim(DelimToken::Curly)))
        );
        let ident = b"_ident.;";
        assert_eq!(
            recognize_token(ident),
            Ok((b".;" as _, Token::Ident(&ident[0..ident.len() - 2])))
        );
        //assert_eq!(res, Ok((b"efg)" as _, b"abc(d" as _)));
    }

    #[allow(dead_code)]
    fn print_lex<'s>(res: &IResult<&'s [u8], Vec<Token<'s>>>) {
        match res {
            Ok((rem, tokens)) => {
                eprintln!("Read:");
                for token in tokens {
                    eprint!("{} ", token);
                }
                eprintln!("\nRemaining:\n{}", String::from_utf8(rem.to_vec()).unwrap());
            }
            Err(e) => {
                dbg!(e);
            }
        }
    }

    #[test]
    fn tokens() {
        fn test_res<'s>(res: &IResult<&'s [u8], Vec<Token<'s>>>) -> bool {
            let success = match res {
                Ok((rem, _)) => rem.len() == 0,
                _ => false,
            };
            if !success {
                print_lex(res);
            }
            success
        }
        let about = include_bytes!("../data/ashwalker.net/about.stn");
        let a_res = multilex(about);
        assert!(test_res(&a_res));
        let style = include_bytes!("../data/ashwalker.net/style.stn");
        let s_res = multilex(style);
        assert!(test_res(&s_res));
        // print_lex(&s_res);
        let painter = include_bytes!("../data/painter/painter.stn");
        let p_res = multilex(painter);
        assert!(test_res(&p_res));
        //print_lex(p_res);
    }
}
