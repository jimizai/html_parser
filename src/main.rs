use std::fmt;
use std::fs;

#[derive(Debug, Clone)]
struct NodeTree {
    tag: String,
    index: usize,
    children: Vec<Box<NodeTree>>,
}

impl NodeTree {
    pub fn new(tag: String, children: Vec<Box<NodeTree>>, index: usize) -> Self {
        NodeTree {
            tag,
            index,
            children,
        }
    }
}

impl fmt::Display for NodeTree {
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
    position: usize,
    end: usize,
}

#[derive(Debug, Clone)]
enum Tokenizer {
    Tag,
    EndTag,
    Text,
    Attribute,
}

#[derive(Debug, Clone)]
struct Token {
    tokenizer: Tokenizer,
    value: String,
}

impl Token {
    fn new(value: String, is_tag: bool, is_end: bool, is_attribute: bool) -> Token {
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
    fn new(bytes: &[u8]) -> Scanner {
        Scanner {
            bytes,
            position: 0,
            end: bytes.len(),
        }
    }

    fn parse(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        let mut v: Vec<u8> = Vec::new();
        let mut is_tag = false;
        let mut is_end = false;
        let mut is_attribute = false;
        while self.end > self.position {
            let byte = self.bytes[self.position];
            match byte {
                b'<' => {
                    if v.len() != 0 {
                        let text = std::str::from_utf8(&v).unwrap();
                        tokens.push(Token::new(text.to_string(), is_tag, is_end, is_attribute));
                        v.clear()
                    }
                    is_tag = true;
                }
                b'>' => {
                    let text = std::str::from_utf8(&v).unwrap();
                    tokens.push(Token::new(text.to_string(), is_tag, is_end, is_attribute));
                    v.clear();
                    is_tag = false;
                    is_end = false;
                    is_attribute = false;
                }
                b'/' => {
                    if is_tag {
                        is_end = true
                    }
                }
                b' ' => {
                    if is_tag {
                        let text = std::str::from_utf8(&v).unwrap();
                        tokens.push(Token::new(text.to_string(), is_tag, is_end, is_attribute));
                        is_attribute = true;
                        v.clear();
                    }
                }
                b'\n' | b'\r' => {}
                _ => v.push(byte),
            }
            self.position += 1;
        }
        tokens
    }

    fn lexer(&mut self, tokens: Vec<Token>) {
        let mut stack: Vec<Token> = Vec::new();
        let mut trees: Vec<NodeTree> = Vec::new();
        let mut node_tree = NodeTree::new(String::new(), Vec::new(), 0);
        for token in tokens {
            println!("{:?}", token);
            match token.tokenizer {
                Tokenizer::Tag => {
                    stack.push(token.clone());
                    trees.push(NodeTree::new(token.value, Vec::new(), stack.len()))
                }
                Tokenizer::EndTag => {
                    let data = stack.pop();
                    if let Some(_) = data {
                        let tree = trees.pop();
                        if let Some(tree) = tree {
                            if node_tree.index == 0 {
                                node_tree.index = tree.index - 1;
                            }
                            if node_tree.index == (tree.index - 1) {
                                node_tree.children.push(Box::new(tree));
                            } else {
                                node_tree.tag = tree.tag;
                                node_tree = NodeTree::new(
                                    String::from("div"),
                                    vec![Box::new(node_tree)],
                                    tree.index - 1,
                                );
                            }
                        }
                    } else {
                        eprintln!("error parse token end tag {}", token.value);
                    }
                }
                _ => {
                    let length = trees.len() - 1;
                    let tree = &mut trees[length];
                    tree.children.push(Box::new(NodeTree::new(
                        token.value,
                        Vec::new(),
                        stack.len(),
                    )));
                }
            }
        }
        println!("{}", node_tree);
    }
}

fn main() {
    let text = fs::read_to_string("index.html").unwrap();
    let mut scanner = Scanner::new(text.as_bytes());
    let tokens = scanner.parse();
    scanner.lexer(tokens);
}
