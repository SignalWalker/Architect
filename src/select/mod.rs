use super::*;
use birch::Tree;
use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest::Parser;
use std::str::FromStr;
use Stone;

#[derive(Clone, Default)]
pub struct Environment {
    // pub branches: Vec<usize>,
// pub selectors: Vec<Selector<'a>>,
// // Selector Index > Part of Selection > Stone Indices
// pub selected: HashMap<usize, HashMap<usize, Vec<usize>>>,
}

// impl<'a> Environment<'a> {
//     pub fn select(&mut self, trees: &[Tree<Stone>], stone: &Stone, stone_index: usize) {
//         let len = self.selectors.len();
//         for i in 0..len {
//             let (selected, selection) = {
//                 let selector = &self.selectors[i];
//                 selector.select(&self, trees, stone)
//             };
//             if selected {
//                 self.push(i, selection, stone_index)
//             }
//         }
//     }

//     pub fn push(&mut self, selector: usize, selection: usize, stone: usize) {
//         self.selected
//             .entry(selector)
//             .or_insert_with(HashMap::new)
//             .entry(selection)
//             .or_insert_with(Vec::new)
//             .push(stone);
//     }
// }

const _GRAMMAR: &str = include_str!("select.pest");

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum SelectorError {
    Parse,
    NotSelector,
}

impl From<SelectorError> for StoneError {
    fn from(e: SelectorError) -> StoneError {
        StoneError::Selector(e)
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum Either<A, B> {
    A(A),
    B(B),
}

type EiStr = Either<Selector, String>;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum SelectRule {
    All,
    None,
    This,
    Parent,
    Ancestors,
    Root,
    Children,
    Descendants,
    Class(String),
    Env(EiStr),
    Order(Selector),
    Attr(EiStr, Option<EiStr>),
    Index(usize),
    State(EiStr),
    IfElse(Selector, Selector),
    Slide(Selector),
    Or(Selector),
}

#[derive(Parser, PartialEq, Clone, Eq, Hash, Debug)]
#[grammar = "select/select.pest"]
pub struct Selector {
    raw: String,
    rules: Vec<SelectRule>,
}

impl Selector {
    pub fn as_str(&self) -> &str {
        &self.raw[..]
    }
}

impl Into<String> for Selector {
    fn into(self) -> String {
        self.raw
    }
}

impl FromStr for Selector {
    type Err = SelectorError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Selector::from_pairs(
            s,
            match Selector::parse(Rule::Main, s) {
                Ok(p) => p,
                Err(_) => return Err(SelectorError::NotSelector),
            },
        ))
    }
}

