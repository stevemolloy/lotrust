use crate::beam::MASS;
use crate::elegant_rpn::RpnCalculator;
use crate::elements;
use crate::parse_lotr::Simulation;
use ndarray::Array2;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::fmt;
use std::fs::read_to_string;
use std::process::exit;

type Line = Vec<ElegantElement>;

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

#[derive(Debug, Clone)]
struct ElegantElement {
    name: String,
    intermed_type: IntermedType,
    params: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
enum IntermedType {
    Drift,
    AccCav,
    Kick,
    Moni,
    Bend,
    Quad,
    Sext,
}

pub fn load_elegant_file(filename: &str, line_to_expand: &str) -> Simulation {
    let mut calc: RpnCalculator = Default::default();
    let mut line: Line = vec![];
    let tokens = tokenize_file_contents(filename);
    let inter_repr = parse_tokens(&tokens, &mut calc);
    intermed_to_line(&mut line, &inter_repr, line_to_expand);
    line_to_simulation(line)
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
    LineEnd,
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
        if input.starts_with(|c: char| {
            c == '_' || c.is_ascii_alphanumeric() || c == '.' || c == '$' || c == '-'
        }) {
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
    if actually_a_word {
        Token {
            token_type: TokenType::Word,
            value,
            loc,
        }
    } else {
        Token {
            token_type: TokenType::Value,
            value,
            loc,
        }
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
                    if !tokens.is_empty()
                        && (tokens.last().unwrap().token_type != TokenType::LineJoin
                            && tokens.last().unwrap().token_type != TokenType::LineEnd)
                    {
                        tokens.push(Token {
                            token_type: TokenType::LineEnd,
                            value: "LineEnd".to_string(),
                            loc: FileLoc {
                                row,
                                col,
                                filename: filename.to_string(),
                            },
                        });
                    }
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
        } else if contents.starts_with('%') && col == 1 {
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
        } else if contents.starts_with('!') {
            let mut next = chop_character(&mut contents);
            while next != '\n' {
                next = chop_character(&mut contents);
            }
            if !tokens.is_empty()
                && (tokens.last().unwrap().token_type != TokenType::LineJoin
                    && tokens.last().unwrap().token_type != TokenType::LineEnd)
            {
                tokens.push(Token {
                    token_type: TokenType::LineEnd,
                    value: "LineEnd".to_string(),
                    loc: FileLoc {
                        row,
                        col,
                        filename: filename.to_string(),
                    },
                });
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

fn add_ele_to_store(
    token_list: &[Token],
    ind: &mut usize,
    store: &mut Library,
    calc: &mut RpnCalculator,
) {
    use TokenType::*;
    assert!(
        compare_tokentype_at(token_list, *ind, Word)
            || compare_tokentype_at(token_list, *ind, EleStr)
    );
    assert!(compare_tokentype_at(token_list, *ind + 1, Colon));
    assert!(compare_tokentype_at(token_list, *ind + 2, Word));

    let elegant_type = &token_list[*ind + 2];
    let elegant_name = &token_list[*ind].value;
    match elegant_type.value.to_lowercase().as_str() {
        "charge" | "magnify" | "malign" | "watch" | "watchpoint" | "mark" => {
            let str_to_ignore = token_list[*ind].value.to_lowercase().replace('"', "");
            store.ignore(str_to_ignore);
        }
        "line" => {
            assert!(compare_tokentype_at(token_list, *ind + 3, Assign));
            assert!(compare_tokentype_at(token_list, *ind + 4, Oparen));
            let mut offset = 5;
            let mut params: Vec<String> = vec![];
            while token_list[*ind + offset].token_type != Cparen {
                if compare_tokentype_at(token_list, *ind + offset, Word) {
                    params.push(token_list[*ind + offset].value.clone());
                }
                offset += 1;
            }
            store.add_line(token_list[*ind].value.to_lowercase(), params);
        }
        "drift" => {
            assert!(compare_tokentype_at(token_list, *ind + 3, Comma));
            assert!(compare_tokentype_at(token_list, *ind + 4, Word));
            assert!(token_list[*ind + 4].value.to_lowercase() == "l");
            assert!(compare_tokentype_at(token_list, *ind + 5, Assign));
            assert!(compare_tokentype_at(token_list, *ind + 6, Value));
            let ele = ElegantElement {
                name: elegant_name.to_string(),
                intermed_type: IntermedType::Drift,
                params: HashMap::<String, f64>::from([(
                    "l".to_string(),
                    token_list[*ind + 6].value.parse::<f64>().unwrap(),
                )]),
            };
            store.add_element(token_list[*ind].value.to_lowercase(), ele);
        }
        "marker" => {
            let mut params = HashMap::<String, f64>::new();
            let name = token_list[*ind].value.to_lowercase();
            if compare_tokentype_at(token_list, *ind + 3, Comma) {
                params.insert(
                    token_list[*ind + 4].value.clone(),
                    token_list[*ind + 6].value.parse::<f64>().unwrap(),
                );
                *ind += 6;
            } else {
                *ind += 2;
            }
            let ele = ElegantElement {
                name: elegant_name.to_string(),
                intermed_type: IntermedType::Drift,
                params: HashMap::<String, f64>::from([("l".to_string(), 0f64)]),
            };
            store.add_element(name, ele);
        }
        "rfcw" => {
            let mut params = HashMap::<String, f64>::new();
            let mut offset = 3;
            assert!(compare_tokentype_at(token_list, *ind + offset, Comma));
            offset += 1;
            while token_list[*ind + offset].token_type != LineEnd {
                assert!(compare_tokentype_at(token_list, *ind + offset, Word));
                assert!(compare_tokentype_at(token_list, *ind + offset + 1, Assign));
                let key = token_list[*ind + offset].value.clone();
                if key == "zwakefile"
                    || key == "trwakefile"
                    || key == "tColumn"
                    || key == "wzColumn"
                    || key == "wxColumn"
                    || key == "wyColumn"
                {
                    offset += 3;
                    if compare_tokentype_at(token_list, *ind + offset, LineEnd) {
                        break;
                    }
                    offset += 1;
                    if compare_tokentype_at(token_list, *ind + offset, LineJoin) {
                        offset += 1;
                    }
                    continue;
                }
                let val = if compare_tokentype_at(token_list, *ind + offset + 2, Value) {
                    token_list[*ind + offset + 2].value.parse::<f64>().unwrap()
                } else {
                    let store_key = token_list[*ind + offset + 2].value.clone().replace('"', "");
                    calc.interpret_string(&store_key).unwrap()
                };
                params.insert(key, val);
                offset += 3;
                if compare_tokentype_at(token_list, *ind + offset, LineEnd) {
                    break;
                }
                offset += 1;
                if compare_tokentype_at(token_list, *ind + offset, LineJoin) {
                    offset += 1;
                }
            }
            let ele = ElegantElement {
                name: elegant_name.to_string(),
                intermed_type: IntermedType::AccCav,
                params,
            };
            store.add_element(token_list[*ind].value.to_lowercase(), ele);
        }
        "rfdf" => {
            let mut params = HashMap::<String, f64>::new();
            let mut offset = 3;
            assert!(compare_tokentype_at(token_list, *ind + offset, Comma));
            offset += 1;
            while token_list[*ind + offset].token_type != LineEnd {
                assert!(compare_tokentype_at(token_list, *ind + offset, Word));
                assert!(compare_tokentype_at(token_list, *ind + offset + 1, Assign));
                let key = token_list[*ind + offset].value.clone();
                if key == "zwakefile"
                    || key == "trwakefile"
                    || key == "tColumn"
                    || key == "wzColumn"
                    || key == "wxColumn"
                    || key == "wyColumn"
                {
                    offset += 3;
                    if compare_tokentype_at(token_list, *ind + offset, LineEnd) {
                        break;
                    }
                    offset += 1;
                    if compare_tokentype_at(token_list, *ind + offset, LineJoin) {
                        offset += 1;
                    }
                    continue;
                }
                let val = if compare_tokentype_at(token_list, *ind + offset + 2, Value) {
                    token_list[*ind + offset + 2].value.parse::<f64>().unwrap()
                } else {
                    let store_key = token_list[*ind + offset + 2].value.clone().replace('"', "");
                    calc.interpret_string(&store_key).unwrap()
                };
                params.insert(key, val);
                offset += 3;
                if compare_tokentype_at(token_list, *ind + offset, LineEnd) {
                    break;
                }
                offset += 1;
                if compare_tokentype_at(token_list, *ind + offset, LineJoin) {
                    offset += 1;
                }
            }
            let ele = ElegantElement {
                name: elegant_name.to_string(),
                intermed_type: IntermedType::AccCav,
                params,
            };
            store.add_element(token_list[*ind].value.to_lowercase(), ele);
        }
        "kquad" => {
            let mut params = HashMap::<String, f64>::new();
            let mut offset = 3;
            assert!(compare_tokentype_at(token_list, *ind + offset, Comma));
            offset += 1;
            while token_list[*ind + offset].token_type != LineEnd {
                assert!(compare_tokentype_at(token_list, *ind + offset, Word));
                assert!(compare_tokentype_at(token_list, *ind + offset + 1, Assign));
                let key = token_list[*ind + offset].value.clone();
                if key == "SYSTEMATIC_MULTIPOLES" {
                    offset += 3;
                    if compare_tokentype_at(token_list, *ind + offset, LineEnd) {
                        break;
                    }
                    offset += 1;
                    if compare_tokentype_at(token_list, *ind + offset, LineJoin) {
                        offset += 1;
                    }
                    continue;
                }
                let val = if compare_tokentype_at(token_list, *ind + offset + 2, Value) {
                    token_list[*ind + offset + 2].value.parse::<f64>().unwrap()
                } else {
                    let store_key = token_list[*ind + offset + 2].value.clone().replace('"', "");
                    calc.interpret_string(&store_key).unwrap()
                };
                params.insert(key, val);
                offset += 3;
                if compare_tokentype_at(token_list, *ind + offset, LineEnd) {
                    break;
                }
                offset += 1;
                if compare_tokentype_at(token_list, *ind + offset, LineJoin) {
                    offset += 1;
                }
            }
            let ele = ElegantElement {
                name: elegant_name.to_string(),
                intermed_type: IntermedType::Quad,
                params,
            };
            store.add_element(token_list[*ind].value.to_lowercase(), ele);
        }
        "hkick" | "vkick" => {
            let mut params = HashMap::<String, f64>::new();
            let mut offset = 3;
            assert!(compare_tokentype_at(token_list, *ind + offset, Comma));
            offset += 1;
            while token_list[*ind + offset].token_type != LineEnd {
                assert!(compare_tokentype_at(token_list, *ind + offset, Word));
                assert!(compare_tokentype_at(token_list, *ind + offset + 1, Assign));
                let key = token_list[*ind + offset].value.clone();
                let val = if compare_tokentype_at(token_list, *ind + offset + 2, Value) {
                    token_list[*ind + offset + 2].value.parse::<f64>().unwrap()
                } else {
                    let store_key = token_list[*ind + offset + 2].value.clone().replace('"', "");
                    calc.interpret_string(&store_key).unwrap()
                };
                params.insert(key, val);
                offset += 3;
                if compare_tokentype_at(token_list, *ind + offset, LineEnd) {
                    break;
                }
                offset += 1;
                if compare_tokentype_at(token_list, *ind + offset, LineJoin) {
                    offset += 1;
                }
            }
            let ele = ElegantElement {
                name: elegant_name.to_string(),
                intermed_type: IntermedType::Kick,
                params,
            };
            store.add_element(token_list[*ind].value.to_lowercase(), ele);
        }
        "wiggler" | "csrcsbend" | "rben" | "sben" | "sbend" => {
            let mut params = HashMap::<String, f64>::new();
            let mut offset = 3;
            assert!(compare_tokentype_at(token_list, *ind + offset, Comma));
            offset += 1;
            while token_list[*ind + offset].token_type != LineEnd {
                assert!(compare_tokentype_at(token_list, *ind + offset, Word));
                assert!(compare_tokentype_at(token_list, *ind + offset + 1, Assign));
                let key = token_list[*ind + offset].value.clone();
                if key == "output_file" {
                    offset += 3;
                    if compare_tokentype_at(token_list, *ind + offset, LineEnd) {
                        break;
                    }
                    offset += 1;
                    if compare_tokentype_at(token_list, *ind + offset, LineJoin) {
                        offset += 1;
                    }
                    continue;
                }
                let val = if compare_tokentype_at(token_list, *ind + offset + 2, Value) {
                    token_list[*ind + offset + 2].value.parse::<f64>().unwrap()
                } else {
                    let store_key = token_list[*ind + offset + 2].value.clone().replace('"', "");
                    calc.interpret_string(&store_key).unwrap()
                };
                params.insert(key, val);
                offset += 3;
                if compare_tokentype_at(token_list, *ind + offset, LineEnd) {
                    break;
                }
                offset += 1;
                if compare_tokentype_at(token_list, *ind + offset, LineJoin) {
                    offset += 1;
                }
            }
            let ele = ElegantElement {
                name: elegant_name.to_string(),
                intermed_type: IntermedType::Bend,
                params,
            };
            store.add_element(token_list[*ind].value.to_lowercase(), ele);
        }
        "ksext" => {
            let mut params = HashMap::<String, f64>::new();
            let mut offset = 3;
            assert!(compare_tokentype_at(token_list, *ind + offset, Comma));
            offset += 1;
            while token_list[*ind + offset].token_type != LineEnd {
                assert!(compare_tokentype_at(token_list, *ind + offset, Word));
                assert!(compare_tokentype_at(token_list, *ind + offset + 1, Assign));
                let key = token_list[*ind + offset].value.clone();
                if key == "SYSTEMATIC_MULTIPOLES" {
                    offset += 3;
                    if compare_tokentype_at(token_list, *ind + offset, LineEnd) {
                        break;
                    }
                    offset += 1;
                    if compare_tokentype_at(token_list, *ind + offset, LineJoin) {
                        offset += 1;
                    }
                    continue;
                }
                let val = if compare_tokentype_at(token_list, *ind + offset + 2, Value) {
                    token_list[*ind + offset + 2].value.parse::<f64>().unwrap()
                } else {
                    let store_key = token_list[*ind + offset + 2].value.clone().replace('"', "");
                    calc.interpret_string(&store_key).unwrap()
                };
                params.insert(key, val);
                offset += 3;
                if compare_tokentype_at(token_list, *ind + offset, LineEnd) {
                    break;
                }
                offset += 1;
                if compare_tokentype_at(token_list, *ind + offset, LineJoin) {
                    offset += 1;
                }
            }
            let ele = ElegantElement {
                name: elegant_name.to_string(),
                intermed_type: IntermedType::Sext,
                params,
            };
            store.add_element(token_list[*ind].value.to_lowercase(), ele);
        }
        "scraper" | "ecol" => {
            let mut params = HashMap::<String, f64>::new();
            let mut offset = 3;
            assert!(compare_tokentype_at(token_list, *ind + offset, Comma));
            offset += 1;
            while token_list[*ind + offset].token_type != LineEnd {
                assert!(compare_tokentype_at(token_list, *ind + offset, Word));
                assert!(compare_tokentype_at(token_list, *ind + offset + 1, Assign));
                let key = token_list[*ind + offset].value.clone();
                if key == "insert_from" {
                    offset += 3;
                    if compare_tokentype_at(token_list, *ind + offset, LineEnd) {
                        break;
                    }
                    offset += 1;
                    if compare_tokentype_at(token_list, *ind + offset, LineJoin) {
                        offset += 1;
                    }
                    continue;
                }
                let val = if compare_tokentype_at(token_list, *ind + offset + 2, Value) {
                    token_list[*ind + offset + 2].value.parse::<f64>().unwrap()
                } else {
                    let store_key = token_list[*ind + offset + 2].value.clone().replace('"', "");
                    calc.interpret_string(&store_key).unwrap()
                };
                params.insert(key, val);
                offset += 3;
                if compare_tokentype_at(token_list, *ind + offset, LineEnd) {
                    break;
                }
                offset += 1;
                if compare_tokentype_at(token_list, *ind + offset, LineJoin) {
                    offset += 1;
                }
            }
            let ele = ElegantElement {
                name: elegant_name.to_string(),
                intermed_type: IntermedType::Drift,
                params,
            };
            store.add_element(token_list[*ind].value.to_lowercase(), ele);
        }
        "monitor" | "moni" => {
            let mut params = HashMap::<String, f64>::new();
            let mut offset = 3;
            assert!(compare_tokentype_at(token_list, *ind + offset, Comma));
            offset += 1;
            while token_list[*ind + offset].token_type != LineEnd {
                assert!(compare_tokentype_at(token_list, *ind + offset, Word));
                assert!(compare_tokentype_at(token_list, *ind + offset + 1, Assign));
                let key = token_list[*ind + offset].value.clone();
                let val = if compare_tokentype_at(token_list, *ind + offset + 2, Value) {
                    token_list[*ind + offset + 2].value.parse::<f64>().unwrap()
                } else {
                    let store_key = token_list[*ind + offset + 2].value.clone().replace('"', "");
                    calc.interpret_string(&store_key).unwrap()
                };
                params.insert(key, val);
                offset += 3;
                if compare_tokentype_at(token_list, *ind + offset, LineEnd) {
                    break;
                }
                offset += 1;
                if compare_tokentype_at(token_list, *ind + offset, LineJoin) {
                    offset += 1;
                }
            }
            let ele = ElegantElement {
                name: elegant_name.to_string(),
                intermed_type: IntermedType::Moni,
                params,
            };
            store.add_element(token_list[*ind].value.to_lowercase(), ele);
        }
        _ => {
            for ele in &store.elements {
                println!("{ele:?}");
            }
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
    while token_list[*ind].token_type != TokenType::LineEnd {
        *ind += 1;
    }
}

fn parse_tokens(token_list: &[Token], calc: &mut RpnCalculator) -> Library {
    use TokenType::*;
    let mut element_store: Library = Default::default();
    let mut ind: usize = 0;
    while ind < token_list.len() {
        let tok = &token_list[ind];
        if tok.token_type == RpnExpr {
            calc.interpret_string(&tok.value);
        } else if (tok.token_type == Word || tok.token_type == EleStr)
            && compare_tokentype_at(token_list, ind + 1, Colon)
        {
            add_ele_to_store(token_list, &mut ind, &mut element_store, calc);
        } else {
            println!("{tok:?}", tok = token_list[ind - 1]);
            println!("{tok:?}");
            println!("{tok:?}", tok = token_list[ind + 1]);
            eprintln!(
                "{}:{}:{} Cannot handle '{}' with value '{}'",
                tok.loc.filename, tok.loc.row, tok.loc.col, tok.token_type, tok.value
            );
            exit(1);
        }
        ind += 1;
        if ind >= token_list.len() {
            break;
        }
        while compare_tokentype_at(token_list, ind, LineEnd) {
            ind += 1;
        }
    }
    element_store
}

fn intermed_to_line(line: &mut Line, intermed: &Library, line_name: &str) {
    let line_name = &line_name.to_lowercase();
    if let Some(line_defn) = intermed.lines.get(line_name) {
        for subline in line_defn {
            intermed_to_line(line, intermed, subline);
        }
    } else if intermed.ignored.contains(&line_name.to_string()) {
    } else if let Some(ele) = intermed.elements.get(line_name) {
        line.push(ele.clone());
    } else {
        println!("{:#?}", intermed.ignored);
        eprintln!("Trying to expand the line called {line_name} but it cannot be found");
        exit(1);
    }
}

fn line_to_simulation(line: Line) -> Simulation {
    let mut acc = Simulation {
        elements: vec![],
        beam: Array2::from(vec![[0f64, 0f64]]),
    };
    // let mut beam_vec: Vec<[f64; 2]> = vec![];
    // let mut sync_ke: f64;
    let mut design_gamma = 204.80244139169827f64;
    for ele in line {
        match ele.intermed_type {
            IntermedType::Drift
            | IntermedType::Quad
            | IntermedType::Kick
            | IntermedType::Moni
            | IntermedType::Sext => {
                let l = match ele.params.get("l") {
                    Some(l) => *l,
                    None => 0f64,
                };
                acc.elements.push(Box::new(elements::Drift::new(
                    ele.name.to_string(),
                    l,
                    design_gamma,
                )))
            }
            IntermedType::AccCav => {
                let l = match ele.params.get("l") {
                    Some(x) => *x,
                    None => 0f64,
                };
                let volt = match ele.params.get("volt") {
                    Some(x) => *x,
                    None => 0f64,
                };
                let freq = match ele.params.get("freq") {
                    Some(x) => *x,
                    None => 0f64,
                };
                let phase = match ele.params.get("phase") {
                    Some(x) => x.to_radians() - PI / 2f64, // Convert from elegant phase definition
                    None => 0f64,
                };
                design_gamma += (volt * phase.cos()) / MASS;
                acc.elements.push(Box::new(elements::AccCav::new(
                    ele.name.to_string(),
                    l,
                    volt,
                    freq,
                    phase,
                    design_gamma,
                )))
            }
            IntermedType::Bend => {
                let l = match ele.params.get("l") {
                    Some(x) => *x,
                    None => 0f64,
                };
                let angle = match ele.params.get("angle") {
                    Some(x) => *x,
                    None => 0f64,
                };
                acc.elements.push(Box::new(elements::Dipole::new(
                    ele.name.to_string(),
                    l,
                    angle,
                    design_gamma,
                )));
            }
        }
    }
    acc
}

fn compare_tokentype_at(token_list: &[Token], ind: usize, tok_type: TokenType) -> bool {
    token_list[ind].token_type == tok_type
}
