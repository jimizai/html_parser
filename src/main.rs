use html_parser::Scanner;
use std::fs;

fn main() {
    let text = fs::read_to_string("test.html").unwrap();
    let mut scanner = Scanner::new(text.as_bytes());
    let data = scanner.parse();
    let node_tree = scanner.lexer(data);
    println!("{:?}", node_tree);
}