impl Selector {
    pub fn from_pairs<'p>(raw: &str, pairs: Pairs<'p, Rule>) -> Selector {
        fn pair_to_either(pair: Pair<Rule>) -> EiStr {
            match pair.as_rule() {
                Rule::Selector => Either::A(Selector::from_pairs(pair.as_str(), pair.into_inner())),
                Rule::Word => Either::B(pair.as_str().into()),
                _ => panic!("Unreachable"),
            }
        }
        let mut rules = Vec::new();
        for pair in pairs {
            use SelectRule::*;
            rules.push(match pair.as_rule() {
                Rule::All => All,
                Rule::None => None,
                Rule::This => This,
                Rule::Parent => Parent,
                Rule::Ancestors => Ancestors,
                Rule::Root => Root,
                Rule::Children => Children,
                Rule::Descendants => Descendants,
                Rule::Class => Class(pair.as_str().into()),
                Rule::Env => Env(pair_to_either(pair.into_inner().next().unwrap())),
                Rule::State => State(pair_to_either(pair.into_inner().next().unwrap())),
                Rule::Attr => {
                    let mut inner = pair.into_inner();
                    let key = pair_to_either(inner.next().unwrap());
                    let val = match inner.next() {
                        Some(v) => Some(pair_to_either(v)),
                        Option::None => Option::None,
                    };
                    Attr(key, val)
                }
                Rule::Order => {
                    let inner = pair.into_inner();
                    Order(Selector::from_pairs(inner.as_str(), inner))
                }
                Rule::Index => Index(pair.into_inner().next().unwrap().as_str().parse().unwrap()),
                Rule::EOI => continue,
                _ => panic!("Not Implemented: {:?}", pair.as_rule()),
            });
        }
        Selector {
            raw: raw.into(),
            rules,
        }
    }

    pub fn select<'t>(
        &'t self,
        tree: &'t Tree<Stone>,
        rule: usize,
        from: usize,
    ) -> (Vec<usize>, Vec<&'t str>) {
        // boy howdy
        use SelectRule::*;
        match &self.rules[rule] {
            All => {
                let (mut ind, mut st) = (Vec::new(), Vec::new());
                for leaf in tree[from].leaves() {
                    let (mut i, mut s) = self.select(tree, rule + 1, *leaf);
                    ind.append(&mut i);
                    st.append(&mut s);
                }
                (ind, st)
            }
            None => (Vec::new(), Vec::new()),
            This => self.select(tree, rule + 1, from),
            Parent => self.select(tree, rule + 1, tree[from].branch().unwrap()),
            Ancestors => {
                let (mut ind, mut st) = (Vec::new(), Vec::new());
                for leaf in tree.ancestors(from) {
                    let (mut i, mut s) = self.select(tree, rule + 1, leaf);
                    ind.append(&mut i);
                    st.append(&mut s);
                }
                (ind, st)
            }
            Children => {
                let (mut ind, mut st) = (Vec::new(), Vec::new());
                for leaf in tree[from].leaves() {
                    let (mut i, mut s) = self.select(tree, rule + 1, *leaf);
                    ind.append(&mut i);
                    st.append(&mut s);
                }
                (ind, st)
            }
            Descendants => {
                let (mut ind, mut st) = (Vec::new(), Vec::new());
                for leaf in tree.descendants(from) {
                    let (mut i, mut s) = self.select(tree, rule + 1, leaf);
                    ind.append(&mut i);
                    st.append(&mut s);
                }
                (ind, st)
            }
            Root => self.select(tree, rule + 1, 0),
            Class(s) => {
                let el = tree[from].value.as_el();
                if el.class == *s {
                    if rule == self.rules.len() - 1 {
                        (vec![from], vec![&el.class[..]])
                    } else {
                        self.select(tree, rule + 1, from)
                    }
                } else {
                    (Vec::new(), Vec::new())
                }
            }
            Order(osel) => {
                let (i, _) = osel.select(tree, 0, from);

                let (mut ind, mut st) = (Vec::new(), Vec::new());
                for leaf in i {
                    let (mut i, mut s) = self.select(tree, rule + 1, leaf);
                    ind.append(&mut i);
                    st.append(&mut s);
                }
                (ind, st)
            }
            Attr(ei, eq) => {
                let key = match ei {
                    Either::A(ksel) => ksel.select(tree, 0, from).1[0],
                    Either::B(k) => &k[..],
                };
                let el = tree[from].value.as_el();
                if el.attr.contains_key(&key[..]) {
                    let mut v = match &el[key] {
                        Attribute::String(s) => &s[..],
                        Attribute::Select(s) => s.select(tree, 0, from).1[0],
                    };
                    let sel = match eq {
                        Option::None => true,
                        Some(ei) => {
                            let val = match ei {
                                Either::B(v) => &v[..],
                                Either::A(vsel) => vsel.select(tree, 0, from).1[0],
                            };
                            let res = *v == *val;
                            v = val;
                            res
                        }
                    };
                    if sel {
                        if rule == self.rules.len() - 1 {
                            (vec![from], vec![v])
                        } else {
                            self.select(tree, rule + 1, from)
                        }
                    } else {
                        (Vec::new(), Vec::new())
                    }
                } else {
                    (Vec::new(), Vec::new())
                }
            }
            Slide(sel) => {
                let (mut li, mut ls) = sel.select(tree, 0, from);
                if !li.is_empty() {
                    (li, ls)
                } else {
                    self.select(tree, rule + 1, from)
                }
            }
            Or(sel) => {
                let (mut li, mut ls) = sel.select(tree, 0, from);
                let (mut ri, mut rs) = self.select(tree, rule + 1, from);
                li.append(&mut ri);
                ls.append(&mut rs);
                (li, ls)
            }
            // TODO - IfElse, Index, State, Env
            _ => panic!("Not Implemented"),
        }
    }

    pub fn subsumes(&self, _other: &Selector) -> bool {
        panic!("Not Implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pest::iterators::Pair;
    use pest::*;

    const TEST_SEL: &str = "$test[x=$(x)] / *";

    #[test]
    fn make_pairs() {
        println!("Testing selector parsing for: {}", TEST_SEL);
        let pairs = match Selector::parse(Rule::Main, TEST_SEL) {
            Ok(p) => p,
            // Err(ParsingError {
            //     positives,
            //     negatives,
            //     pos,
            // }) => {
            //     println!("[ERROR]");
            //     println!("{}", selector);
            //     for _i in 0..pos.pos() {
            //         print!(" ")
            //     }
            //     println!("^");
            //     println!("+ :: {:?}", positives);
            //     println!("- :: {:?}", negatives);
            //     return;
            // }
            Err(e) => panic!("{}", e),
        };
        for pair in pairs {
            print_pair(&pair, 0)
        }
    }

    #[test]
    fn make_selector() {
        println!("Testing selector parsing for: {}", TEST_SEL);
        let sel: Selector = TEST_SEL.parse().unwrap();
        println!("{:?}", sel);
    }

    fn print_pair(pair: &Pair<Rule>, depth: u32) {
        fn tab(depth: u32) -> String {
            let mut res = String::new();
            for _i in 0..depth {
                res.push_str("|   ");
            }
            res
        }
        println!(
            "{}{:?} :: {}",
            tab(depth),
            pair.as_rule(),
            pair.clone().into_span().as_str()
        );
        //println!("{}Span: {:?}", tab(depth, false), pair.clone().into_span());
        for pair in pair.clone().into_inner() {
            print_pair(&pair, depth + 1);
        }
    }
}
