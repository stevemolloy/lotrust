use std::fmt;
use std::fs::read_to_string;
use std::process::exit;
use crate::elements::*;
use crate::beam::{Beam, Electron, MASS};

pub struct Simulation {
    pub elements: Vec<Box<dyn Tracking>>,
    pub beam: Beam,
}

impl Simulation {
    pub fn track(&mut self) {
        for element in self.elements.iter() {
            element.track(&mut self.beam);
        }
    }
}

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
    Colon,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Token {
    token_type: TokenType,
    value: String,
    loc: FileLoc,
}

fn parse_word(input: &mut String, loc: FileLoc) -> Token {
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

pub fn tokenize_file_contents(filename: &str) -> Vec<Token> {
    let mut contents = read_to_string(filename).expect("Could not read file.");
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
        } else if contents.starts_with("//") {
            let mut next = chop_character(&mut contents);
            while next != '\n' {
                next = chop_character(&mut contents);
            }
            row += 1;
            col = 1;
        } else {
            let chr = chop_character(&mut contents);
            eprintln!("Unknown character: {}", chr);
        }
    }
    tokens
}

pub fn parse_tokens(token_list: &[Token]) -> Simulation {
    use TokenType::*;
    let mut acc = Simulation {
        elements: vec![],
        beam: vec![],
    };
    let mut ind: usize = 0;
    let mut sync_ke: f64;
    while ind < token_list.len() {
        let tok = &token_list[ind];
        if tok.token_type == Word && tok.value == "beam" {
            ind += 1;
            token_check(&token_list[ind], Ocurly);
            ind += 1;
            token_check(&token_list[ind], Word);
            while token_list[ind].token_type != Ccurly {
                match token_list[ind].value.as_str() {
                    "particles" => {
                        ind += 1;
                        token_check(&token_list[ind], Ocurly);
                        ind += 1;
                        while token_list[ind].token_type != Ccurly {
                            token_check(&token_list[ind], Value);
                            token_check(&token_list[ind + 1], Value);
                            let t = token_list[ind].value.parse::<f64>().expect("uh oh!");
                            ind += 1;
                            let e = token_list[ind].value.parse::<f64>().expect("uh oh!");
                            ind += 1;
                            acc.beam.push(Electron { t, ke: e });
                        }
                    }
                    _ => todo!("Implement more beam definitions"),
                }
                ind += 1;
            }
        }
        if tok.token_type == Word && tok.value == "accelerator" {
            ind += 1;
            token_check(&token_list[ind], Ocurly);
            ind += 1;
            token_check(&token_list[ind], Word);
            if token_list[ind].value != "initial_ke" {
                eprintln!(
                    "{}:{}:{}: The first item in 'accelerator' should be 'initial_ke', not {}'",
                    token_list[ind].loc.filename,
                    token_list[ind].loc.row,
                    token_list[ind].loc.col,
                    token_list[ind].value,
                );
                exit(1);
            }
            ind += 1;
            token_check(&token_list[ind], Colon);
            ind += 1;
            token_check(&token_list[ind], Value);
            sync_ke = token_list[ind].value.parse::<f64>().expect("uh oh!");
            ind += 1;
            while token_list[ind].token_type != Ccurly {
                let ele_type = token_list[ind].value.as_str();
                match ele_type {
                    "drift" => {
                        ind += 1;
                        token_check(&token_list[ind], Colon);
                        ind += 1;
                        token_check(&token_list[ind], Value);
                        let drift_len = token_list[ind].value.parse::<f64>().expect("uh oh!");
                        acc.elements
                            .push(Box::new(Drift::new(drift_len, sync_ke / MASS)));
                    }
                    "corrector" => {
                        ind += 1;
                        token_check(&token_list[ind], Colon);
                        ind += 1;
                        token_check(&token_list[ind], Value);
                        let drift_len = token_list[ind].value.parse::<f64>().expect("uh oh!");
                        acc.elements
                            .push(Box::new(Corr::new(drift_len, sync_ke / MASS)));
                    }
                    "quad" => {
                        ind += 1;
                        token_check(&token_list[ind], Colon);
                        ind += 1;
                        token_check(&token_list[ind], Value);
                        let drift_len = token_list[ind].value.parse::<f64>().expect("uh oh!");
                        acc.elements
                            .push(Box::new(Quad::new(drift_len, sync_ke / MASS)));
                    }
                    "sext" => {
                        ind += 1;
                        token_check(&token_list[ind], Colon);
                        ind += 1;
                        token_check(&token_list[ind], Value);
                        let drift_len = token_list[ind].value.parse::<f64>().expect("uh oh!");
                        acc.elements
                            .push(Box::new(Sext::new(drift_len, sync_ke / MASS)));
                    }
                    "dipole" => {
                        ind += 1;
                        token_check(&token_list[ind], Colon);
                        ind += 1;
                        token_check(&token_list[ind], Value);
                        let b_field = token_list[ind].value.parse::<f64>().expect("uh oh!");
                        ind += 1;
                        token_check(&token_list[ind], Value);
                        let angle = token_list[ind].value.parse::<f64>().expect("uh oh!");
                        acc.elements
                            .push(Box::new(Dipole::new(b_field, angle, sync_ke / MASS)));
                    }
                    "acccav" => {
                        ind += 1;
                        token_check(&token_list[ind], Colon);
                        ind += 1;
                        token_check(&token_list[ind], Value);
                        let length = token_list[ind].value.parse::<f64>().expect("uh oh!");
                        ind += 1;
                        token_check(&token_list[ind], Value);
                        let voltage = token_list[ind].value.parse::<f64>().expect("uh oh!");
                        ind += 1;
                        token_check(&token_list[ind], Value);
                        let freq = token_list[ind].value.parse::<f64>().expect("uh oh!");
                        ind += 1;
                        token_check(&token_list[ind], Value);
                        let phi = token_list[ind].value.parse::<f64>().expect("uh oh!");
                        sync_ke += voltage * length * phi.cos();
                        acc.elements
                            .push(Box::new(AccCav::new(length, voltage, freq, phi)));
                    }
                    _ => todo!("Element '{ele_type}' not defined."),
                }
                ind += 1;
            }
        }
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