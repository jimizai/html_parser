use html_parser::Scanner;
use std::fs;

fn main() {
    let text = fs::read_to_string("index.html").unwrap();
    let mut scanner = Scanner::new(text.as_bytes());
    let data = scanner.parse();
    println!("{:?}", data);
}
