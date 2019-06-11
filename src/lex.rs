use nom::character::is_alphanumeric;
use nom::character::streaming::digit1;

use nom::number::streaming::recognize_float;
use nom::{complete, many1, named, one_of, take_while1, ws, IResult};
use std::convert::TryFrom;
use std::fmt::Debug;
use std::fmt::Display;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DelimToken {
    Paren,
    Bracket,
    Curly,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NumLiteral<'s> {
    Int(&'s [u8]),
    Float(&'s [u8]),
}

impl Into<String> for NumLiteral<'_> {
    fn into(self) -> String {
        match self {
            NumLiteral::Int(i) => String::from_utf8(i.to_vec()).unwrap(),
            NumLiteral::Float(f) => String::from_utf8(f.to_vec()).unwrap(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Token<'s> {
    Ident(&'s [u8]), // alphanumeric1
    Number(NumLiteral<'s>),
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
            Number(n) => n.into(),
            Ident(i) => String::from_utf8(i.to_vec()).unwrap(),
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s: String = (*self).into();
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

pub fn int_literal<'s>(input: &'s [u8]) -> IResult<&'s [u8], NumLiteral<'s>> {
    match digit1(input) {
        Ok((rem, num)) => Ok((rem, NumLiteral::Int(num))),
        Err(e) => Err(e),
    }
}

pub fn float_literal<'s>(input: &'s [u8]) -> IResult<&'s [u8], NumLiteral<'s>> {
    match recognize_float(input) {
        Ok((rem, num)) => Ok((rem, NumLiteral::Float(num))),
        Err(e) => Err(e),
    }
}

pub fn recognize_number<'s>(input: &'s [u8]) -> IResult<&'s [u8], NumLiteral<'s>> {
    // basically all of this is because ints are also recognized as floats
    let float = float_literal(input);
    match float {
        Ok((_, NumLiteral::Float(f))) => {
            let int = int_literal(input);
            match int {
                Ok((_, NumLiteral::Int(i))) => {
                    if i.len() == f.len() {
                        int
                    } else {
                        float
                    }
                }
                _ => float,
            }
        }
        Err(_) => float,
        _ => unreachable!(),
    }
}

pub fn recognize_token<'s>(input: &'s [u8]) -> IResult<&'s [u8], Token<'s>> {
    let special = recognize_special(input);
    if let Ok((rem, c)) = special {
        return Ok((rem, Token::try_from(c).unwrap()));
    }
    let number = recognize_number(input);
    if let Ok((rem, num)) = number {
        return Ok((rem, Token::Number(num)));
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
            Ok((b";" as _, NumLiteral::Int(&num[0..num.len() - 1])))
        );
    }

    #[test]
    fn float() {
        let num = b"12345.6789;";
        assert_eq!(
            float_literal(num),
            Ok((b";" as _, NumLiteral::Float(&num[0..num.len() - 1])))
        );
    }

    #[test]
    fn number() {
        let int = b"12345;";
        assert_eq!(
            recognize_number(int),
            Ok((b";" as _, NumLiteral::Int(&int[0..int.len() - 1])))
        );
        let float = b"12345.6789;";
        assert_eq!(
            recognize_number(float),
            Ok((b";" as _, NumLiteral::Float(&float[0..float.len() - 1])))
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
        let int = b"54321;";
        assert_eq!(
            recognize_token(int),
            Ok((
                b";" as _,
                Token::Number(NumLiteral::Int(&int[0..int.len() - 1]))
            ))
        );
        let float = b"54321.9876;";
        assert_eq!(
            recognize_token(float),
            Ok((
                b";" as _,
                Token::Number(NumLiteral::Float(&float[0..float.len() - 1]))
            ))
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
