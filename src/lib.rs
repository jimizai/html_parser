use std::collections::HashMap;
use std::fmt;

pub mod status;

use status::Flags;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeTree<'a> {
    tag: &'a str,
    index: usize,
    children: Vec<Box<NodeTree<'a>>>,
    attributes: HashMap<&'a str, &'a str>,
    text: &'a str,
}

impl<'a> NodeTree<'a> {
    pub fn new(tag: &'a str, children: Vec<Box<NodeTree<'a>>>, index: usize) -> Self {
        NodeTree {
            tag,
            index,
            children,
            attributes: HashMap::new(),
            text: "",
        }
    }

    pub fn set_attributes(&mut self, key: &'a str, value: &'a str) {
        self.attributes.insert(key, value);
    }

    pub fn set_text(&mut self, text: &'a str) {
        self.text = text;
    }

    pub fn is_empty(&self) -> bool {
        self.children.len() == 0 && self.attributes.is_empty() && self.text.is_empty()
    }
}

impl<'a> fmt::Display for NodeTree<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ tag: {}, index: {}, children: {:?} }}",
            self.tag, self.index, self.children
        )
    }
}

#[derive(Debug)]
pub struct Scanner<'a> {
    bytes: &'a [u8],
    start_position: usize,
    position: usize,
    end: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Tokenizer {
    Tag,
    EndTag,
    Text,
    Attribute,
    Annocation,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Token<'a> {
    pub tokenizer: Tokenizer,
    pub value: &'a str,
}

impl<'a> Token<'a> {
    pub fn new(value: &'a str, status: Flags) -> Token {
        let tokenizer = if status.contains(Flags::IS_ANNOTATION) {
            Tokenizer::Annocation
        } else if status.contains(Flags::IS_TAG_END) {
            Tokenizer::EndTag
        } else if status.contains(Flags::IS_ATTRIBUTE) {
            Tokenizer::Attribute
        } else if status.contains(Flags::IS_TAG) {
            Tokenizer::Tag
        } else {
            Tokenizer::Text
        };

        Token { tokenizer, value }
    }
}

