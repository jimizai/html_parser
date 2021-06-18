use html_parser::{status::Flags, Scanner, Token, Tokenizer::*};

fn assert_eq_tokens(mut target: Vec<Token>, mut result: Vec<Token>) {
    for _ in 0..target.len() {
        assert_eq!(target.pop().unwrap(), result.pop().unwrap())
    }
}

#[test]
fn single_tag_close() {
    let text = b" <div /> ";
    let mut scanner = Scanner::new(text);
    let tokens = scanner.parse();
    assert_eq!(*tokens.get(0).unwrap(), Token::new("div", Flags::IS_TAG));
    assert_eq!(
        *tokens.get(1).unwrap(),
        Token::new("", Flags::IS_TAG | Flags::IS_TAG_END)
    )
}

#[test]
fn loose_tag_close() {
    let text = b" <   div  /    > ";
    let mut scanner = Scanner::new(text);
    let tokens = scanner.parse();
    assert_eq!(*tokens.get(0).unwrap(), Token::new("div", Flags::IS_TAG));
    assert_eq!(
        *tokens.get(1).unwrap(),
        Token::new("", Flags::IS_TAG | Flags::IS_TAG_END)
    );
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

#[test]
fn tag_annotations() {
    let text = r#"<li name="attr tets" test="sda">test</li>
    <!-- 
      <li>test</li>
     -->
    <li>test</li>"#;

    let result = [
        Token {
            tokenizer: Tag,
            value: "li",
        },
        Token {
            tokenizer: Attribute,
            value: "name=\"attr tets\"",
        },
        Token {
            tokenizer: Attribute,
            value: "test=\"sda\"",
        },
        Token {
            tokenizer: Text,
            value: "test",
        },
        Token {
            tokenizer: EndTag,
            value: "li",
        },
        Token {
            tokenizer: Annocation,
            value: " \n      <li>test</li>\n     ",
        },
        Token {
            tokenizer: Tag,
            value: "li",
        },
        Token {
            tokenizer: Text,
            value: "test",
        },
        Token {
            tokenizer: EndTag,
            value: "li",
        },
    ];

    let mut scanner = Scanner::new(text.as_bytes());
    let tokens = scanner.parse();
    assert_eq_tokens(tokens, result.to_vec());
}
