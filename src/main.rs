use std::fs::read_to_string;

#[derive(Debug)]
enum TokenType {
    Word,
    Value,
    Ocurly,
    Ccurly,
    Colon,
}

#[derive(Debug)]
struct Token {
    token_type: TokenType,
    value: String,
}

fn parse_word(input: &mut String) -> Token {
    let mut name: String = chop_character(input).to_string();
    while !input.is_empty() {
        if input.starts_with(|c: char| c == '_' || c.is_ascii_alphanumeric()) {
            name.push(chop_character(input));
        } else {
            break;
        }
    }
    Token {
        token_type: TokenType::Word,
        value: name,
    }
}

fn parse_digit(input: &mut String) -> Token {
    let mut already_decimal: bool = false;
    let mut already_exp: bool = false;
    let mut name: String = chop_character(input).to_string();
    while !input.is_empty() {
        if input.starts_with(|c: char| c.is_ascii_digit()) {
            name.push(chop_character(input));
        } else if input.starts_with('e') {
            if already_exp {
                panic!("Attempt to add 'e' to a digit twice");
            }
            already_exp = true;
            name.push(chop_character(input));
        } else if input.starts_with('.') {
            if already_decimal {
                panic!("Attempt to add a second decimal point to a digit");
            }
            already_decimal = true;
            name.push(chop_character(input));
        } else {
            break;
        }
    }
    Token {
        token_type: TokenType::Value,
        value: name,
    }
}

fn chop_character(input: &mut String) -> char {
    input.remove(0)
}

fn tokenize_file_contents(contents: &mut String) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];
    while !contents.is_empty() {
        if contents.starts_with(|c: char| c.is_whitespace()) {
            chop_character(contents);
            continue;
        } else if contents.starts_with(|c: char| c.is_ascii_alphabetic()) {
            tokens.push(parse_word(contents));
        } else if contents.starts_with(|c: char| c.is_ascii_digit()) {
            tokens.push(parse_digit(contents));
        } else if contents.starts_with('{') {
            chop_character(contents);
            tokens.push(Token {
                token_type: TokenType::Ocurly,
                value: "{".to_string(),
            });
        } else if contents.starts_with('}') {
            chop_character(contents);
            tokens.push(Token {
                token_type: TokenType::Ccurly,
                value: "}".to_string(),
            });
        } else if contents.starts_with(':') {
            chop_character(contents);
            tokens.push(Token {
                token_type: TokenType::Colon,
                value: ":".to_string(),
            });
        } else {
            let chr = chop_character(contents);
            eprintln!("Unknown character: {}", chr);
        }
    }
    tokens
}

fn main() {
    let filename = "acc_defn.lotr";
    let mut contents = read_to_string(filename).expect("Could not read file.");

    let tokens = tokenize_file_contents(&mut contents);
    println!("{:?}", tokens);
}
