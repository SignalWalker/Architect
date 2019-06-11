use crate::lex::Token;
use crate::parse::number::SizedNum;
use std::collections::HashMap;

pub mod machine;
pub mod number;

#[derive(Debug, Copy, Clone)]
pub struct Selector {}

impl Into<String> for Selector {
    fn into(self) -> String {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    Str(String),
    Selector(Selector),
    Integer(SizedNum),
    F32(f32),
    F64(f32),
}

impl Into<String> for Literal {
    fn into(self) -> String {
        use Literal::*;
        match self {
            Str(s) => format!("\"{}\"", s),
            Selector(s) => s.into(),
            Integer(i) => i.into(),
            F32(f) => format!("{}f32", f),
            F64(f) => format!("{}f64", f),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ObjType {
    Super,
    Type,
    Class,
}

#[derive(Debug, Copy, Clone)]
pub struct ClassDef {}

#[derive(Debug, Clone)]
pub enum Ast<'s> {
    Use(String, Option<String>),
    Literal(Literal),
    Def(ObjType, ClassDef),
    Enum(ObjType, Vec<ClassDef>),
    List(Option<String>, Vec<Ast<'s>>),
    Map(Option<String>, HashMap<String, Ast<'s>>),
    Tuple(Option<String>, Vec<Ast<'s>>),
    Error(Vec<Token<'s>>),
}

impl<'s> From<Vec<Token<'s>>> for Ast<'s> {
    fn from(tokens: Vec<Token<'s>>) -> Self {
        Ast::Error(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::machine::*;
    const ABOUT: &[u8] = include_bytes!("../data/ashwalker.net/about.stn");

    #[test]
    fn parse() {
        use Token::*;
        let mut machine = ParseMachine::init(
            |mut state, token| {
                state.stack.push(token);
                match token {
                    Semi => Vec::new(),
                    _ => vec![state],
                }
            },
            true,
        );
        machine.parse(ABOUT);
        eprintln!("{}", machine);
    }
}