impl<'a> Scanner<'a> {
    pub fn new(bytes: &'a [u8]) -> Scanner<'a> {
        Scanner {
            bytes,
            start_position: 0,
            position: 0,
            end: bytes.len(),
        }
    }

    pub fn parse(&mut self) -> Vec<Token<'a>> {
        let mut tokens: Vec<Token<'a>> = Vec::new();
        let mut status = Flags::NONE;

        while self.end > self.position {
            let byte = self.bytes[self.position];
            if status.contains(Flags::IS_ANNOTATION) {
                match byte {
                    b'-' => {
                        let next_byte = self.bytes[self.position + 1];
                        let third_byte = self.bytes[self.position + 2];
                        if next_byte == b'-' && third_byte == b'>' {
                            let text = std::str::from_utf8(self.get_bytes()).unwrap();
                            tokens.push(Token::new(text, status));
                            self.start_position = 0;
                            status -= Flags::IS_ANNOTATION;
                            self.position += 2;
                            continue;
                        }
                    }
                    _ => {}
                }
            } else {
                match byte {
                    b'<' => {
                        if self.start_position != 0 && self.position != self.start_position {
                            let text = std::str::from_utf8(self.get_bytes()).unwrap();
                            tokens.push(Token::new(text, status));
                            self.start_position = 0;
                        }
                        status |= Flags::IS_TAG;
                        status -= Flags::IGNORE_ONCE;
                    }
                    b'!' => {
                        if status.contains(Flags::IS_TAG) {
                            let next_byte = self.bytes[self.position + 1];
                            let third_byte = self.bytes[self.position + 2];
                            if next_byte == b'-' && third_byte == b'-' {
                                self.start_position = self.position + 3;
                                status.clear();
                                status |= Flags::IS_ANNOTATION;
                            }
                        }
                    }
                    b'>' => {
                        if self.start_position != 0 {
                            let text = std::str::from_utf8(self.get_bytes()).unwrap();
                            tokens.push(Token::new(text, status));
                            self.start_position = 0;
                        } else if status.contains(Flags::IS_TAG_END) {
                            if !status.contains(Flags::IGNORE_ONCE) {
                                tokens.push(Token::new("", status));
                            } else {
                                status -= Flags::IGNORE_ONCE;
                            }
                        }
                        status -= Flags::IS_TAG;
                        status -= Flags::IS_TAG_END;
                        status -= Flags::IS_ATTRIBUTE;
                    }
                    b'/' => {
                        if status.contains(Flags::IS_STRING) {
                            self.position += 1;
                            continue;
                        }
                        if status.contains(Flags::IS_TAG) {
                            status |= Flags::IS_TAG_END;
                        }
                        self.start_position = self.position + 1;
                    }
                    b'\n' | b'\r' | b' ' => {
                        if status.contains(Flags::IS_STRING) {
                            self.position += 1;
                            continue;
                        }
                        if !status.contains(Flags::HAS_TEXT) {
                            self.position += 1;
                            self.start_position = 0;
                            continue;
                        } else if status.contains(Flags::IS_TAG) {
                            if status.contains(Flags::IS_TAG_END) {
                                status |= Flags::IGNORE_ONCE;
                            }
                            let text = std::str::from_utf8(self.get_bytes()).unwrap();
                            tokens.push(Token::new(text, status));
                            status |= Flags::IS_ATTRIBUTE;
                            status -= Flags::HAS_TEXT;
                            self.start_position = 0;
                        }
                    }
                    b'"' => {
                        if self.start_position == 0 && !status.contains(Flags::IS_STRING) {
                            self.start_position = self.position;
                        }
                        status.toggle(Flags::IS_STRING);
                    }
                    _ => {
                        if self.start_position == 0 {
                            status |= Flags::HAS_TEXT;
                            self.start_position = self.position;
                        }
                    }
                }
            }
            self.position += 1;
        }
        tokens
    }

    pub fn lexer(&mut self, tokens: Vec<Token<'a>>) -> NodeTree<'a> {
        let mut stack: Vec<Token> = Vec::new();
        let mut trees: Vec<NodeTree> = Vec::new();
        let mut node_tree = NodeTree::new("", Vec::new(), 0);
        for token in tokens {
            match token.tokenizer {
                Tokenizer::Tag => {
                    stack.push(token.clone());
                    let tree = NodeTree::new(&token.value, Vec::new(), stack.len());
                    trees.push(tree);
                }
                Tokenizer::EndTag => {
                    let data = stack.pop();
                    if data.is_none() {
                        eprintln!("error parse token end tag {}", token.value);
                        continue;
                    }
                    let tree = trees.pop();
                    if let Some(tree) = tree {
                        // init
                        if node_tree.index == 0 {
                            node_tree.index = tree.index - 1;
                        }
                        // same tier
                        if node_tree.index == (tree.index - 1) {
                            node_tree.children.push(Box::new(tree));
                        } else {
                            node_tree.tag = tree.tag;
                            node_tree.children.extend(tree.children);
                            node_tree.attributes = tree.attributes;
                            node_tree =
                                NodeTree::new("div", vec![Box::new(node_tree)], tree.index - 1);
                        }
                    }
                }
                Tokenizer::Attribute => {
                    let tokens: Vec<&str> = token.value.split('=').collect();
                    trees
                        .last_mut()
                        .unwrap()
                        .set_attributes(tokens.get(0).unwrap_or(&""), tokens.get(1).unwrap_or(&""));
                }
                Tokenizer::Text => trees.last_mut().unwrap().set_text(token.value),
                _ => {}
            }
        }
        node_tree
    }

    fn get_bytes(&self) -> &'a [u8] {
        &self.bytes[self.start_position..self.position]
    }
}
