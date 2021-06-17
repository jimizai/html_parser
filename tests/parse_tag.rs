use html_parser::{Scanner, Token, Tokenizer::*};

#[test]
fn single_tag_close() {
    let text = b" <div /> ";
    let mut scanner = Scanner::new(text);
    let tokens = scanner.parse();
    assert_eq!(
        *tokens.get(0).unwrap(),
        Token::new("div", true, false, false)
    );
    assert_eq!(*tokens.get(1).unwrap(), Token::new("", true, true, false))
}

#[test]
fn loose_tag_close() {
    let text = b" <   div  /    > ";
    let mut scanner = Scanner::new(text);
    let tokens = scanner.parse();
    println!("{:?}", tokens);
    assert_eq!(
        *tokens.get(0).unwrap(),
        Token::new("div", true, false, false)
    );
    assert_eq!(*tokens.get(1).unwrap(), Token::new("", true, true, false));
}

#[test]
fn loose_tag() {
    let text = r#"<a href="https://esdoc.org"
        >ESDoc<span data-ice="esdocVersion">(0.4.8)</span></a
    >"#;
    let mut scanner = Scanner::new(text.as_bytes());
    let mut tokens = scanner.parse();
    let mut result = [
        Token {
            tokenizer: Tag,
            value: "a",
        },
        Token {
            tokenizer: Attribute,
            value: "href=\"https://esdoc.org\"",
        },
        Token {
            tokenizer: Text,
            value: "ESDoc",
        },
        Token {
            tokenizer: Tag,
            value: "span",
        },
        Token {
            tokenizer: Attribute,
            value: "data-ice=\"esdocVersion\"",
        },
        Token {
            tokenizer: Text,
            value: "(0.4.8)",
        },
        Token {
            tokenizer: EndTag,
            value: "span",
        },
        Token {
            tokenizer: EndTag,
            value: "a",
        },
    ]
    .to_vec();

    for _ in 0..tokens.len() {
        assert_eq!(tokens.pop().unwrap(), result.pop().unwrap())
    }
}
