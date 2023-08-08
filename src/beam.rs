use crate::elements::EleType;
use ndarray::{arr2, s, Array2, Axis};
use std::io::Write;

use crate::elements::Element;
pub const MASS: f64 = 510998.9499961642f64;
pub const C: f64 = 299792458f64;

// TODO(#1): The beam should (?) be sorted by the z coord
#[derive(Clone)]
pub struct Beam {
    pub pos: Array2<f64>,
}

impl Beam {
    pub fn new(pos: Array2<f64>) -> Self {
        Self { pos }
    }

    pub fn track(&mut self, ele: &Element) {
        match ele.ele_type {
            EleType::Drift | EleType::Dipole => {
                let r56 = match ele.params.get("r56") {
                    Some(val) => *val,
                    None => 0f64,
                };
                let t_matrix = arr2(&[[1f64, 0f64], [r56, 1f64]]);
                self.pos = self.pos.dot(&t_matrix);
            }
            EleType::AccCav => {
                for particle in self.pos.outer_iter() {
                    let neg_matrix = arr2(&[[-1f64, 0f64], [0f64, -1f64]]);
                    println!("particle details: {:#?}", neg_matrix.dot(&particle));
                    println!("particle details: {:#?}", particle);
                }
                let synchro_ke = gamma_2_ke(ele.gamma);
                let synchro_ke_gain = ele.params["v"] * ele.params["phi"].cos();
                let new_synchro_ke = synchro_ke + synchro_ke_gain;

                let e_err_mat = arr2(&[[1f64, 0f64], [0f64, synchro_ke / new_synchro_ke]]);

                let r56_drift = match ele.params.get("r56_drift") {
                    Some(val) => *val,
                    None => 0f64,
                };
                let drift_matrix = arr2(&[[1f64, 0f64], [r56_drift, 1f64]]);

                let r65_kick = match ele.params.get("r65_kick") {
                    Some(val) => *val,
                    None => 0f64,
                };
                let kick_matrix = arr2(&[[1f64, r65_kick], [0f64, 1f64]]);

                self.pos = self.pos.dot(&drift_matrix);
                self.pos = self.pos.dot(&e_err_mat);
                self.pos = self.pos.dot(&kick_matrix);
                self.pos = self.pos.dot(&drift_matrix);
            }
        }
    }
}

pub fn print_beam(sink: &mut impl Write, beam: &Beam) {
    let num_electrons = beam.pos.len_of(Axis(0));
    for e_num in 0..num_electrons {
        let this_electron = beam.pos.slice(s![e_num, ..]);
        if let Err(e) = writeln!(sink, "{}, {}", this_electron[0], this_electron[1]) {
            println!("ERROR: {e}");
        }
    }
}

pub fn ke_2_gamma(ke: f64) -> f64 {
    ke / MASS + 1f64
}

pub fn gamma_2_ke(gamma: f64) -> f64 {
    (gamma - 1f64) * MASS
}

pub fn gamma_2_beta(g: f64) -> f64 {
    (1f64 - (1f64 / g.powi(2))).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ke_of_restmass_has_gamma_two() {
        let ke = MASS;
        assert_eq!(ke_2_gamma(ke), 2.0);
    }

    #[test]
    fn zero_ke_has_unity_gamma() {
        let ke = 0f64;
        assert_eq!(ke_2_gamma(ke), 1.0);
    }

    #[test]
    fn zero_ke_has_zero_beta() {
        let ke = 0f64;
        assert_eq!(gamma_2_beta(ke_2_gamma(ke)), 0.0);
    }
}
