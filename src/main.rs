use std::collections::HashMap;
use std::fmt;
use std::fs;

#[derive(Debug, Clone)]
struct NodeTree<'a> {
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
struct Scanner<'a> {
    bytes: &'a [u8],
    start_position: usize,
    position: usize,
    end: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Tokenizer {
    Tag,
    EndTag,
    Text,
    Attribute,
}

#[derive(Debug, Copy, Clone)]
struct Token<'a> {
    tokenizer: Tokenizer,
    value: &'a str,
}

impl<'a> Token<'a> {
    fn new(value: &'a str, is_tag: bool, is_end: bool, is_attribute: bool) -> Token {
        let tokenizer = match is_tag {
            true => match is_end {
                true => Tokenizer::EndTag,
                false => match is_attribute {
                    false => Tokenizer::Tag,
                    true => Tokenizer::Attribute,
                },
            },
            _ => Tokenizer::Text,
        };
        Token { tokenizer, value }
    }
}

impl<'a> Scanner<'a> {
    fn new(bytes: &'a [u8]) -> Scanner<'a> {
        Scanner {
            bytes,
            start_position: 0,
            position: 0,
            end: bytes.len(),
        }
    }

    fn parse(&mut self) -> Vec<Token<'a>> {
        let mut tokens: Vec<Token<'a>> = Vec::new();
        let mut is_tag = false;
        let mut is_end = false;
        let mut is_attribute = false;
        let mut is_string = false;
        while self.end > self.position {
            let byte = self.bytes[self.position];
            match byte {
                b'<' => {
                    if self.start_position != 0 {
                        let text = std::str::from_utf8(self.get_bytes()).unwrap();
                        tokens.push(Token::new(text, is_tag, is_end, is_attribute));
                        self.start_position = 0;
                    }
                    is_tag = true;
                }
                b'>' => {
                    let text = std::str::from_utf8(self.get_bytes()).unwrap();
                    println!("==={}", text);
                    tokens.push(Token::new(text, is_tag, is_end, is_attribute));
                    self.start_position = 0;
                    is_tag = false;
                    is_end = false;
                    is_attribute = false;
                }
                b'/' => {
                    if is_tag {
                        is_end = true;
                    }
                    self.start_position = self.position + 1;
                }
                b' ' => {
                    if is_string == true {
                        self.position += 1;
                        continue;
                    }
                    if is_tag {
                        let text = std::str::from_utf8(self.get_bytes()).unwrap();
                        tokens.push(Token::new(text, is_tag, is_end, is_attribute));
                        is_attribute = true;
                        self.start_position = 0;
                    }
                }
                b'\n' | b'\r' => {}
                b'"' => {
                    if self.start_position == 0 && !is_string {
                        self.start_position = self.position;
                    }
                    is_string = !is_string
                }
                _ => {
                    if self.start_position == 0 {
                        self.start_position = self.position;
                    }
                }
            }
            self.position += 1;
        }
        tokens
    }

    fn lexer(&mut self, tokens: Vec<Token<'a>>) -> NodeTree<'a> {
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
            }
        }
        node_tree
    }

    fn get_bytes(&self) -> &'a [u8] {
        &self.bytes[self.start_position..self.position]
    }
}

fn main() {
    let text = fs::read_to_string("index.html").unwrap();
    let mut scanner = Scanner::new(text.as_bytes());
    let tokens = scanner.parse();
    let node_tree = scanner.lexer(tokens);
    println!("{:?}", node_tree);
}
