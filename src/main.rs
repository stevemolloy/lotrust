use crate::parse_lotr::{load_lotr_file, Simulation};

mod beam;
mod elements;
mod parse_lotr;

fn main() {
    let filename = "acc_defn.lotr";
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
