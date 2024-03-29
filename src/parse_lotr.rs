use crate::beam::{gamma_2_beta, ke_2_gamma, Beam, C, MASS};
use crate::elements::{make_acccav, make_dipole, make_drift, AccCavDetails, EleType, Element};
use core::f64::consts::PI;
use ndarray::Array2;
use std::fmt;
use std::fs::read_to_string;
use std::process::exit;

pub struct Simulation {
    pub elements: Vec<Element>,
    pub input_beam: Beam,
    pub output_beam: Beam,
    pub input_beam_ke: f64,
    pub breakpoints: Vec<usize>,
    pub breakpoints_passed: Vec<usize>,
    pub current: usize,
}

impl Simulation {
    pub fn step(&mut self) {
        if self.current == self.elements.len() {
            println!("ERROR: Have already tracked to the last element. Consider using `reset`.");
            return;
        }
        if self.current == 0 {
            self.output_beam = self.input_beam.clone();
        }
        println!(
            "Stepping {} particles through a single element...",
            self.input_beam.pos.shape()[0]
        );
        self.output_beam.track(&self.elements[self.current]);
        self.current += 1;
    }

    pub fn track(&mut self) {
        if self.current == self.elements.len() {
            println!("ERROR: Have already tracked to the last element. Consider using `reset`.");
            return;
        }
        if self.current == 0 {
            self.output_beam = self.input_beam.clone();
        }
        let mut eles_to_track = self.elements.len() - self.current;
        for bp in self.breakpoints.iter() {
            if bp > &self.current {
                eles_to_track = bp - self.current;
                break;
            }
        }
        println!(
            "Tracking {} particles through {} accelerator elements...",
            self.input_beam.pos.shape()[0],
            eles_to_track
        );
        for element in self.elements[self.current..].iter() {
            if self.breakpoints.contains(&self.current)
                && !self.breakpoints_passed.contains(&self.current)
            {
                println!(
                    "Stopping at element {} ({}) due to a breakpoint",
                    self.current, element.name
                );
                self.breakpoints_passed.push(self.current);
                break;
            }
            self.current += 1;
            self.output_beam.track(element);
        }
    }

    pub fn find_element_by_name(&self, searchterm: String) -> Option<usize> {
        self.elements.iter().position(|x| x.name == searchterm)
    }

    pub fn rescale_acc_energy(&mut self, mut new_ke: f64) {
        for ele in self.elements.iter_mut() {
            match ele.ele_type {
                EleType::Drift => *ele = make_drift(ele.name.clone(), ele.length, new_ke / MASS),
                EleType::Dipole => {
                    *ele = make_dipole(
                        ele.name.clone(),
                        ele.length,
                        ele.params["angle"],
                        new_ke / MASS,
                    )
                }
                EleType::AccCav(details) => {
                    new_ke += details.voltage * details.phase.cos();
                    *ele = make_acccav(ele.name.clone(), details, ke_2_gamma(new_ke));
                }
            }
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
        input_beam: Beam::new(Array2::from(vec![[0f64, 0f64]])),
        output_beam: Beam::new(Array2::from(vec![[0f64, 0f64]])),
        input_beam_ke: 100e6,
        breakpoints: Vec::new(),
        breakpoints_passed: Vec::new(),
        current: 0,
    };
    let mut beam_vec: Vec<[f64; 2]> = vec![];
    let mut ind: usize = 0;
    let mut sync_ke: f64;
    let mut design_ke: f64;
    let mut design_beta: f64;
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
                    acc.input_beam_ke = design_ke;
                    design_beta = gamma_2_beta(ke_2_gamma(design_ke));
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
                            beam_vec.push([z, (1f64 / design_beta) * (del_e / design_ke)]);
                        }
                    }
                    _ => todo!("Implement more beam definitions"),
                }
                ind += 1;
            }
            acc.input_beam = Beam::new(Array2::from(beam_vec.clone()));
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
                        acc.elements.push(make_drift(
                            "drift_name".to_string(),
                            drift_len,
                            ke_2_gamma(sync_ke),
                        ));
                    }
                    "corrector" => {
                        ind += 1;
                        token_check(&token_list[ind], Colon);
                        ind += 1;
                        token_check(&token_list[ind], Value);
                        let drift_len = token_list[ind].value.parse::<f64>().expect("uh oh!");
                        acc.elements.push(make_drift(
                            "corr_name".to_string(),
                            drift_len,
                            ke_2_gamma(sync_ke),
                        ));
                    }
                    "quad" => {
                        ind += 1;
                        token_check(&token_list[ind], Colon);
                        ind += 1;
                        token_check(&token_list[ind], Value);
                        let drift_len = token_list[ind].value.parse::<f64>().expect("uh oh!");
                        acc.elements.push(make_drift(
                            "quad_name".to_string(),
                            drift_len,
                            ke_2_gamma(sync_ke),
                        ));
                    }
                    "sext" => {
                        ind += 1;
                        token_check(&token_list[ind], Colon);
                        ind += 1;
                        token_check(&token_list[ind], Value);
                        let drift_len = token_list[ind].value.parse::<f64>().expect("uh oh!");
                        acc.elements.push(make_drift(
                            "sext_name".to_string(),
                            drift_len,
                            ke_2_gamma(sync_ke),
                        ));
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
                        acc.elements.push(make_dipole(
                            "dipole_name".to_string(),
                            b_field,
                            angle,
                            ke_2_gamma(sync_ke),
                        ));
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
                        println!("Calling make_acccav in parse_tokens");
                        // let mut params = HashMap::<String, f64>::new();
                        let k = 2f64 * PI * freq / C;
                        let details = AccCavDetails {
                            length,
                            voltage,
                            frequency: freq,
                            phase: phi,
                            wavenumber: k,
                        };
                        // params.insert("l".to_string(), length);
                        // params.insert("v".to_string(), voltage);
                        // params.insert("freq".to_string(), freq);
                        // params.insert("phi".to_string(), phi);
                        acc.elements.push(make_acccav(
                            "acccav_name".to_string(),
                            details,
                            ke_2_gamma(sync_ke),
                        ));
                        sync_ke += voltage * length * phi.cos();
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
