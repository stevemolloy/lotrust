use std::fs::read_to_string;
use std::f64::consts::PI;
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

// struct Dipole {
//     b_field: f64,
//     theta: f64,
//     gamma0: f64,
// }
// 
// impl Dipole {
//     fn new(b: f64, angle: f64, g: f64) -> Dipole {
//         Dipole {
//             b_field: b,
//             theta: angle,
//             gamma0: g,
//         }
//     }
// }
// 
// impl Tracking for Dipole {
//     fn track(&self, beam: Beam) -> Beam {
//         let mut output_beam: Beam = vec![];
//         for electron in beam {
//             let g0 = self.gamma0;
//             let g = electron.ke / MASS;
// 
//             let pc0 = (g0.powi(2) - 1.0).sqrt() * MASS;
//             let pc = (g.powi(2) - 1.0).sqrt() * MASS;
// 
//             let rho0 = pc0 / (C * self.b_field);
//             let rho = pc / (C * self.b_field);
// 
//             let l0 = rho0 * self.theta;
//             let l = rho * self.theta;
// 
//             let delta_l = l - l0;
//             let v = C * (1.0 - (1.0 / g.powi(2))).sqrt();
// 
//             let new_t = electron.t + delta_l / v;
// 
//             output_beam.push(Electron {
//                 t: new_t,
//                 ke: electron.ke,
//             });
//         }
//         output_beam
//     }
// }
// 
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
