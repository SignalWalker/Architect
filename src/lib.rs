#![feature(associated_type_defaults)]

pub extern crate birch;
pub extern crate pest;
extern crate quick_xml;
//extern crate serde;
#[macro_use]
pub extern crate pest_derive;

use birch::*;
use quick_xml::{events::*, Reader};
use select::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::io::BufRead;
use std::ops::{BitAnd, Index};
use std::str::FromStr;

pub mod import;
pub mod select;

#[derive(Debug)]
pub enum StoneError {
    XML(quick_xml::Error),
    InvalidRoot,
    MissingAttr(String),
    InvalidAttr(String),
    Selector(SelectorError),
    Duplicate,
}

pub trait FromAttr: Sized {
    type Err;
    fn from_attr(&Attribute, &Tree<Stone>, usize) -> Result<Self, Self::Err>;
}

impl<S: FromStr> FromAttr for S
where
    S::Err: Debug,
{
    type Err = S::Err;
    fn from_attr(attr: &Attribute, tree: &Tree<Stone>, from: usize) -> Result<Self, Self::Err> {
        match attr {
            Attribute::String(s) => s.parse(),
            Attribute::Select(s) => s.select(tree, 0, from).1[0].parse(),
        }
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub enum Attribute {
    String(String),
    Select(Selector),
}

impl Attribute {
    pub fn parse<A: FromAttr>(&self, tree: &Tree<Stone>, from: usize) -> Result<A, A::Err> {
        A::from_attr(self, tree, from)
    }

    pub fn parse_or<A: FromAttr>(&self, tree: &Tree<Stone>, from: usize, def: A) -> A {
        match A::from_attr(self, tree, from) {
            Ok(a) => a,
            Err(_) => def,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Attribute::String(s) => &s[..],
            Attribute::Select(s) => s.as_str(),
        }
    }
}

impl Into<String> for Attribute {
    fn into(self) -> String {
        match self {
            Attribute::String(s) => s,
            Attribute::Select(s) => s.into(),
        }
    }
}

impl FromStr for Attribute {
    type Err = StoneError;
    fn from_str(s: &str) -> Result<Attribute, Self::Err> {
        match Selector::from_str(s) {
            Ok(s) => Ok(Attribute::Select(s)),
            Err(SelectorError::NotSelector) => Ok(Attribute::String(s.into())),
            Err(SelectorError::Parse) => Err(StoneError::Selector(SelectorError::Parse)),
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct Element {
    pub class: String,
    pub attr: HashMap<String, Attribute>,
}

impl Element {
    pub fn new(class: String) -> Element {
        Element {
            class,
            attr: HashMap::new(),
        }
    }

    pub fn from_xml_start<T: BufRead>(
        ev: &BytesStart,
        read: &Reader<T>,
    ) -> Result<Element, StoneError> {
        let class = read.decode(ev.name()).to_string();
        let mut res = Element::new(class);
        for attr in ev.attributes() {
            match attr {
                Ok(attr) => {
                    let key = read.decode(attr.key).to_string();
                    let value = match attr.unescape_and_decode_value(&read) {
                        Ok(s) => s,
                        Err(err) => return Err(StoneError::XML(err)),
                    };
                    res.attr.insert(
                        key,
                        match value.parse() {
                            Ok(v) => v,
                            Err(e) => panic!("Attr Parse: {:?}", e),
                        },
                    );
                }
                Err(err) => return Err(StoneError::XML(err)),
            }
        }
        Ok(res)
    }

    pub fn parse_attr_or<D: FromAttr>(
        &self,
        tree: &Tree<Stone>,
        from: usize,
        a: &str,
        def: D,
    ) -> D {
        if self.attr.contains_key(a) {
            if let Ok(a) = self[a].parse(tree, from) {
                return a;
            }
        }
        def
    }

    pub fn to_xml(&self, branch: bool) -> (String, String) {
        let mut attr = String::new();
        for (k, v) in &self.attr {
            attr += &format!("{}='{:?}'", k, v);
        }
        let open = format!(
            "<{} {}{}>",
            self.class,
            {
                let mut attr = String::new();
                for (k, v) in &self.attr {
                    attr.push_str(&format!("{}=\"{}\" ", k, v.as_str()))
                }
                attr
            },
            if branch { "" } else { "/" }
        );
        let close = format!("</{}>", self.class);
        (open, close)
    }
}

impl Debug for Element {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let mut res = format!("<{} ", self.class);
        for (a, v) in &self.attr {
            res += &format!("{}='{:?}' ", a, v);
        }
        res += "/>";
        write!(f, "{}", res);
        Ok(())
    }
}

impl BitAnd for Element {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        let mut res = Element::new(rhs.class.clone());
        for (k, v) in self.attr.iter().chain(rhs.attr.iter()) {
            res.attr.insert(k.clone(), v.clone());
        }
        res
    }
}

impl<'a> Index<&'a str> for Element {
    type Output = Attribute;
    fn index(&self, index: &str) -> &Attribute {
        &self.attr[index]
    }
}

pub fn str_to_bool(s: &str) -> bool {
    match s.to_lowercase().as_str() {
        "yes" => true,
        "no" => false,
        "true" => true,
        "false" => false,
        _ => panic!("Invalid Bool"),
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Stone {
    Element(Element),
    Text(String),
}

impl Stone {
    pub fn from_xml_text<T: BufRead>(
        ev: &BytesText,
        read: &Reader<T>,
    ) -> Result<Stone, StoneError> {
        match ev.unescape_and_decode(read) {
            Ok(res) => Ok(Stone::Text(res)),
            Err(err) => Err(StoneError::XML(err)),
        }
    }

    pub fn as_el(&self) -> &Element {
        match self {
            Stone::Element(el) => &el,
            _ => panic!("Tried to cast Stone::Text as Element"),
        }
    }
}

impl From<Element> for Stone {
    fn from(el: Element) -> Stone {
        Stone::Element(el)
    }
}

impl From<String> for Stone {
    fn from(s: String) -> Stone {
        Stone::Text(s)
    }
}

pub trait StoneMason {
    fn handle_stones(
        &mut self,
        &mut Architect,
        &mut HashMap<String, Vec<usize>>,
    ) -> HashSet<usize> {
        HashSet::new()
    }
}

pub struct Architect {
    pub stones: Tree<Stone>,
    pub errors: Vec<StoneError>,
}

impl Architect {
    pub fn from_root(root: Element) -> Architect {
        Architect::from_tree(Tree::with_root(Stone::Element(root)))
    }

    pub fn from_tree(stones: Tree<Stone>) -> Architect {
        Architect {
            stones,
            errors: Vec::new(),
        }
    }

    pub fn from_buffer<B: BufRead>(
        buf: B,
    ) -> Result<(Architect, HashMap<String, Vec<usize>>), StoneError> {
        let mut reader = Reader::from_reader(buf);
        let root = {
            reader.trim_text(true);
            let mut buf = Vec::new();
            let root = match reader.read_event(&mut buf) {
                Ok(Event::Start(ref ev)) => Element::from_xml_start(&ev, &reader)?,
                Err(e) => return Err(StoneError::XML(e)),
                _ => return Err(StoneError::InvalidRoot),
            };
            buf.clear();
            root
        };

        let mut arch = Architect::from_root(root);
        let mut map = arch.decode_reader(&mut reader, 0, false);
        map.entry(arch.stones[0].value.as_el().class.clone())
            .or_insert_with(Vec::new)
            .push(0);

        Ok((arch, map))
    }

    pub fn decode_buffer<B: BufRead>(
        &mut self,
        reader: B,
        branch: usize,
        replace: bool,
    ) -> HashMap<String, Vec<usize>> {
        let mut reader = Reader::from_reader(reader);
        reader.trim_text(true);
        self.decode_reader(&mut reader, branch, replace)
    }

    pub fn decode_reader<B: BufRead>(
        &mut self,
        reader: &mut Reader<B>,
        branch: usize,
        mut replace: bool,
    ) -> HashMap<String, Vec<usize>> {
        let mut buf = Vec::new();
        let mut res: HashMap<String, Vec<usize>> = HashMap::new();
        let mut branches = vec![branch];

        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref ev)) => match Element::from_xml_start(&ev, reader) {
                    Ok(el) => {
                        res.entry(el.class.clone())
                            .or_insert_with(Vec::new)
                            .push(if replace {
                                replace = false;
                                self.stones[branch].value = el.into();
                                branch
                            } else {
                                let index = self.stones.size();
                                self.stones.push(*branches.last().unwrap(), el.into());
                                branches.push(index);
                                index
                            })
                    }
                    Err(err) => self.errors.push(err),
                },
                // "Empty" means an element without leaves
                Ok(Event::Empty(ref ev)) => match Element::from_xml_start(&ev, reader) {
                    Ok(el) => {
                        res.entry(el.class.clone())
                            .or_insert_with(Vec::new)
                            .push(if replace {
                                replace = false;
                                self.stones[branch].value = el.into();
                                branch
                            } else {
                                self.stones.push(*branches.last().unwrap(), el.into());
                                self.stones.size() - 1
                            })
                    }
                    Err(err) => self.errors.push(err),
                },
                Ok(Event::Text(ref e)) => match e.unescape_and_decode(&reader) {
                    Ok(text) => self.stones.push(*branches.last().unwrap(), text.into()),
                    Err(err) => self.errors.push(StoneError::XML(err)),
                },
                Ok(Event::End(ref _e)) => {
                    branches.pop();
                }
                Ok(Event::Eof) => break,
                Err(e) => self.errors.push(StoneError::XML(e)),
                _ => (),
            }
            buf.clear();
        }

        res
    }

    pub fn to_xml(&self) -> String {
        fn tab(depth: usize) -> String {
            let mut res = String::new();
            for _i in 0..depth {
                res += "   ";
            }
            res
        }
        fn recurse(arch: &Architect, branch: usize, depth: usize) -> String {
            let node = &arch.stones[branch];
            match &node.value {
                Stone::Element(el) => {
                    let leaves = node.leaves();
                    if leaves.is_empty() {
                        tab(depth) + &el.to_xml(false).0
                    } else {
                        let (mut open, close) = el.to_xml(true);
                        open = tab(depth) + &open;
                        for leaf in leaves {
                            open += "\n";
                            open += &recurse(arch, *leaf, depth + 1);
                        }
                        open += "\n";
                        open + &tab(depth) + &close
                    }
                }
                Stone::Text(t) => tab(depth) + t,
            }
        }
        recurse(self, 0, 0)
    }
}
