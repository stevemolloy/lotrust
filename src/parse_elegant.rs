use crate::beam::{Beam, MASS};
use crate::elegant_rpn::RpnCalculator;
use crate::elements::{make_acccav, make_dipole, make_drift, make_quad};
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
    Line(Vec<String>),
    Ignore,
}

#[derive(Debug, Clone)]
struct FileLoc {
    filename: String,
    row: usize,
    col: usize,
}

#[derive(Debug, PartialEq, Clone)]
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
    Eof,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
struct Token {
    token_type: TokenType,
    value: String,
    loc: FileLoc,
}

pub fn load_elegant_file(filename: &str, line_to_expand: &str) -> Simulation {
    let line_to_expand = line_to_expand.to_lowercase();
    let mut calc: RpnCalculator = Default::default();
    let mut line: Line = vec![];
    let tokens = tokenize_file_contents(filename);
    let inter_repr = parse_tokens(&tokens, &mut calc);
    intermed_to_line(&mut line, &inter_repr, &line_to_expand);
    line_to_simulation(line)
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
        Ok(contents) => contents.to_lowercase(),
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
        let location = FileLoc {
            row,
            col,
            filename: filename.to_string(),
        };
        if contents.starts_with(|c: char| c.is_whitespace()) {
            let c = chop_character(&mut contents);
            match c {
                '\n' => {
                    if !tokens.is_empty()
                        && (tokens.last().unwrap().token_type != TokenType::LineEnd)
                    {
                        tokens.push(Token {
                            token_type: TokenType::LineEnd,
                            value: "LineEnd".to_string(),
                            loc: location,
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
            let tok = parse_word(&mut contents, location);
            col += tok.value.len();
            tokens.push(tok);
        } else if contents.starts_with(|c: char| c == '"') {
            let tok = parse_string(&mut contents, location);
            col += tok.value.len();
            tokens.push(tok);
        } else if contents.starts_with(|c: char| c.is_ascii_digit() || c == '-') {
            let tok = parse_digit(&mut contents, location);
            col += tok.value.len();
            tokens.push(tok);
        } else if contents.starts_with('(') {
            chop_character(&mut contents);
            tokens.push(Token {
                token_type: TokenType::Oparen,
                value: "(".to_string(),
                loc: location,
            });
            col += 1;
        } else if contents.starts_with(')') {
            chop_character(&mut contents);
            tokens.push(Token {
                token_type: TokenType::Cparen,
                value: ")".to_string(),
                loc: location,
            });
            col += 1;
        } else if contents.starts_with('&') {
            chop_character(&mut contents);
            tokens.push(Token {
                token_type: TokenType::LineJoin,
                value: "&".to_string(),
                loc: location,
            });
            col += 1;
        } else if contents.starts_with(',') {
            chop_character(&mut contents);
            tokens.push(Token {
                token_type: TokenType::Comma,
                value: ",".to_string(),
                loc: location,
            });
            col += 1;
        } else if contents.starts_with('=') {
            chop_character(&mut contents);
            tokens.push(Token {
                token_type: TokenType::Assign,
                value: "=".to_string(),
                loc: location,
            });
            col += 1;
        } else if contents.starts_with('{') {
            chop_character(&mut contents);
            tokens.push(Token {
                token_type: TokenType::Ocurly,
                value: "{".to_string(),
                loc: location,
            });
            col += 1;
        } else if contents.starts_with('}') {
            chop_character(&mut contents);
            tokens.push(Token {
                token_type: TokenType::Ccurly,
                value: "}".to_string(),
                loc: location,
            });
            col += 1;
        } else if contents.starts_with(':') {
            chop_character(&mut contents);
            tokens.push(Token {
                token_type: TokenType::Colon,
                value: ":".to_string(),
                loc: location,
            });
            col += 1;
        } else if contents.starts_with('%') && col == 1 {
            chop_character(&mut contents);
            let tok = parse_rpn_expr(&mut contents, location);
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
                    loc: location,
                });
            }
            row += 1;
            col = 1;
        } else {
            let chr = chop_character(&mut contents);
            eprintln!("{}:{}:{} Unknown character '{}'", filename, row, col, chr,);
        }
    }

    tokens.push(Token {
        token_type: TokenType::Eof,
        value: "EOF".to_string(),
        loc: FileLoc {
            row,
            col,
            filename: filename.to_string(),
        },
    });

    let mut ind = 0;
    let mut cleaned_tokens: Vec<Token> = vec![];

    while ind < tokens.len() {
        let this_tok = &tokens[ind];
        ind += 1;
        if (this_tok.token_type == TokenType::LineJoin
            && tokens[ind].token_type == TokenType::LineEnd)
            || (this_tok.token_type == TokenType::LineEnd
                && tokens[ind - 2].token_type == TokenType::LineJoin)
        {
            continue;
        }

        cleaned_tokens.push(this_tok.clone());
    }

    cleaned_tokens
}

fn get_tokens_for_next_ele(token_list: &[Token], ind: &mut usize) -> Vec<Token> {
    let mut return_token_list: Vec<Token> = vec![];

    while token_list[*ind].token_type != TokenType::LineEnd {
        return_token_list.push(token_list[*ind].clone());
        *ind += 1;
    }
    return_token_list.push(token_list[*ind].clone());

    return_token_list
}

fn get_param_list(token_list: &[Token], calc: &mut RpnCalculator) -> HashMap<String, f64> {
    let params_to_ignore = vec![
        "zwake",
        "trwake",
        "zwakefile",
        "tcolumn",
        "wzcolumn",
        "trwakefile",
        "wxcolumn",
        "wycolumn",
        "systematic_multipoles",
        "insert_from",
        "output_file",
    ];
    let mut ind = 4;
    let mut params = HashMap::<String, f64>::new();
    if token_list.len() > 4 {
        while token_list[ind].token_type != TokenType::LineEnd {
            if token_list[ind].token_type == TokenType::Comma {
                ind += 1;
                continue;
            }
            let param = token_list[ind].clone();
            if !params_to_ignore.contains(&param.value.as_str()) {
                let name_of_param = param.value;
                let value = token_list[ind + 2].clone();
                assert!(param.token_type == TokenType::Word);
                assert!(token_list[ind + 1].token_type == TokenType::Assign);
                assert!(
                    value.token_type == TokenType::Value || value.token_type == TokenType::EleStr
                );

                if value.token_type == TokenType::Value {
                    params.insert(name_of_param, value.value.parse().unwrap());
                } else {
                    let store_key = value.value.clone().replace('"', "");
                    let val = calc.interpret_string(&store_key).unwrap();
                    params.insert(name_of_param, val);
                }
            }
            ind += 3;
        }
    }
    if !params.contains_key("l") {
        params.insert("l".to_string(), 0f64);
    }
    params
}

fn get_next_ele_from_tokens(token_list: &[Token], calc: &mut RpnCalculator) -> ElegantElement {
    assert!(
        token_list[0].token_type == TokenType::Word
            || token_list[0].token_type == TokenType::EleStr
    );
    assert!(token_list[1].token_type == TokenType::Colon);
    assert!(token_list[2].token_type == TokenType::Word);

    let ele_name = token_list[0].value.replace('"', "");

    match token_list[2].value.as_str() {
        "charge" | "magnify" | "malign" | "watch" | "watchpoint" | "mark" => ElegantElement {
            name: ele_name,
            intermed_type: IntermedType::Ignore,
            params: HashMap::<String, f64>::new(),
        },
        "drift" | "marker" | "scraper" | "ecol" | "wiggler" => ElegantElement {
            name: ele_name,
            intermed_type: IntermedType::Drift,
            params: get_param_list(token_list, calc),
        },
        "rfcw" | "rfdf" => ElegantElement {
            name: ele_name,
            intermed_type: IntermedType::AccCav,
            params: get_param_list(token_list, calc),
        },
        "kquad" => ElegantElement {
            name: ele_name,
            intermed_type: IntermedType::Quad,
            params: get_param_list(token_list, calc),
        },
        "hkick" | "vkick" => ElegantElement {
            name: ele_name,
            intermed_type: IntermedType::Kick,
            params: get_param_list(token_list, calc),
        },
        "monitor" | "moni" => ElegantElement {
            name: ele_name,
            intermed_type: IntermedType::Moni,
            params: get_param_list(token_list, calc),
        },
        "csrcsbend" | "rben" | "sben" | "sbend" => {
            println!(
                "param_list for bend '{}': {:#?}",
                ele_name,
                get_param_list(token_list, calc)
            );
            ElegantElement {
                name: ele_name,
                intermed_type: IntermedType::Bend,
                params: get_param_list(token_list, calc),
            }
        }
        "ksext" => ElegantElement {
            name: ele_name,
            intermed_type: IntermedType::Sext,
            params: get_param_list(token_list, calc),
        },
        "line" => {
            assert!(token_list[4].token_type == TokenType::Oparen);
            let mut ind = 5;
            let mut contained: Vec<String> = vec![];
            while token_list[ind].token_type != TokenType::Cparen {
                if token_list[ind].token_type == TokenType::Comma {
                    ind += 1;
                    continue;
                }
                let subline_name = token_list[ind].clone();
                assert!(
                    subline_name.token_type == TokenType::Word
                        || subline_name.token_type == TokenType::EleStr,
                    "Expected a 'Word', but got {:#?}",
                    subline_name
                );
                if subline_name.token_type == TokenType::EleStr {
                    contained.push(subline_name.value.replace('"', ""));
                }
                contained.push(subline_name.value);
                ind += 1;
            }
            ind += 1;
            assert!(token_list[ind].token_type == TokenType::LineEnd);
            ElegantElement {
                name: ele_name,
                intermed_type: IntermedType::Line(contained),
                params: HashMap::<String, f64>::new(),
            }
        }
        _ => {
            eprintln!("Tokens to get ele from: {:#?}", token_list);
            eprintln!("Exiting as unable to interpret the above.");
            exit(1);
        }
    }
}

fn add_ele_to_store(
    token_list: &[Token],
    ind: &mut usize,
    store: &mut Library,
    calc: &mut RpnCalculator,
) {
    use IntermedType::*;
    let toks = get_tokens_for_next_ele(token_list, ind);
    let new_ele = get_next_ele_from_tokens(&toks, calc);
    match new_ele.intermed_type {
        Ignore => store.ignore(new_ele.name),
        Drift | AccCav | Quad | Kick | Moni | Bend | Sext => {
            store.add_element(new_ele.name.clone(), new_ele)
        }
        Line(contents) => store.add_line(new_ele.name, contents),
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
        } else if tok.token_type == Eof {
            break;
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
    let line_name = &line_name.replace('"', "");
    if let Some(line_defn) = intermed.lines.get(line_name) {
        for subline in line_defn {
            intermed_to_line(line, intermed, subline);
        }
    } else if intermed.ignored.contains(&line_name.to_string()) {
    } else if let Some(ele) = intermed.elements.get(line_name) {
        line.push(ele.clone());
    } else {
        println!("{:#?}", intermed.ignored);
        println!("{:#?}", intermed.lines.keys());
        println!("{:#?}", intermed.ignored.contains(&line_name.to_string()));
        eprintln!(
            "Trying to expand the line called '{:#?}' but it cannot be found",
            line_name
        );
    }
}

fn line_to_simulation(line: Line) -> Simulation {
    let input_beam = Beam::new(Array2::from(vec![[0f64, 0f64]]));
    let output_beam = Beam::new(Array2::from(vec![[0f64, 0f64]]));
    let mut acc = Simulation {
        elements: vec![],
        input_beam,
        output_beam,
        input_beam_ke: 100e6,
        breakpoints: Vec::new(),
        breakpoints_passed: Vec::new(),
        current: 0,
    };
    let mut design_gamma = acc.input_beam_ke / MASS;
    for ele in line {
        match ele.intermed_type {
            IntermedType::Drift | IntermedType::Kick | IntermedType::Moni | IntermedType::Sext => {
                let l = match ele.params.get("l") {
                    Some(l) => *l,
                    None => 0f64,
                };
                acc.elements
                    .push(make_drift(ele.name.to_string(), l, design_gamma))
            }
            IntermedType::Quad => {
                let l = match ele.params.get("l") {
                    Some(l) => *l,
                    None => 0f64,
                };
                acc.elements
                    .push(make_quad(ele.name.to_string(), l, design_gamma))
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
                    Some(x) => x.to_radians() - PI / 2f64, // Convert from elegant phase defn
                    None => 0f64,
                };
                design_gamma += (volt * phase.cos()) / MASS;
                acc.elements.push(make_acccav(
                    ele.name.to_string(),
                    l,
                    volt,
                    freq,
                    phase,
                    design_gamma,
                ))
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
                acc.elements
                    .push(make_dipole(ele.name.to_string(), l, angle, design_gamma));
            }
            IntermedType::Line(_) => {
                todo!()
            }
            IntermedType::Ignore => {
                todo!()
            }
        }
    }
    acc
}

fn compare_tokentype_at(token_list: &[Token], ind: usize, tok_type: TokenType) -> bool {
    token_list[ind].token_type == tok_type
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;

    use crate::{
        beam::print_beam,
        parse_elegant::load_elegant_file,
        parse_lotr::{load_lotr_file, Simulation},
    };

    pub fn diff_files(f1: &mut File, f2: &mut File) -> bool {
        let buff1: &mut [u8] = &mut [0; 1024];
        let buff2: &mut [u8] = &mut [0; 1024];

        loop {
            match f1.read(buff1) {
                Err(_) => return false,
                Ok(f1_read_len) => match f2.read(buff2) {
                    Err(_) => return false,
                    Ok(f2_read_len) => {
                        if f1_read_len != f2_read_len {
                            return false;
                        }
                        if f1_read_len == 0 {
                            return true;
                        }
                        if buff1[0..f1_read_len] != buff2[0..f2_read_len] {
                            return false;
                        }
                    }
                },
            }
        }
    }

    const SPF_TESTFILE: &str = "tests/elegant_example.lte";
    const ELEGANT_TESTFILE: &str = "tests/test_lines.lte";
    const BEAM_TESTFILE: &str = "tests/test_beam.lotr";

    const DRIFT_BEAM_TRUE: &str = "tests/drift_output_true.beam";
    const DRIFT_BEAM_TEST: &str = "tests/drift_output_test.beam";

    const MARKER_BEAM_TRUE: &str = "tests/marker_output_true.beam";
    const MARKER_BEAM_TEST: &str = "tests/marker_output_test.beam";

    const SBEND_BEAM_TRUE: &str = "tests/sbend_output_true.beam";
    const SBEND_BEAM_TEST: &str = "tests/sbend_output_test.beam";

    const HKICK_BEAM_TRUE: &str = "tests/hkick_output_true.beam";
    const HKICK_BEAM_TEST: &str = "tests/hkick_output_test.beam";

    const VKICK_BEAM_TRUE: &str = "tests/vkick_output_true.beam";
    const VKICK_BEAM_TEST: &str = "tests/vkick_output_test.beam";

    const KQUAD_BEAM_TRUE: &str = "tests/kquad_output_true.beam";
    const KQUAD_BEAM_TEST: &str = "tests/kquad_output_test.beam";

    const RFCW_BEAM_TRUE: &str = "tests/rfcw_output_true.beam";
    const RFCW_BEAM_TEST: &str = "tests/rfcw_output_test.beam";

    const RFDF_BEAM_TRUE: &str = "tests/rfdf_output_true.beam";
    const RFDF_BEAM_TEST: &str = "tests/rfdf_output_test.beam";

    const WIGGLER_BEAM_TRUE: &str = "tests/wiggler_output_true.beam";
    const WIGGLER_BEAM_TEST: &str = "tests/wiggler_output_test.beam";

    const CSRCSBEND_BEAM_TRUE: &str = "tests/csrcsbend_output_true.beam";
    const CSRCSBEND_BEAM_TEST: &str = "tests/csrcsbend_output_test.beam";

    const RBEN_BEAM_TRUE: &str = "tests/rben_output_true.beam";
    const RBEN_BEAM_TEST: &str = "tests/rben_output_test.beam";

    const SBEN_BEAM_TRUE: &str = "tests/sben_output_true.beam";
    const SBEN_BEAM_TEST: &str = "tests/sben_output_test.beam";

    const KSEXT_BEAM_TRUE: &str = "tests/ksext_output_true.beam";
    const KSEXT_BEAM_TEST: &str = "tests/ksext_output_test.beam";

    const SCRAPER_BEAM_TRUE: &str = "tests/scraper_output_true.beam";
    const SCRAPER_BEAM_TEST: &str = "tests/scraper_output_test.beam";

    const ECOL_BEAM_TRUE: &str = "tests/ecol_output_true.beam";
    const ECOL_BEAM_TEST: &str = "tests/ecol_output_test.beam";

    const MONITOR_BEAM_TRUE: &str = "tests/monitor_output_true.beam";
    const MONITOR_BEAM_TEST: &str = "tests/monitor_output_test.beam";

    const MONI_BEAM_TRUE: &str = "tests/moni_output_true.beam";
    const MONI_BEAM_TEST: &str = "tests/moni_output_test.beam";

    const SPF_BEAM_TRUE: &str = "tests/spf_output_true.beam";
    const SPF_BEAM_TEST: &str = "tests/spf_output_test.beam";

    #[test]
    fn track_thru_drift() {
        let mut sim: Simulation = load_elegant_file(ELEGANT_TESTFILE, "DRIFT");
        let newsim = load_lotr_file(BEAM_TESTFILE);
        sim.input_beam = newsim.input_beam;
        sim.rescale_acc_energy(newsim.input_beam_ke);
        sim.track();
        if let Ok(mut file) = File::create(DRIFT_BEAM_TEST) {
            print_beam(&mut file, &sim.output_beam);
        }

        let mut file_true = File::open(DRIFT_BEAM_TRUE).unwrap();
        let mut file_test = File::open(DRIFT_BEAM_TEST).unwrap();
        assert!(diff_files(&mut file_true, &mut file_test));
    }

    #[test]
    fn track_thru_sbend() {
        let mut sim: Simulation = load_elegant_file(ELEGANT_TESTFILE, "SBEND");
        let newsim = load_lotr_file(BEAM_TESTFILE);
        sim.input_beam = newsim.input_beam;
        sim.rescale_acc_energy(newsim.input_beam_ke);
        sim.track();
        if let Ok(mut file) = File::create(SBEND_BEAM_TEST) {
            print_beam(&mut file, &sim.output_beam);
        }

        let mut file_true = File::open(SBEND_BEAM_TRUE).unwrap();
        let mut file_test = File::open(SBEND_BEAM_TEST).unwrap();
        assert!(diff_files(&mut file_true, &mut file_test));
    }

    #[test]
    fn track_thru_marker() {
        let mut sim: Simulation = load_elegant_file(ELEGANT_TESTFILE, "MARKER");
        let newsim = load_lotr_file(BEAM_TESTFILE);
        sim.input_beam = newsim.input_beam;
        sim.rescale_acc_energy(newsim.input_beam_ke);
        sim.track();
        if let Ok(mut file) = File::create(MARKER_BEAM_TEST) {
            print_beam(&mut file, &sim.output_beam);
        }

        let mut file_true = File::open(MARKER_BEAM_TRUE).unwrap();
        let mut file_test = File::open(MARKER_BEAM_TEST).unwrap();
        assert!(diff_files(&mut file_true, &mut file_test));
    }

    #[test]
    fn track_thru_hkick() {
        let mut sim: Simulation = load_elegant_file(ELEGANT_TESTFILE, "HKICK");
        let newsim = load_lotr_file(BEAM_TESTFILE);
        sim.input_beam = newsim.input_beam;
        sim.rescale_acc_energy(newsim.input_beam_ke);
        sim.track();
        if let Ok(mut file) = File::create(HKICK_BEAM_TEST) {
            print_beam(&mut file, &sim.output_beam);
        }

        let mut file_true = File::open(HKICK_BEAM_TRUE).unwrap();
        let mut file_test = File::open(HKICK_BEAM_TEST).unwrap();
        assert!(diff_files(&mut file_true, &mut file_test));
    }

    #[test]
    fn track_thru_vkick() {
        let mut sim: Simulation = load_elegant_file(ELEGANT_TESTFILE, "VKICK");
        let newsim = load_lotr_file(BEAM_TESTFILE);
        sim.input_beam = newsim.input_beam;
        sim.rescale_acc_energy(newsim.input_beam_ke);
        sim.track();
        if let Ok(mut file) = File::create(VKICK_BEAM_TEST) {
            print_beam(&mut file, &sim.output_beam);
        }

        let mut file_true = File::open(VKICK_BEAM_TRUE).unwrap();
        let mut file_test = File::open(VKICK_BEAM_TEST).unwrap();
        assert!(diff_files(&mut file_true, &mut file_test));
    }

    #[test]
    fn track_thru_kquad() {
        let mut sim: Simulation = load_elegant_file(ELEGANT_TESTFILE, "KQUAD");
        let newsim = load_lotr_file(BEAM_TESTFILE);
        sim.input_beam = newsim.input_beam;
        sim.rescale_acc_energy(newsim.input_beam_ke);
        sim.track();
        if let Ok(mut file) = File::create(KQUAD_BEAM_TEST) {
            print_beam(&mut file, &sim.output_beam);
        }

        let mut file_true = File::open(KQUAD_BEAM_TRUE).unwrap();
        let mut file_test = File::open(KQUAD_BEAM_TEST).unwrap();
        assert!(diff_files(&mut file_true, &mut file_test));
    }

    #[test]
    fn track_thru_rfcw() {
        let mut sim: Simulation = load_elegant_file(ELEGANT_TESTFILE, "RFCW");
        let newsim = load_lotr_file(BEAM_TESTFILE);
        sim.input_beam = newsim.input_beam;
        sim.rescale_acc_energy(newsim.input_beam_ke);
        sim.track();
        if let Ok(mut file) = File::create(RFCW_BEAM_TEST) {
            print_beam(&mut file, &sim.output_beam);
        }

        let mut file_true = File::open(RFCW_BEAM_TRUE).unwrap();
        let mut file_test = File::open(RFCW_BEAM_TEST).unwrap();
        assert!(diff_files(&mut file_true, &mut file_test));
    }

    #[test]
    fn track_thru_rfdf() {
        let mut sim: Simulation = load_elegant_file(ELEGANT_TESTFILE, "RFDF");
        let newsim = load_lotr_file(BEAM_TESTFILE);
        sim.input_beam = newsim.input_beam;
        sim.rescale_acc_energy(newsim.input_beam_ke);
        sim.track();
        if let Ok(mut file) = File::create(RFDF_BEAM_TEST) {
            print_beam(&mut file, &sim.output_beam);
        }

        let mut file_true = File::open(RFDF_BEAM_TRUE).unwrap();
        let mut file_test = File::open(RFDF_BEAM_TEST).unwrap();
        assert!(diff_files(&mut file_true, &mut file_test));
    }

    #[test]
    fn track_thru_wiggler() {
        let mut sim: Simulation = load_elegant_file(ELEGANT_TESTFILE, "WIGGLER");
        let newsim = load_lotr_file(BEAM_TESTFILE);
        sim.input_beam = newsim.input_beam;
        sim.rescale_acc_energy(newsim.input_beam_ke);
        sim.track();
        if let Ok(mut file) = File::create(WIGGLER_BEAM_TEST) {
            print_beam(&mut file, &sim.output_beam);
        }

        let mut file_true = File::open(WIGGLER_BEAM_TRUE).unwrap();
        let mut file_test = File::open(WIGGLER_BEAM_TEST).unwrap();
        assert!(diff_files(&mut file_true, &mut file_test));
    }

    #[test]
    fn track_thru_csrcsbend() {
        let mut sim: Simulation = load_elegant_file(ELEGANT_TESTFILE, "CSRCSBEND");
        let newsim = load_lotr_file(BEAM_TESTFILE);
        sim.input_beam = newsim.input_beam;
        sim.rescale_acc_energy(newsim.input_beam_ke);
        sim.track();
        if let Ok(mut file) = File::create(CSRCSBEND_BEAM_TEST) {
            print_beam(&mut file, &sim.output_beam);
        }

        if let Ok(mut file_true) = File::open(CSRCSBEND_BEAM_TRUE) {
            let mut file_test = File::open(CSRCSBEND_BEAM_TEST).unwrap();
            assert!(diff_files(&mut file_true, &mut file_test));
        } else {
            panic!("No file to compare against");
        }
    }

    #[test]
    fn track_thru_rben() {
        let mut sim: Simulation = load_elegant_file(ELEGANT_TESTFILE, "RBEN");
        let newsim = load_lotr_file(BEAM_TESTFILE);
        sim.input_beam = newsim.input_beam;
        sim.rescale_acc_energy(newsim.input_beam_ke);
        sim.track();
        if let Ok(mut file) = File::create(RBEN_BEAM_TEST) {
            print_beam(&mut file, &sim.output_beam);
        }

        let mut file_true = File::open(RBEN_BEAM_TRUE).unwrap();
        let mut file_test = File::open(RBEN_BEAM_TEST).unwrap();
        assert!(diff_files(&mut file_true, &mut file_test));
    }

    #[test]
    fn track_thru_sben() {
        let mut sim: Simulation = load_elegant_file(ELEGANT_TESTFILE, "SBEN");
        let newsim = load_lotr_file(BEAM_TESTFILE);
        sim.input_beam = newsim.input_beam;
        sim.rescale_acc_energy(newsim.input_beam_ke);
        sim.track();
        if let Ok(mut file) = File::create(SBEN_BEAM_TEST) {
            print_beam(&mut file, &sim.output_beam);
        }

        let mut file_true = File::open(SBEN_BEAM_TRUE).unwrap();
        let mut file_test = File::open(SBEN_BEAM_TEST).unwrap();
        assert!(diff_files(&mut file_true, &mut file_test));
    }

    #[test]
    fn track_thru_ksext() {
        let mut sim: Simulation = load_elegant_file(ELEGANT_TESTFILE, "KSEXT");
        let newsim = load_lotr_file(BEAM_TESTFILE);
        sim.input_beam = newsim.input_beam;
        sim.rescale_acc_energy(newsim.input_beam_ke);
        sim.track();
        if let Ok(mut file) = File::create(KSEXT_BEAM_TEST) {
            print_beam(&mut file, &sim.output_beam);
        }

        let mut file_true = File::open(KSEXT_BEAM_TRUE).unwrap();
        let mut file_test = File::open(KSEXT_BEAM_TEST).unwrap();
        assert!(diff_files(&mut file_true, &mut file_test));
    }

    #[test]
    fn track_thru_scraper() {
        let mut sim: Simulation = load_elegant_file(ELEGANT_TESTFILE, "SCRAPER");
        let newsim = load_lotr_file(BEAM_TESTFILE);
        sim.input_beam = newsim.input_beam;
        sim.rescale_acc_energy(newsim.input_beam_ke);
        sim.track();
        if let Ok(mut file) = File::create(SCRAPER_BEAM_TEST) {
            print_beam(&mut file, &sim.output_beam);
        }

        let mut file_true = File::open(SCRAPER_BEAM_TRUE).unwrap();
        let mut file_test = File::open(SCRAPER_BEAM_TEST).unwrap();
        assert!(diff_files(&mut file_true, &mut file_test));
    }

    #[test]
    fn track_thru_ecol() {
        let mut sim: Simulation = load_elegant_file(ELEGANT_TESTFILE, "ECOL");
        let newsim = load_lotr_file(BEAM_TESTFILE);
        sim.input_beam = newsim.input_beam;
        sim.rescale_acc_energy(newsim.input_beam_ke);
        sim.track();
        if let Ok(mut file) = File::create(ECOL_BEAM_TEST) {
            print_beam(&mut file, &sim.output_beam);
        }

        let mut file_true = File::open(ECOL_BEAM_TRUE).unwrap();
        let mut file_test = File::open(ECOL_BEAM_TEST).unwrap();
        assert!(diff_files(&mut file_true, &mut file_test));
    }

    #[test]
    fn track_thru_monitor() {
        let mut sim: Simulation = load_elegant_file(ELEGANT_TESTFILE, "MONITOR");
        let newsim = load_lotr_file(BEAM_TESTFILE);
        sim.input_beam = newsim.input_beam;
        sim.rescale_acc_energy(newsim.input_beam_ke);
        sim.track();
        if let Ok(mut file) = File::create(MONITOR_BEAM_TEST) {
            print_beam(&mut file, &sim.output_beam);
        }

        let mut file_true = File::open(MONITOR_BEAM_TRUE).unwrap();
        let mut file_test = File::open(MONITOR_BEAM_TEST).unwrap();
        assert!(diff_files(&mut file_true, &mut file_test));
    }

    #[test]
    fn track_thru_moni() {
        let mut sim: Simulation = load_elegant_file(ELEGANT_TESTFILE, "MONI");
        let newsim = load_lotr_file(BEAM_TESTFILE);
        sim.input_beam = newsim.input_beam;
        sim.rescale_acc_energy(newsim.input_beam_ke);
        sim.track();
        if let Ok(mut file) = File::create(MONI_BEAM_TEST) {
            print_beam(&mut file, &sim.output_beam);
        }

        let mut file_true = File::open(MONI_BEAM_TRUE).unwrap();
        let mut file_test = File::open(MONI_BEAM_TEST).unwrap();
        assert!(diff_files(&mut file_true, &mut file_test));
    }

    #[test]
    fn track_thru_spf() {
        let mut sim: Simulation = load_elegant_file(SPF_TESTFILE, "SPF");
        let newsim = load_lotr_file(BEAM_TESTFILE);
        sim.input_beam = newsim.input_beam;
        sim.rescale_acc_energy(newsim.input_beam_ke);
        sim.track();
        if let Ok(mut file) = File::create(SPF_BEAM_TEST) {
            print_beam(&mut file, &sim.output_beam);
        }

        let mut file_true = File::open(SPF_BEAM_TRUE).unwrap();
        let mut file_test = File::open(SPF_BEAM_TEST).unwrap();
        assert!(diff_files(&mut file_true, &mut file_test));
    }
}
