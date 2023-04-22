// use crate::beam::{gamma_2_beta, ke_2_gamma};
use crate::elements::*;
use crate::parse_lotr::Simulation;
use crate::elegant_rpn::RpnCalculator;
use ndarray::Array2;
use std::fmt;
use std::fs::read_to_string;
use std::process::exit;

pub fn load_elegant_file(filename: &str) -> Simulation {
    let tokens = tokenize_file_contents(filename);
    parse_tokens(&tokens)
}

#[derive(Debug)]
struct FileLoc {
    filename: String,
    row: usize,
    col: usize,
}

#[derive(Debug, PartialEq)]
enum TokenType {
    Word,
    Value,
    Ocurly,
    Ccurly,
    Oparen,
    Cparen,
    Comma,
    Assign,
    Colon,
    EleStr,
    RpnExpr,
    LineJoin,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
struct Token {
    token_type: TokenType,
    value: String,
    loc: FileLoc,
}

fn parse_word(input: &mut String, loc: FileLoc) -> Token {
    let mut name: String = chop_character(input).to_string();
    while !input.is_empty() {
        if input
            .starts_with(|c: char| c == '_' || c.is_ascii_alphanumeric() || c == '.' || c == '$')
        {
            name.push(chop_character(input));
        } else {
            break;
        }
    }
    Token {
        token_type: TokenType::Word,
        value: name,
        loc,
    }
}

fn parse_string(input: &mut String, loc: FileLoc) -> Token {
    let mut name: String = chop_character(input).to_string();
    while !input.is_empty() {
        if input.starts_with(|c: char| c == '"') {
            break;
        } else {
            name.push(chop_character(input));
        }
    }
    chop_character(input);
    Token {
        token_type: TokenType::EleStr,
        value: name,
        loc,
    }
}

fn parse_rpn_expr(input: &mut String, mut loc: FileLoc) -> Token {
    while !input.is_empty() && input.starts_with(|c: char| c.is_whitespace()) {
        chop_character(input);
        loc.col += 1;
    }
    let mut name: String = chop_character(input).to_string();
    while !input.is_empty() {
        if input.starts_with(|c: char| c == '\n') {
            break;
        } else {
            name.push(chop_character(input));
        }
    }
    chop_character(input);
    Token {
        token_type: TokenType::RpnExpr,
        value: name,
        loc,
    }
}

fn parse_digit(input: &mut String, loc: FileLoc) -> Token {
    let mut already_decimal: bool = false;
    let mut already_exp: bool = false;
    let mut value: String = chop_character(input).to_string();
    while !input.is_empty() {
        if input.starts_with(|c: char| c.is_ascii_digit()) {
            value.push(chop_character(input));
        } else if input.starts_with("e-") {
            if already_exp {
                panic!("Attempt to add 'e' to a digit twice");
            }
            already_exp = true;
            value.push(chop_character(input));
            value.push(chop_character(input));
        } else if input.starts_with('e') {
            if already_exp {
                panic!("Attempt to add 'e' to a digit twice");
            }
            already_exp = true;
            value.push(chop_character(input));
        } else if input.starts_with('.') {
            if already_decimal {
                panic!("Attempt to add a second decimal point to a digit");
            }
            already_decimal = true;
            value.push(chop_character(input));
        } else {
            break;
        }
    }
    Token {
        token_type: TokenType::Value,
        value,
        loc,
    }
}

fn chop_character(input: &mut String) -> char {
    input.remove(0)
}

fn tokenize_file_contents(filename: &str) -> Vec<Token> {
    let mut contents = match read_to_string(filename) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("{}", e);
            eprintln!("Could not open file: '{}'", filename);
            exit(1);
        }
    };
    let mut tokens: Vec<Token> = vec![];
    let mut row = 1;
    let mut col = 1;
    while !contents.is_empty() {
        if contents.starts_with(|c: char| c.is_whitespace()) {
            let c = chop_character(&mut contents);
            match c {
                '\n' => {
                    row += 1;
                    col = 1;
                }
                _ => {
                    col += 1;
                }
            }
        } else if contents.starts_with(|c: char| c.is_ascii_alphabetic()) {
            let tok = parse_word(
                &mut contents,
                FileLoc {
                    row,
                    col,
                    filename: filename.to_string(),
                },
            );
            col += tok.value.len();
            tokens.push(tok);
        } else if contents.starts_with(|c: char| c == '"') {
            chop_character(&mut contents);
            col += 1;
            let tok = parse_string(
                &mut contents,
                FileLoc {
                    row,
                    col,
                    filename: filename.to_string(),
                },
            );
            col += tok.value.len();
            tokens.push(tok);
        } else if contents.starts_with(|c: char| c.is_ascii_digit() || c == '-') {
            let tok = parse_digit(
                &mut contents,
                FileLoc {
                    row,
                    col,
                    filename: filename.to_string(),
                },
            );
            col += tok.value.len();
            tokens.push(tok);
        } else if contents.starts_with('(') {
            chop_character(&mut contents);
            tokens.push(Token {
                token_type: TokenType::Oparen,
                value: "(".to_string(),
                loc: FileLoc {
                    row,
                    col,
                    filename: filename.to_string(),
                },
            });
            col += 1;
        } else if contents.starts_with(')') {
            chop_character(&mut contents);
            tokens.push(Token {
                token_type: TokenType::Cparen,
                value: ")".to_string(),
                loc: FileLoc {
                    row,
                    col,
                    filename: filename.to_string(),
                },
            });
            col += 1;
        } else if contents.starts_with('&') {
            chop_character(&mut contents);
            tokens.push(Token {
                token_type: TokenType::LineJoin,
                value: "&".to_string(),
                loc: FileLoc {
                    row,
                    col,
                    filename: filename.to_string(),
                },
            });
            col += 1;
        } else if contents.starts_with(',') {
            chop_character(&mut contents);
            tokens.push(Token {
                token_type: TokenType::Comma,
                value: ",".to_string(),
                loc: FileLoc {
                    row,
                    col,
                    filename: filename.to_string(),
                },
            });
            col += 1;
        } else if contents.starts_with('=') {
            chop_character(&mut contents);
            tokens.push(Token {
                token_type: TokenType::Assign,
                value: "=".to_string(),
                loc: FileLoc {
                    row,
                    col,
                    filename: filename.to_string(),
                },
            });
            col += 1;
        } else if contents.starts_with('{') {
            chop_character(&mut contents);
            tokens.push(Token {
                token_type: TokenType::Ocurly,
                value: "{".to_string(),
                loc: FileLoc {
                    row,
                    col,
                    filename: filename.to_string(),
                },
            });
            col += 1;
        } else if contents.starts_with('}') {
            chop_character(&mut contents);
            tokens.push(Token {
                token_type: TokenType::Ccurly,
                value: "}".to_string(),
                loc: FileLoc {
                    row,
                    col,
                    filename: filename.to_string(),
                },
            });
            col += 1;
        } else if contents.starts_with(':') {
            chop_character(&mut contents);
            tokens.push(Token {
                token_type: TokenType::Colon,
                value: ":".to_string(),
                loc: FileLoc {
                    row,
                    col,
                    filename: filename.to_string(),
                },
            });
            col += 1;
        } else if contents.starts_with("%") && col == 1 {
            chop_character(&mut contents);
            col += 1;
            let tok = parse_rpn_expr(
                &mut contents,
                FileLoc {
                    row,
                    col,
                    filename: filename.to_string(),
                },
            );
            col = 1;
            row += 1;
            tokens.push(tok);
        } else if contents.starts_with("!") {
            let mut next = chop_character(&mut contents);
            while next != '\n' {
                next = chop_character(&mut contents);
            }
            row += 1;
            col = 1;
        } else {
            let chr = chop_character(&mut contents);
            eprintln!("{}:{}:{} Unknown character '{}'", filename, row, col, chr,);
        }
    }
    tokens
}

fn parse_tokens(token_list: &[Token]) -> Simulation {
    use TokenType::*;
    let mut calc: RpnCalculator = Default::default();
    let mut acc = Simulation {
        elements: vec![],
        beam: Array2::from(vec![[]]),
    };
    // let mut beam_vec: Vec<[f64; 2]> = vec![];
    // let mut sync_ke: f64;
    // let mut design_ke: f64;
    // let mut design_beta: f64;
    let mut ind: usize = 0;
    while ind < token_list.len() {
        let tok = &token_list[ind];
        if tok.token_type == RpnExpr {
            let (varname, varval) = calc.interpret_string(&tok.value);
            println!("tok = {tok:?}");
            println!("{:?}", calc.mem);
        }
        // println!("tok = {tok:?}");
        ind += 1;
    }
    acc
}

fn token_check(tok: &Token, expected: TokenType) {
    if tok.token_type != expected {
        eprintln!(
            "{}:{}:{} Expected '{}', got '{}'",
            tok.loc.filename, tok.loc.row, tok.loc.col, expected, tok.value,
        );
        exit(1);
    }
}
