// use crate::beam::gamma_2_beta;
use crate::elegant_rpn::RpnCalculator;
// use crate::elements::*;
use crate::parse_lotr::Simulation;
use ndarray::Array2;
use std::collections::HashMap;
use std::fmt;
use std::fs::read_to_string;
use std::process::exit;

#[derive(Debug, Default)]
struct Library {
    elements: HashMap<String, ElegantElement>,
    ignored: Vec<String>,
    lines: HashMap<String, Vec<String>>,
}

impl Library {
    fn add_element(&mut self, key: String, value: ElegantElement) {
        self.elements.insert(key, value);
    }

    fn ignore(&mut self, name: String) {
        self.ignored.push(name);
    }

    fn add_line(&mut self, name: String, elements: Vec<String>) {
        self.lines.insert(name, elements);
    }
}

#[derive(Debug)]
struct ElegantElement {
    intermed_type: IntermedType,
    params: HashMap<String, f64>,
}

#[derive(Debug)]
enum IntermedType {
    Drift,
    // AccCav,
    // Dipole,
    // Quad,
    // Sext,
}

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
        name.push(chop_character(input));
        if input.starts_with(|c: char| c == '"') {
            break;
        }
    }
    name.push(chop_character(input));
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
    let mut actually_a_word: bool = false;
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
            if already_exp && !actually_a_word {
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
        } else if input.starts_with(|c: char| c.is_ascii_alphabetic()) {
            actually_a_word = true;
            value.push(chop_character(input));
        } else {
            break;
        }
    }
    let tok: Token;
    if actually_a_word {
        tok = Token {
            token_type: TokenType::Word,
            value,
            loc,
        }
    } else {
        tok = Token {
            token_type: TokenType::Value,
            value,
            loc,
        }
    }
    tok
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
            // chop_character(&mut contents);
            // col += 1;
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
            //tokens.push(Token {
            //    token_type: TokenType::LineJoin,
            //    value: "&".to_string(),
            //    loc: FileLoc {
            //        row,
            //        col,
            //        filename: filename.to_string(),
            //    },
            //});
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

fn add_ele_to_store(token_list: &[Token], ind: &mut usize, store: &mut Library) {
    use TokenType::*;
    assert!(token_list[*ind + 0].token_type == Word || token_list[*ind + 0].token_type == EleStr);
    assert!(token_list[*ind + 1].token_type == Colon);
    assert!(token_list[*ind + 2].token_type == Word);

    let elegant_type = &token_list[*ind + 2];
    match elegant_type.value.as_str() {
        "CHARGE" => {
            store.ignore(token_list[*ind].value.clone());
            *ind += 6;
        }
        "MALIGN" => {
            store.ignore(token_list[*ind].value.clone());
            *ind += 2;
        }
        "MAGNIFY" => {
            store.ignore(token_list[*ind].value.clone());
            *ind += 2;
        }
        "WATCH" => {
            store.ignore(token_list[*ind].value.clone());
            *ind += 6;
        }
        "line" => {
            assert!(token_list[*ind + 3].token_type == Assign);
            assert!(token_list[*ind + 4].token_type == Oparen);
            let mut offset = 5;
            let mut params: Vec<String> = vec![];
            while token_list[*ind + offset].token_type != Cparen {
                if token_list[*ind + offset].token_type == Word {
                    params.push(token_list[*ind + offset].value.clone());
                }
                offset += 1;
            }
            store.add_line(token_list[*ind].value.clone(), params);
            *ind += offset;
        }
        "drift" => {
            assert!(token_list[*ind + 3].token_type == Comma);
            assert!(token_list[*ind + 4].token_type == Word);
            assert!(token_list[*ind + 4].value == "l");
            assert!(token_list[*ind + 5].token_type == Assign);
            assert!(token_list[*ind + 6].token_type == Value);
            let ele = ElegantElement {
                intermed_type: IntermedType::Drift,
                params: HashMap::<String, f64>::from([(
                    "l".to_string(),
                    token_list[*ind + 6].value.parse::<f64>().unwrap(),
                )]),
            };
            store.add_element(token_list[*ind].value.clone(), ele);
            *ind += 6;
        }
        "marker" => {
            let mut params = HashMap::<String, f64>::new();
            if token_list[*ind + 3].token_type == Comma {
                params.insert(
                    token_list[*ind + 4].value.clone(),
                    token_list[*ind + 6].value.parse::<f64>().unwrap(),
                );
                *ind += 6;
            } else {
                *ind += 2;
            }
            let ele = ElegantElement {
                intermed_type: IntermedType::Drift,
                params: HashMap::<String, f64>::from([("l".to_string(), 0f64)]),
            };
            store.add_element(token_list[*ind].value.clone(), ele);
            // println!("{:?}", store);
        }
        "rfcw" => {
            let mut params = HashMap::<String, f64>::new();
            let mut offset = 3;
            assert_eq!(token_list[*ind + offset].token_type, Comma);
            offset += 1;
            loop {
                let isparam = token_list[*ind + offset].token_type == Word
                    && token_list[*ind + offset + 1].token_type == Assign;
                if !isparam {
                    break;
                }
                if token_list[*ind + offset].token_type = Value
            }
        }
        _ => {
            eprintln!(
                "{}:{}:{} Unrecognised elegant type: '{}'",
                elegant_type.loc.filename,
                elegant_type.loc.row,
                elegant_type.loc.col,
                elegant_type.value
            );
            exit(1);
        }
    }
}

fn parse_tokens(token_list: &[Token]) -> Simulation {
    use TokenType::*;
    let mut calc: RpnCalculator = Default::default();
    let mut element_store: Library = Default::default();
    let acc = Simulation {
        elements: vec![],
        beam: Array2::from(vec![[]]),
    };
    // let mut beam_vec: Vec<[f64; 2]> = vec![];
    // let mut sync_ke: f64;
    // let design_gamma = 182f64;
    // let design_beta = gamma_2_beta(design_gamma);
    let mut ind: usize = 0;
    while ind < token_list.len() {
        let tok = &token_list[ind];
        if tok.token_type == RpnExpr {
            calc.interpret_string(&tok.value);
        } else if tok.token_type == Word && token_list[ind + 1].token_type == Colon {
            add_ele_to_store(&token_list, &mut ind, &mut element_store);
        } else if tok.token_type == EleStr && token_list[ind + 1].token_type == Colon {
            add_ele_to_store(&token_list, &mut ind, &mut element_store);
        } else {
            eprintln!(
                "{}:{}:{} Cannot handle '{}' with value '{}'",
                tok.loc.filename, tok.loc.row, tok.loc.col, tok.token_type, tok.value
            );
            exit(1);
        }
        ind += 1;
    }
    acc
}
