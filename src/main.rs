use crate::parse_lotr::{load_lotr_file, Simulation};
use ndarray::{s, Axis};
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
    let mut simulation: Simulation = load_lotr_file(filename);

    println!("---   INPUT  ---");
    let num_electrons = simulation.beam.len_of(Axis(0));
    for e_num in 0..num_electrons {
        let this_electron = simulation.beam.slice(s![e_num, ..]);
        println!(
            // "{:0.3} mm :: {:0.3}",
            "{} mm :: {}",
            this_electron[0] * 1e3,
            this_electron[1]
        );
    }
    println!("--- TRACKING ---");
    simulation.track();
    println!("---  OUTPUT  ---");

    for e_num in 0..num_electrons {
        let this_electron = simulation.beam.slice(s![e_num, ..]);
        println!(
            // "{:0.3} mm :: {:0.3}",
            "{} mm :: {}",
            this_electron[0] * 1e3,
            this_electron[1]
        );
    }
}
