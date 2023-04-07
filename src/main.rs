use std::fs::read_to_string;
// use std::f64::consts::PI;
const MASS: f64 = 510998.9499961642f64;
const C: f64 = 299792458f64;

// TODO: Electrons may be better described as a simple array. Look at ndarray.
#[derive(Copy, Clone)]
pub struct Electron {
    t: f64,
    ke: f64,
}

type Beam = Vec<Electron>;

pub trait Tracking {
    fn track(&self, beam: Beam) -> Beam;
}

struct Drift {
    length: f64,
    gamma0: f64,
}

impl Drift {
    fn new(l: f64, g: f64) -> Drift {
        Drift {
            length: l,
            gamma0: g,
        }
    }
}

impl Tracking for Drift {
    fn track(&self, beam: Beam) -> Beam {
        let mut output_beam: Beam = vec![];
        for electron in beam {
            let t = electron.t;
            let l = self.length;

            let g0 = self.gamma0;
            let g = electron.ke / MASS;

            let beta = (1.0 - (1.0 / g.powi(2))).sqrt();
            let beta0 = (1.0 - (1.0 / g0.powi(2))).sqrt();

            let new_t = t + (l / C) * (1.0 / beta - 1.0 / beta0);

            output_beam.push(Electron {
                t: new_t,
                ke: electron.ke,
            });
        }
        output_beam
    }
}

struct Dipole {
    b_field: f64,
    theta: f64,
    gamma0: f64,
}

impl Dipole {
    fn new(b: f64, angle: f64, g: f64) -> Dipole {
        Dipole {
            b_field: b,
            theta: angle,
            gamma0: g,
        }
    }
}

impl Tracking for Dipole {
    fn track(&self, beam: Beam) -> Beam {
        let mut output_beam: Beam = vec![];
        for electron in beam {
            let g0 = self.gamma0;
            let g = electron.ke / MASS;

            let pc0 = (g0.powi(2) - 1.0).sqrt() * MASS;
            let pc = (g.powi(2) - 1.0).sqrt() * MASS;

            let rho0 = pc0 / (C * self.b_field);
            let rho = pc / (C * self.b_field);

            let l0 = rho0 * self.theta;
            let l = rho * self.theta;

            let delta_l = l - l0;
            let v = C * (1.0 - (1.0 / g.powi(2))).sqrt();

            let new_t = electron.t + delta_l / v;

            output_beam.push(Electron {
                t: new_t,
                ke: electron.ke,
            });
        }
        output_beam
    }
}

// struct AccCav {
//     length: f64,
//     voltage: f64,
//     freq: f64,
//     phi: f64,
// }
//
// impl AccCav {
//     fn new(l: f64, v: f64, freq: f64, phi: f64) -> AccCav {
//         AccCav {
//             length: l,
//             voltage: v,
//             freq: freq,
//             phi: phi,
//         }
//     }
// }

// impl Tracking for AccCav {
//     fn track(&self, beam: Beam) -> Beam {
//         let mut output_beam: Beam = vec![];
//         let egain = self.length * self.voltage;
//         for electron in beam {
//             let phase = self.phi + 2.0 * PI * (electron.t * self.freq);
//             output_beam.push(Electron {
//                 t: electron.t,
//                 ke: electron.ke + egain * phase.cos(),
//             });
//         }
//         output_beam
//     }
// }

struct Accelerator {
    pub elements: Vec<Box<dyn Tracking>>,
}

impl Accelerator {
    fn track(&self, beam: Beam) -> Beam {
        let mut output_beam = beam;
        for element in self.elements.iter() {
            output_beam = element.track(output_beam.clone());
        }
        output_beam
    }
}

