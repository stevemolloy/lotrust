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
            EleType::AccCav(details) => {
                let gamma0_i = ele.gamma;
                let beta0_i = gamma_2_beta(gamma0_i);
                let gamma0_f =
                    ke_2_gamma(gamma_2_ke(gamma0_i) + details.voltage * details.phase.cos());

                let r56_drift = (details.length / 2f64) / (beta0_i.powi(2) * gamma0_i.powi(2));
                let drift_matrix = arr2(&[[1f64, 0f64], [r56_drift, 1f64]]);

                self.pos = self.pos.dot(&drift_matrix);

                for mut particle in self.pos.outer_iter_mut() {
                    let actual_phase = details.phase - particle[0] * details.wavenumber;
                    let new_ke =
                        delta_2_ke(particle[1], gamma0_i) + (details.voltage * actual_phase.cos());
                    let new_gamma = ke_2_gamma(new_ke);

                    particle[1] = gamma_2_delta(new_gamma, gamma0_f);
                }

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

fn gamma_2_delta(gamma: f64, gamma0: f64) -> f64 {
    let beta0 = gamma_2_beta(gamma0);
    (1f64 / beta0) * ((gamma / gamma0) - 1f64)
}

fn delta_2_gamma(delta: f64, gamma0: f64) -> f64 {
    let beta0 = gamma_2_beta(gamma0);
    beta0 * gamma0 * delta + gamma0
}

fn delta_2_ke(delta: f64, gamma0: f64) -> f64 {
    gamma_2_ke(delta_2_gamma(delta, gamma0))
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

    #[test]
    fn zero_delta_means_gamma_is_gamma0() {
        let delta = 0f64;
        let gamma0 = 100f64;
        assert_eq!(delta_2_gamma(delta, gamma0), gamma0);
    }

    #[test]
    fn gamma_is_gamma0_implies_zero_delta() {
        let gamma0 = 100f64;
        let gamma = gamma0;
        assert_eq!(gamma_2_delta(gamma, gamma0), 0f64);
    }
}
