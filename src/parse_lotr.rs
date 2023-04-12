use crate::beam::ke_2_gamma;
use crate::elements::*;
use ndarray::Array2;
use std::fmt;
use std::fs::read_to_string;
use std::process::exit;

pub struct Simulation {
    pub elements: Vec<Box<dyn Tracking>>,
    pub beam: Array2<f64>,
}

impl Simulation {
    pub fn track(&mut self) {
        for element in self.elements.iter() {
            element.track(&mut self.beam);
        }
    }
}

pub fn load_lotr_file(filename: &str) -> Simulation {
    let tokens = tokenize_file_contents(filename);
    parse_tokens(&tokens)
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

struct Token {
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

fn parse_tokens(token_list: &[Token]) -> Simulation {
    use TokenType::*;
    let mut acc = Simulation {
        elements: vec![],
        beam: Array2::from(vec![[]]),
    };
    let mut beam_vec: Vec<[f64; 2]> = vec![];
    let mut ind: usize = 0;
    let mut sync_ke: f64;
    let mut design_ke: f64;
    while ind < token_list.len() {
        let tok = &token_list[ind];
        if tok.token_type == Word && tok.value == "beam" {
            ind += 1;
            token_check(&token_list[ind], Ocurly);
            ind += 1;
            token_check(&token_list[ind], Word);
            match token_list[ind].value.as_str() {
                "design_ke" => {
                    ind += 1;
                    token_check(&token_list[ind], Colon);
                    ind += 1;
                    token_check(&token_list[ind], Value);
                    design_ke = token_list[ind].value.parse::<f64>().expect("uh oh!");
                    ind += 1;
                }
                _ => {
                    eprintln!("Expected 'design_ke', but got {}", token_list[ind].value);
                    exit(1);
                }
            }
            while token_list[ind].token_type != Ccurly {
                match token_list[ind].value.as_str() {
                    "particles" => {
                        ind += 1;
                        token_check(&token_list[ind], Ocurly);
                        ind += 1;
                        while token_list[ind].token_type != Ccurly {
                            token_check(&token_list[ind], Value);
                            token_check(&token_list[ind + 1], Value);
                            let z = token_list[ind].value.parse::<f64>().expect("uh oh!");
                            ind += 1;
                            let del_e = token_list[ind].value.parse::<f64>().expect("uh oh!");
                            ind += 1;
                            beam_vec.push([z, del_e / design_ke]);
                        }
                    }
                    _ => todo!("Implement more beam definitions"),
                }
                ind += 1;
            }
            acc.beam = Array2::from(beam_vec.clone());
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
                            .push(Box::new(Drift::new(drift_len, ke_2_gamma(sync_ke))));
                    }
                    "corrector" => {
                        ind += 1;
                        token_check(&token_list[ind], Colon);
                        ind += 1;
                        token_check(&token_list[ind], Value);
                        let drift_len = token_list[ind].value.parse::<f64>().expect("uh oh!");
                        acc.elements
                            .push(Box::new(Corr::new(drift_len, ke_2_gamma(sync_ke))));
                    }
                    "quad" => {
                        ind += 1;
                        token_check(&token_list[ind], Colon);
                        ind += 1;
                        token_check(&token_list[ind], Value);
                        let drift_len = token_list[ind].value.parse::<f64>().expect("uh oh!");
                        acc.elements
                            .push(Box::new(Quad::new(drift_len, ke_2_gamma(sync_ke))));
                    }
                    "sext" => {
                        ind += 1;
                        token_check(&token_list[ind], Colon);
                        ind += 1;
                        token_check(&token_list[ind], Value);
                        let drift_len = token_list[ind].value.parse::<f64>().expect("uh oh!");
                        acc.elements
                            .push(Box::new(Sext::new(drift_len, ke_2_gamma(sync_ke))));
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
                        acc.elements.push(Box::new(Dipole::new(
                            b_field,
                            angle,
                            ke_2_gamma(sync_ke),
                        )));
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
