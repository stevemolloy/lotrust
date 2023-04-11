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
    let mut simulation: Simulation = load_lotr_file(filename);

    println!("---   INPUT  ---");
    for electron in &simulation.beam {
        println!(
            "{:0.6} ps :: {:0.3} MeV",
            electron.t * 1e12,
            electron.ke * 1e-6
        );
    }
    println!("--- TRACKING ---");
    simulation.track();
    println!("---  OUTPUT  ---");

    for electron in simulation.beam {
        println!(
            "{:0.6} ps :: {:0.3} MeV",
            electron.t * 1e12,
            electron.ke * 1e-6
        );
    }
}
