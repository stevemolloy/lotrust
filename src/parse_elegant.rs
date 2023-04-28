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
    AccCav,
    Kick,
    // Dipole,
    Quad,
    // Sext,
}

pub fn load_elegant_file(filename: &str) -> Simulation {
    let mut calc: RpnCalculator = Default::default();
    let tokens = tokenize_file_contents(filename);
    parse_tokens(&tokens, &mut calc)
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
    assert_tokentype_at(token_list, *ind + 1, Colon);
    assert_tokentype_at(token_list, *ind + 2, Word);

    let elegant_type = &token_list[*ind + 2];
    match elegant_type.value.as_str() {
        "CHARGE" | "MAGNIFY" | "MALIGN" | "WATCH" => {
            store.ignore(token_list[*ind].value.clone());
        }
        "line" => {
            assert_tokentype_at(token_list, *ind + 3, Assign);
            assert_tokentype_at(token_list, *ind + 4, Oparen);
            let mut offset = 5;
            let mut params: Vec<String> = vec![];
            while token_list[*ind + offset].token_type != Cparen {
                if compare_tokentype_at(token_list, *ind + offset, Word) {
                    params.push(token_list[*ind + offset].value.clone());
                }
                offset += 1;
            }
            store.add_line(token_list[*ind].value.clone(), params);
        }
        "drift" => {
            assert_tokentype_at(token_list, *ind + 3, Comma);
            assert_tokentype_at(token_list, *ind + 4, Word);
            assert!(token_list[*ind + 4].value == "l");
            assert_tokentype_at(token_list, *ind + 5, Assign);
            assert_tokentype_at(token_list, *ind + 6, Value);
            let ele = ElegantElement {
                intermed_type: IntermedType::Drift,
                params: HashMap::<String, f64>::from([(
                    "l".to_string(),
                    token_list[*ind + 6].value.parse::<f64>().unwrap(),
                )]),
            };
            store.add_element(token_list[*ind].value.clone(), ele);
        }
        "marker" => {
            let mut params = HashMap::<String, f64>::new();
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
                intermed_type: IntermedType::Drift,
                params: HashMap::<String, f64>::from([("l".to_string(), 0f64)]),
            };
            store.add_element(token_list[*ind].value.clone(), ele);
        }
        "rfcw" => {
            let mut params = HashMap::<String, f64>::new();
            let mut offset = 3;
            assert_tokentype_at(token_list, *ind + offset, Comma);
            offset += 1;
            while token_list[*ind + offset].token_type != LineEnd {
                assert_tokentype_at(token_list, *ind + offset + 0, Word);
                assert_tokentype_at(token_list, *ind + offset + 1, Assign);
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
                let val: f64;
                if compare_tokentype_at(token_list, *ind + offset + 2, Value) {
                    val = token_list[*ind + offset + 2].value.parse::<f64>().unwrap();
                } else {
                    let store_key = token_list[*ind + offset + 2]
                        .value
                        .clone()
                        .replace("\"", "");
                    val = calc.interpret_string(&store_key).unwrap();
                }
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
                intermed_type: IntermedType::AccCav,
                params,
            };
            store.add_element(token_list[*ind].value.clone(), ele);
        }
        "Kquad" => {
            let mut params = HashMap::<String, f64>::new();
            let mut offset = 3;
            assert_tokentype_at(token_list, *ind + offset, Comma);
            offset += 1;
            while token_list[*ind + offset].token_type != LineEnd {
                assert_tokentype_at(token_list, *ind + offset + 0, Word);
                assert_tokentype_at(token_list, *ind + offset + 1, Assign);
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
                let val: f64;
                if compare_tokentype_at(token_list, *ind + offset + 2, Value) {
                    val = token_list[*ind + offset + 2].value.parse::<f64>().unwrap();
                } else {
                    let store_key = token_list[*ind + offset + 2]
                        .value
                        .clone()
                        .replace("\"", "");
                    val = calc.interpret_string(&store_key).unwrap();
                }
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
                intermed_type: IntermedType::Quad,
                params,
            };
            store.add_element(token_list[*ind].value.clone(), ele);
        }
        "HKICK" | "VKICK" => {
            let mut params = HashMap::<String, f64>::new();
            let mut offset = 3;
            assert_tokentype_at(token_list, *ind + offset, Comma);
            offset += 1;
            while token_list[*ind + offset].token_type != LineEnd {
                assert_tokentype_at(token_list, *ind + offset + 0, Word);
                assert_tokentype_at(token_list, *ind + offset + 1, Assign);
                let key = token_list[*ind + offset].value.clone();
                let val: f64;
                if compare_tokentype_at(token_list, *ind + offset + 2, Value) {
                    val = token_list[*ind + offset + 2].value.parse::<f64>().unwrap();
                } else {
                    let store_key = token_list[*ind + offset + 2]
                        .value
                        .clone()
                        .replace("\"", "");
                    val = calc.interpret_string(&store_key).unwrap();
                }
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
                intermed_type: IntermedType::Kick,
                params,
            };
            store.add_element(token_list[*ind].value.clone(), ele);
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

fn parse_tokens(token_list: &[Token], calc: &mut RpnCalculator) -> Simulation {
    use TokenType::*;
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
        } else if (tok.token_type == Word || tok.token_type == EleStr)
            && compare_tokentype_at(token_list, ind + 1, Colon)
        {
            add_ele_to_store(&token_list, &mut ind, &mut element_store, calc);
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
        while compare_tokentype_at(token_list, ind, LineEnd) {
            ind += 1;
        }
    }
    acc
}

fn compare_tokentype_at(token_list: &[Token], ind: usize, tok_type: TokenType) -> bool {
    token_list[ind].token_type == tok_type
}

fn assert_tokentype_at(token_list: &[Token], ind: usize, tok_type: TokenType) {
    assert!(compare_tokentype_at(token_list, ind, tok_type))
}
