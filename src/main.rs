use crate::beam::print_beam;
use crate::parse_elegant::load_elegant_file;
use crate::parse_lotr::{load_lotr_file, Simulation};
use std::env;
use std::process::exit;

mod beam;
mod elegant_rpn;
mod elements;
mod parse_elegant;
mod parse_lotr;
mod elegant_rpn;

fn usage() {
    println!("Please give the name of the LOTR file to use.\n
    Optionally, providing an additional file name will run the program in data preservation mode, outputting all data to that file.");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let valid_filename = args[1].ends_with(".lotr") || args[1].ends_with(".lte");
    if !valid_filename || args.len() < 2 || args.len() > 3 {
        usage();
        exit(1);
    }

    let filename = &args[1];

    // TODO(#8): Should be able to read elegant lte files
    let mut simulation: Simulation = if filename.ends_with(".lte") {
        load_elegant_file(filename)
    } else {
        load_lotr_file(filename)
    };

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
