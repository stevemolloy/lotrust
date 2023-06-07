use crate::beam::print_beam;
use crate::parse_elegant::load_elegant_file;
use crate::parse_lotr::{load_lotr_file, Simulation};
use std::collections::VecDeque;
use std::env;
use std::process::exit;

mod beam;
mod elegant_rpn;
mod elements;
mod parse_elegant;
mod parse_lotr;

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

    // println!("{:#?}", simulation.elements);

    println!("---   INPUT  ---");
    print_beam(&simulation.beam);
    println!("--- TRACKING ---");

    let outfile = if args.len() > 2 {
        Some(String::from(&args[2]))
    } else {
        None
    };

    simulation.track(outfile);

    println!("---  OUTPUT  ---");
    print_beam(&simulation.beam);
    println!("---   DONE   ---");

    // TODO(#7): The output definition of energy error is different from the input. Fix this.
}
