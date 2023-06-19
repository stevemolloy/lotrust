use crate::beam::print_beam;
use crate::parse_elegant::load_elegant_file;
use crate::parse_lotr::{load_lotr_file, Simulation};
use crossterm::{cursor, terminal, ExecutableCommand};
use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;
use std::process::exit;
use std::{env, io};

mod beam;
mod elegant_rpn;
mod elements;
mod parse_elegant;
mod parse_lotr;

const ENERGYPROFFILENAME: &str = "energy_profile.csv";
const ACCELERATORFILENAME: &str = "acceleratorout.lotr";

#[derive(Clone, PartialEq)]
enum Token {
    Exit,
    Track,
    Error,
    Print,
    Save,
    LoadLattice,
    LoadBeam,
    AddBreakPoint,
    Reset,
    Help,
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

fn out_energyprofile(sink: &mut impl Write, state: &State) {
    let mut z = 0f64;
    for (ind, ele) in state.simulation.elements.iter().enumerate() {
        if let Err(e) = writeln!(sink, "{}, {}, {}", ind, z, ele.gamma) {
            println!("{}", e);
            break;
        }
        z += ele.length;
    }
}

fn lex(text: &str) -> Token {
    match text {
        "exit" | "quit" => Token::Exit,
        "track" => Token::Track,
        "print" => Token::Print,
        "save" => Token::Save,
        "load_lattice" => Token::LoadLattice,
        "load_beam" => Token::LoadBeam,
        "break" => Token::AddBreakPoint,
        "reset" => Token::Reset,
        "help" => Token::Help,
        _ => {
            println!("ERROR: Cannot understand token: {}", text);
            Token::Error
        }
    }
}

fn parse_input(text: &str, mut state: State) -> State {
    let mut items: VecDeque<&str> = text.split_whitespace().collect();
    while !items.is_empty() {
        let item = items.pop_front().unwrap();
        match lex(item) {
            Token::Help => {
                println!("help                    :: Print this message.");
                println!("exit|quit               :: End the program.");
                println!("track                   :: Track the beam through the accelerator, stopping at the first breakpoint (if defined) or the end of the accelerator.");
                println!("load_lattice <filename> :: Load a new accelerator from 'filename'.");
                println!("load_beam <filename>    :: Load a new input beam from 'filename'");
                println!("break <element_name>    :: Add a breakpoint to the first element named 'element_name'");
                println!("reset                   :: Remove all breakpoints, reset tracking status to the start of the accelerator, and reset the output beam.");
                println!("save <param>            :: Saves 'param' to a pre-defined file.  'param' may be one of the following:");
                println!("                                        * 'input_beam'");
                println!("                                        * 'output_beam'");
                println!("                                        * 'accelerator'");
                println!("                                        * 'energy_profile'");
                println!("print <param>           :: Prints 'param' to the screen.  'param' may be one of those defined for the 'save' command (above).");
            }
            Token::Error => break,
            Token::Exit => state.running = false,
            Token::Track => {
                state.simulation.track();
            }
            Token::LoadLattice => {
                if items.is_empty() {
                    println!("ERROR: Loading a lattice file requires specifying a filename.");
                    println!("       load_lattice <filename> [<elegant_line>]");
                    break;
                }
                let filename = items.pop_front().unwrap();
                let newsim: Simulation;
                if filename.ends_with("lte") {
                    if items.is_empty() {
                        println!("ERROR: Loading an elegant file requires also specifying which line to use.");
                        println!("       load_lattice <elegantfilename> <elegant_line>");
                        break;
                    }
                    let elegant_line = items.pop_front().unwrap();
                    newsim = load_elegant_file(filename, elegant_line);
                    state.simulation.elements = newsim.elements;
                } else {
                    newsim = load_lotr_file(filename);
                    state.simulation.elements = newsim.elements;
                };
            }
            Token::LoadBeam => {
                if items.is_empty() {
                    println!("ERROR: Loading a beam file requires specifying a filename.");
                    println!("       load_beam <filename>");
                    break;
                }
                let filename = items.pop_front().unwrap();
                let newsim: Simulation = load_lotr_file(filename);
                state.simulation.input_beam = newsim.input_beam;
            }
            Token::Print => {
                if items.is_empty() {
                    println!("ERROR: Expected additional input after the 'print' command");
                    break;
                }
                let print_what = items.pop_front().unwrap();
                match print_what {
                    "input_beam" => print_beam(&mut io::stdout(), &state.simulation.input_beam),
                    "output_beam" => print_beam(&mut io::stdout(), &state.simulation.output_beam),
                    "accelerator" => {
                        if let Err(e) =
                            writeln!(&mut io::stdout(), "{:?}", state.simulation.elements)
                        {
                            println!("Could not write to stdout...: {e}");
                        }
                    }
                    "energy_profile" => out_energyprofile(&mut io::stdout(), &state),
                    _ => println!("ERROR: Cannot understand '{print_what}'"),
                }
            }
            Token::Save => {
                if items.is_empty() {
                    println!("ERROR: Expected additional input after the 'save' command");
                    break;
                }
                let save_what = items.pop_front().unwrap();
                match save_what {
                    "input_beam" => print_beam(&mut io::stdout(), &state.simulation.input_beam),
                    "output_beam" => print_beam(&mut io::stdout(), &state.simulation.output_beam),
                    "accelerator" => {
                        if let Ok(mut file) = File::create(ACCELERATORFILENAME) {
                            if let Err(e) = writeln!(&mut file, "{:?}", state.simulation.elements) {
                                println!("ERROR: Could not write the file: {e}");
                            }
                        } else {
                            println!("ERROR: Could not write the file");
                        }
                    }
                    "energy_profile" => {
                        if let Ok(mut file) = File::create(ENERGYPROFFILENAME) {
                            out_energyprofile(&mut file, &state);
                        } else {
                            println!("ERROR: Could not write the file");
                        }
                    }
                    _ => println!("ERROR: Cannot understand '{save_what}'"),
                }
            }
            Token::AddBreakPoint => {
                if items.is_empty() {
                    println!("ERROR: Expected additional input after the 'break' command");
                    break;
                }
                let breakpoint_name = items.pop_front().unwrap().to_string();
                if let Some(pos) = state.simulation.find_element_by_name(breakpoint_name) {
                    state.simulation.breakpoints.push(pos);
                    println!(
                        "Break point added at element #{pos} ({})",
                        state.simulation.elements.get(pos).unwrap().name
                    );
                }
            }
            Token::Reset => {
                state.simulation.breakpoints_passed = Vec::new();
                state.simulation.breakpoints = Vec::new();
                state.simulation.current = 0;
                state.simulation.output_beam = state.simulation.input_beam.clone();
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
        simulation.input_beam = newsim.input_beam;
    }

    simulation.output_beam = simulation.input_beam.clone();

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
        let prompt = format!(
            "lotrust (ele: {}/{})> ",
            state.simulation.current,
            state.simulation.elements.len()
        );
        stdout.write_all(prompt.as_bytes()).unwrap();
        stdout.flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        state = parse_input(&input, state);

        if !state.running {
            break;
        }
    }

    // TODO(#7): The output definition of energy error is different from the input. Fix this.
}
