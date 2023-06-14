use crate::beam::print_beam;
use crate::parse_elegant::load_elegant_file;
use crate::parse_lotr::{load_lotr_file, Simulation};
use crossterm::{cursor, terminal, ExecutableCommand};
use std::collections::VecDeque;
use std::io::Write;
use std::process::exit;
use std::{env, io};

mod beam;
mod elegant_rpn;
mod elements;
mod parse_elegant;
mod parse_lotr;

#[derive(Clone, PartialEq)]
enum Token {
    Exit,
    Track,
    Error,
}

#[derive(Default)]
struct Options {
    input_filename: String,
    elegant: bool,
    elegant_line: String,
    beam_defined: bool,
    beam_filename: String,
    save_file: bool,
    save_filename: String,
}

struct State {
    running: bool,
    simulation: Simulation,
}

fn lex(text: &str) -> Token {
    match text {
        "exit" | "quit" => Token::Exit,
        "track" => Token::Track,
        _ => {
            println!("ERROR: Cannot understand token: {}", text);
            Token::Error
        }
    }
}

fn parse_input(text: &str, mut state: State) -> State {
    for item in text.split_whitespace() {
        match lex(item) {
            Token::Error => break,
            Token::Exit => state.running = false,
            Token::Track => {
                println!(
                    "Tracking {} particles through {} accelerator elements...",
                    state.simulation.beam.len(),
                    state.simulation.elements.len()
                );
                state.simulation.track(None);
                println!("Done!");
            }
        }
    }

    state
}
fn usage(program_name: String) {
    println!("{program_name} <input_file> [-e line_name] [-b <beam_defn_file>] [-s <output_file>]");
    println!("\tinputfile: The file containing the description of the lattice");
    println!("\t-e: Indicates that the input file is in elegant format. The name of the line to expand must be given");
    println!("\t-b: Overrides any beam definition with that found in <beam_defn_file>");
    println!("\t-s: Saves the output into <output_file>");
}

fn check_options(opts: &Options) -> bool {
    if opts.save_file && opts.save_filename.is_empty() || opts.input_filename.is_empty() {
        return false;
    }
    if opts.beam_defined && opts.beam_filename.is_empty() {
        return false;
    }
    true
}

fn main() {
    let mut options: Options = Default::default();
    let mut args: VecDeque<String> = env::args().collect();

    let program_name: String = args.pop_front().unwrap();

    while !args.is_empty() {
        let next = args.pop_front().unwrap();
        match next.as_str() {
            "-e" => {
                if let Some(linename) = args.pop_front() {
                    options.elegant = true;
                    options.elegant_line = linename;
                } else {
                    usage(program_name);
                    exit(1);
                }
            }
            "-b" => {
                if let Some(beamfile) = args.pop_front() {
                    options.beam_defined = true;
                    options.beam_filename = beamfile;
                } else {
                    usage(program_name);
                    exit(1);
                }
            }
            "-s" => {
                if let Some(outfile) = args.pop_front() {
                    options.save_file = true;
                    options.save_filename = outfile;
                } else {
                    usage(program_name);
                    exit(1);
                }
            }
            _ => options.input_filename = next,
        }
    }

    if !check_options(&options) {
        usage(program_name);
        exit(1);
    }

    // TODO(#8): Should be able to read elegant lte files
    let mut simulation: Simulation = if options.elegant {
        load_elegant_file(&options.input_filename, &options.elegant_line)
    } else {
        load_lotr_file(&options.input_filename)
    };

    if options.beam_defined {
        let newsim: Simulation = load_lotr_file(&options.beam_filename);
        simulation.beam = newsim.beam;
    }

    let mut stdout = io::stdout();
    let stdin = io::stdin();

    stdout
        .execute(terminal::Clear(terminal::ClearType::All))
        .unwrap()
        .execute(cursor::MoveTo(0, 0))
        .unwrap();

    let mut state = State {
        running: true,
        simulation,
    };

    println!("Welcome to LOTR! A Rust powered particle tracker.");

    loop {
        stdout.write_all(b"> ").unwrap();
        stdout.flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        state = parse_input(&input, state);

        if !state.running {
            break;
        }
    }

    // println!("{:#?}", simulation.elements);

    // println!("---   INPUT  ---");
    // print_beam(&simulation.beam);
    // println!("--- TRACKING ---");
    //
    // let outfile = if args.len() > 2 {
    //     Some(String::from(&args[2]))
    // } else {
    //     None
    // };
    //
    // simulation.track(outfile);
    //
    // println!("---  OUTPUT  ---");
    // print_beam(&simulation.beam);
    // println!("---   DONE   ---");

    // TODO(#7): The output definition of energy error is different from the input. Fix this.
}