#[derive(Debug, PartialEq)]
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
    let mut value: String = chop_character(input).to_string();
    while !input.is_empty() {
        if input.starts_with(|c: char| c.is_ascii_digit()) {
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
        value: value,
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
        } else if contents.starts_with(|c: char| c.is_ascii_digit() || c=='-') {
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

fn parse_tokens(token_list: &[Token]) -> Accelerator {
    use TokenType::*;
    let mut acc = Accelerator { elements: vec![] };
    let mut ind: usize = 0;
    let mut starting_ke = 100e6;
    while ind < token_list.len() {
        let tok = &token_list[ind];
        if tok.token_type == Word && tok.value == "beam" {
            ind += 1;
            assert!(token_list[ind].token_type == Ocurly);
            ind += 1;
            assert!(token_list[ind].token_type == Word);
            while token_list[ind].token_type != Ccurly {
                match token_list[ind].value.as_str() {
                    "energy" => {
                        ind += 1;
                        assert!(token_list[ind].token_type == Colon);
                        ind += 1;
                        assert!(token_list[ind].token_type == Value);
                        starting_ke = token_list[ind].value.parse::<f64>().expect("uh oh!");
                    }
                    _ => todo!("Implement more beam definitions"),
                }
                ind += 1;
            }
        }
        if tok.token_type == Word && tok.value == "accelerator" {
            ind += 1;
            assert!(token_list[ind].token_type == Ocurly);
            ind += 1;
            assert!(token_list[ind].token_type == Word);
            while token_list[ind].token_type != Ccurly {
                let ele_type = token_list[ind].value.as_str();
                match ele_type {
                    "drift" => {
                        ind += 1;
                        assert!(token_list[ind].token_type == Colon);
                        ind += 1;
                        assert!(token_list[ind].token_type == Value);
                        let drift_len = token_list[ind].value.parse::<f64>().expect("uh oh!");
                        acc.elements
                            .push(Box::new(Drift::new(drift_len, starting_ke / MASS)));
                    }
                    "dipole" => {
                        ind += 1;
                        assert!(token_list[ind].token_type == Colon);
                        ind += 1;
                        assert!(token_list[ind].token_type == Value);
                        let b_field = token_list[ind].value.parse::<f64>().expect("uh oh!");
                        ind += 1;
                        assert!(token_list[ind].token_type == Value);
                        let angle = token_list[ind].value.parse::<f64>().expect("uh oh!");
                        acc.elements.push(Box::new(Dipole::new(
                            b_field,
                            angle,
                            starting_ke / MASS,
                        )));
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

fn main() {
    let filename = "acc_defn.lotr";
    let mut contents = read_to_string(filename).expect("Could not read file.");
    let tokens = tokenize_file_contents(&mut contents);
    let accelerator: Accelerator = parse_tokens(&tokens);
    let design_ke = 20e6;
    let beam = vec![
        Electron {
            t: -10e-12,
            ke: 0.99 * design_ke,
        },
        Electron {
            t: -10e-12,
            ke: design_ke,
        },
        Electron {
            t: -10e-12,
            ke: 1.01 * design_ke,
        },
        Electron {
            t: 0.0,
            ke: design_ke,
        },
        Electron {
            t: 0.0,
            ke: 0.99 * design_ke,
        },
        Electron {
            t: 0.0,
            ke: 1.01 * design_ke,
        },
        Electron {
            t: 10e-12,
            ke: 0.99 * design_ke,
        },
        Electron {
            t: 10e-12,
            ke: design_ke,
        },
        Electron {
            t: 10e-12,
            ke: 1.01 * design_ke,
        },
    ];

    println!("---   INPUT  ---");
    for electron in &beam {
        println!(
            "{:0.6} ps :: {:0.3} MeV",
            electron.t * 1e12,
            electron.ke * 1e-6
        );
    }
    println!("--- TRACKING ---");
    let out_beam = accelerator.track(beam);
    println!("---  OUTPUT  ---");

    for electron in out_beam {
        println!(
            "{:0.6} ps :: {:0.3} MeV",
            electron.t * 1e12,
            electron.ke * 1e-6
        );
    }
}
