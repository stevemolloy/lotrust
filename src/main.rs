use crate::beam::print_beam;
use crate::parse_lotr::{load_lotr_file, Simulation};
use std::env;
use std::process::exit;

mod beam;
mod elements;
mod parse_lotr;

fn usage() {
    println!("Please give the name of the LOTR file to use.");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        usage();
        exit(1);
    }
    let filename = &args[1];

    // TODO(#8): Should be able to read elegant lte files
    let mut simulation: Simulation = load_lotr_file(filename);

    println!("---   INPUT  ---");
    print_beam(&simulation.beam);
    println!("--- TRACKING ---");
    simulation.track();
    println!("---  OUTPUT  ---");
    print_beam(&simulation.beam);
    println!("---   DONE   ---");

    // TODO(#7): The output definition of energy error is different from the input. Fix this.
}
