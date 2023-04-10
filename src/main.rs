use crate::parse_lotr::{Simulation, tokenize_file_contents, parse_tokens};

pub mod beam;
pub mod elements;
pub mod parse_lotr;

fn main() {
    let filename = "acc_defn.lotr";
    let tokens = tokenize_file_contents(filename);
    let mut simulation: Simulation = parse_tokens(&tokens);

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
