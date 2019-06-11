use crate::lex::Token;
use std::fmt::Display;

use std::fmt::Debug;

use crate::parse::Ast;

use crate::lex::lex;

type StTransition<'s> = fn(State<'s>, Token<'s>) -> Vec<State<'s>>;
type StFinish<'s> = fn(State<'s>) -> Vec<State<'s>>;

#[derive(Clone)]
pub struct State<'s> {
    pub trans: StTransition<'s>,
    pub stack: Vec<Token<'s>>,
    pub accept: bool,
}

impl<'s> From<StateBuilder<'s>> for State<'s> {
    fn from(b: StateBuilder<'s>) -> Self {
        b.build()
    }
}

impl Debug for State<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("State")
            .field("accept", &self.accept)
            .field("stack", &self.stack)
            .finish()
    }
}

impl<'s> State<'s> {
    pub fn trans(trans: StTransition<'s>) -> StateBuilder<'s> {
        StateBuilder::new(trans)
    }
}

pub struct StateBuilder<'s> {
    pub trans: StTransition<'s>,
    pub stack: Vec<Token<'s>>,
    pub accept: bool,
}

impl<'s> StateBuilder<'s> {
    fn new(trans: StTransition<'s>) -> Self {
        Self {
            trans,
            stack: Vec::new(),
            accept: false,
        }
    }

    pub fn accept(mut self, a: bool) -> Self {
        self.accept = a;
        self
    }

    pub fn stack(mut self, stack: Vec<Token<'s>>) -> Self {
        self.stack = stack;
        self
    }

    pub fn build(self) -> State<'s> {
        State {
            trans: self.trans,
            stack: self.stack,
            accept: self.accept,
        }
    }
}

#[derive(Debug)]
pub struct ParseMachine<'s> {
    pub initial: State<'s>,
    pub states: Vec<State<'s>>,
    pub ast: Vec<Ast<'s>>,
}

impl<'s> ParseMachine<'s> {
    pub fn init(trans: StTransition<'s>, accept: bool) -> Self {
        let init = State::trans(trans).accept(accept).build();
        Self {
            initial: init.clone(),
            states: vec![init],
            ast: Vec::new(),
        }
    }

    pub fn state(mut self, trans: StTransition<'s>) -> Self {
        self.states.push(State::trans(trans).build());
        self
    }

    pub fn accept(mut self, trans: StTransition<'s>) -> Self {
        self.states.push(State::trans(trans).accept(true).build());
        self
    }

    pub fn parse(
        &mut self,
        mut data: &'s [u8],
    ) -> Result<(), nom::Err<(&[u8], nom::error::ErrorKind)>> {
        loop {
            let token = lex(data);
            match token {
                Ok((rem, t)) => {
                    data = rem;
                    if let Some(a) = self.consume(t) {
                        self.ast.push(a);
                    }
                }
                Err(e) => return Err(e),
            }
            if data.is_empty() {
                break;
            }
        }
        Ok(())
    }

    pub fn consume(&mut self, t: Token<'s>) -> Option<Ast<'s>> {
        let mut next = Vec::new();
        let mut accept = Vec::new();
        for state in self.states.drain(0..) {
            if state.accept {
                accept.push(state.stack.clone());
            }
            let mut res = (state.trans)(state, t);
            next.append(&mut res);
        }
        self.states = next;
        if self.states.is_empty() {
            self.states.push(self.initial.clone());
            accept.sort_by(|a, b| b.len().cmp(&a.len()));
            let mut prev = None;
            for tokens in accept.drain(0..) {
                let ast = Ast::from(tokens);
                if let Ast::Error(_) = ast {
                    prev = Some(ast);
                    continue;
                }
                return Some(ast);
            }
            return prev;
        }
        None
    }
}

impl Display for ParseMachine<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        fn write_tokens<'s>(tokens: &[Token<'s>]) -> String {
            let mut res = String::new();
            for token in tokens {
                res += &format!("{} ", token)
            }
            res
        }
        let state_str = {
            let mut res = String::new();
            for state in self.states.iter() {
                res += &format!("{} ", write_tokens(&state.stack))
            }
            res
        };
        f.debug_struct("ParseMachine")
            .field("states", &state_str)
            .field("ast", &self.ast)
            .finish()
    }
}
