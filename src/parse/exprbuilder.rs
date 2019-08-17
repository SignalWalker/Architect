use crate::lex::Token;
use either::Either;

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
        let ok = match self.tokens[self.tokens.len() - 1] {};
        Incomplete
    }

    pub fn push_inner(&mut self, inner: usize) {
        self.tokens.push(Either::Right(inner))
    }
}
