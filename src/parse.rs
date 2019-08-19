<<<<<<< HEAD
use crate::lex::number::SizedNum;
use crate::lex::DelimToken;
use crate::lex::{Token, TokenType};
use birch::Tree;
use either::Either;
use std::collections::HashMap;
use std::collections::HashSet;

pub mod exprbuilder;

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
    Int(SizedNum),
    F32(f32),
    F64(f32),
}

impl Into<String> for Literal {
    fn into(self) -> String {
        use Literal::*;
        match self {
            Str(s) => format!("\"{}\"", s),
            Selector(s) => s.into(),
            Int(i) => i.into(),
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

pub struct Parser<'s> {
    pub ast: Tree<AstBlock<'s>>,
}

impl Default for Parser<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'s> Parser<'s> {
    pub fn new() -> Self {
        Self {
            ast: Tree::new(AstBlock::Root),
        }
    }

    pub fn parse(
        &mut self,
        data: &'s [u8],
    ) -> Result<&'s [u8], nom::Err<(&'s [u8], nom::error::ErrorKind)>> {
        let (rem, mut tokens) = crate::lex::multilex(data)?;
        let mut parent_stack = vec![0];
        let mut curr_stack = Vec::new();
        for token in tokens.drain(0..) {
            let parent = parent_stack[parent_stack.len() - 1];
            if curr_stack.is_empty() {
                let curr = self
                    .ast
                    .add_child(parent, AstBlock::Builder(ExprBuilder::new()));
                curr_stack.push(curr);
            }
            let curr = curr_stack[curr_stack.len() - 1];
            let result = match self.ast.0.vert_mut(curr).val {
                AstBlock::Builder(ref mut b) => b.push(token),
                _ => unreachable!(), // Because we take completed builders off of curr_stack
            };
            use ExprStatus::*;
            match result {
                Ready(ex) => {
                    self.ast.0.vert_mut(curr).val = AstBlock::Expr(ex);
                    curr_stack.pop();
                    parent_stack.pop();
                }
                Incomplete => (),
                Error => panic!("Some sort of parse error."),
                Inner => {
                    parent_stack.push(curr);
                    let parent = curr;
                    let inner = self
                        .ast
                        .add_child(parent, AstBlock::Builder(ExprBuilder::new()));
                    if let AstBlock::Builder(ref mut b) = self.ast.0.vert_mut(curr).val {
                        b.push_inner(inner);
                    }
                    curr_stack.push(inner);
                }
            }
        }
        Ok(rem)
    }

    pub fn finish(self) -> Tree<()> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::parse::machine::*;
    const ABOUT: &[u8] = include_bytes!("../data/ashwalker.net/about.stn");

    // #[test]
    // fn parse() {
    //     use Token::*;
    //     let mut machine = ParseMachine::init(
    //         |mut state, token| {
    //             state.stack.push(token);
    //             match token {
    //                 Semi => Vec::new(),
    //                 _ => vec![state],
    //             }
    //         },
    //         true,
    //     );
    //     machine.parse(ABOUT);
    //     eprintln!("{}", machine);
    // }
}
=======
use crate::lex::number::SizedNum;
use crate::lex::DelimToken;
use crate::lex::{Token, TokenType};
use birch::Tree;
use either::Either;
use std::collections::HashMap;
use std::collections::HashSet;

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
    Int(SizedNum),
    F32(f32),
    F64(f32),
}

impl Into<String> for Literal {
    fn into(self) -> String {
        use Literal::*;
        match self {
            Str(s) => format!("\"{}\"", s),
            Selector(s) => s.into(),
            Int(i) => i.into(),
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ExprStatus {
    Ready,
    Incomplete,
    Inner,
    Error,
}

#[derive(Debug)]
pub enum AstBlock<'s> {
    Expr(()),
    Builder(ExprBuilder<'s>),
    Root,
}

#[derive(Debug, Default)]
pub struct ExprBuilder<'s> {
    pub tokens: Vec<Either<Token<'s>, usize>>,
}

impl<'s> ExprBuilder<'s> {
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    pub fn push(&mut self, token: Token<'s>) -> ExprStatus {
        use ExprStatus::*;
        let ok = match self.tokens
        Incomplete
    }

    pub fn push_inner(&mut self, inner: usize) {
        self.tokens.push(Either::Right(inner))
    }
}

pub struct Parser<'s> {
    pub ast: Tree<AstBlock<'s>>,
}

impl Default for Parser<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'s> Parser<'s> {
    pub fn new() -> Self {
        Self {
            ast: Tree::new(AstBlock::Root),
        }
    }

    pub fn parse(
        &mut self,
        data: &'s [u8],
    ) -> Result<&'s [u8], nom::Err<(&'s [u8], nom::error::ErrorKind)>> {
        let (rem, mut tokens) = crate::lex::multilex(data)?;
        let mut parent_stack = vec![0];
        let mut curr_stack = Vec::new();
        for token in tokens.drain(0..) {
            let parent = parent_stack[parent_stack.len() - 1];
            if curr_stack.is_empty() {
                let curr = self
                    .ast
                    .add_child(parent, AstBlock::Builder(ExprBuilder::new()));
                curr_stack.push(curr);
            }
            let curr = curr_stack[curr_stack.len() - 1];
            let result = match self.ast.0.vert_mut(curr).val {
                AstBlock::Builder(ref mut b) => b.push(token),
                _ => unreachable!(), // Because we take completed builders off of curr_stack
            };
            use ExprStatus::*;
            match result {
                Complete(ex) => {
                    self.ast.0.vert_mut(curr).val = AstBlock::Expr(ex);
                    curr_stack.pop();
                    parent_stack.pop();
                }
                Incomplete => (),
                Error => panic!("Some sort of parse error."),
                Inner => {
                    parent_stack.push(curr);
                    let parent = curr;
                    let inner = self
                        .ast
                        .add_child(parent, AstBlock::Builder(ExprBuilder::new()));
                    if let AstBlock::Builder(ref mut b) = self.ast.0.vert_mut(curr).val {
                        b.push_inner(inner);
                    }
                    curr_stack.push(inner);
                }
            }
        }
        Ok(rem)
    }

    pub fn finish(self) -> Tree<()> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::parse::machine::*;
    const ABOUT: &[u8] = include_bytes!("../data/ashwalker.net/about.stn");

    // #[test]
    // fn parse() {
    //     use Token::*;
    //     let mut machine = ParseMachine::init(
    //         |mut state, token| {
    //             state.stack.push(token);
    //             match token {
    //                 Semi => Vec::new(),
    //                 _ => vec![state],
    //             }
    //         },
    //         true,
    //     );
    //     machine.parse(ABOUT);
    //     eprintln!("{}", machine);
    // }
}
>>>>>>> 98476a440fdd34cca12b2fea0108224ca3c2f6f2
