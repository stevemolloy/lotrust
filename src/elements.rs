use crate::beam::{gamma_2_beta, Beam, C, MASS};
use core::fmt::Debug;
use ndarray::{arr2, Array2};
use std::f64::consts::PI;
use std::fmt::{Display, Formatter, Result};
use std::process::exit;

// TODO(#2): Beam should (?) be resorted when tracked by an element that may reorder things.
// Which elements could reorder particles? Dipoles.  AccCavs, but not in the linear approx.
pub trait Tracking {
    fn track(&self, beam: &mut Beam);
    fn ele_type(&self) -> String;
}

impl Display for dyn Tracking {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.ele_type())
    }
}

impl Debug for dyn Tracking {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:}", self.ele_type())
    }
}

// TODO(#3): Add various diag elements that act on the beam as drifts, but produce side-effects.
#[derive(Default)]
pub struct Drift {
    name: String,
    l: f64,
    t_matrix: Array2<f64>,
}

impl Drift {
    pub fn new(name: String, l: f64, g: f64) -> Drift {
        let beta_sq = gamma_2_beta(g).powi(2);
        let gamma_sq = g.powi(2);
        let r56 = l / (beta_sq * gamma_sq);
        Drift {
            name,
            l,
            t_matrix: arr2(&[[1f64, r56], [0f64, 1f64]]),
        }
    }
}

impl Tracking for Drift {
    fn track(&self, beam: &mut Beam) {
        *beam = beam.dot(&self.t_matrix.t());
    }

    fn ele_type(&self) -> String {
        format!("Drift ({}: l->{})", self.name, self.l)
    }
}

pub type Corr = Drift;
pub type Quad = Drift;
pub type Sext = Drift;

pub struct Dipole {
    name: String,
    t_matrix: Array2<f64>,
    l: f64,
    angle: f64,
}

impl Dipole {
    pub fn new(name: String, l: f64, angle: f64, g: f64) -> Dipole {
        if l == 0f64 {
            eprintln!("Path length through a dipole should not be negative or zero");
            exit(1);
        }
        let angle_fixed = if angle == 0f64 {
            f64::MIN_POSITIVE
        } else {
            angle
        };
        let omega = angle_fixed / l;
        let beta_sq = gamma_2_beta(g).powi(2);
        let gamma_sq = g.powi(2);
        let r56 = l / (beta_sq * gamma_sq) - (angle_fixed - angle_fixed.sin()) / (omega * beta_sq);
        if angle_fixed == 0f64 {
            println!("r56 for zero angle_fixed bend = {}", r56);
        }
        Dipole {
            name,
            t_matrix: arr2(&[[1f64, r56], [0f64, 1f64]]),
            l,
            angle: angle_fixed,
        }
    }
}

impl Tracking for Dipole {
    fn track(&self, beam: &mut Beam) {
        *beam = beam.dot(&self.t_matrix.t());
    }

    fn ele_type(&self) -> String {
        format!(
            "Dipole ({}: l->{}, angle->{})",
            self.name, self.l, self.angle
        )
    }
}

// TODO(#4): Accelerating cavities need to have wakefields in their physics.
pub struct AccCav {
    name: String,
    l: f64,
    v: f64,
    freq: f64,
    phi: f64,
    drift_matrix: Array2<f64>,
    kick_matrix: Array2<f64>,
}

impl AccCav {
    pub fn new(name: String, l: f64, v: f64, freq: f64, phi: f64, g: f64) -> AccCav {
        let beta_sq = gamma_2_beta(g).powi(2);
        let gamma_sq = g.powi(2);
        let r56_drift = l / (beta_sq * gamma_sq);

        let k = 2f64 * PI * freq / C;
        let r65_kick = -k * l * v * phi.sin() / (g * MASS);
        AccCav {
            name,
            l,
            v,
            freq,
            phi,
            drift_matrix: arr2(&[[1f64, r56_drift], [0f64, 1f64]]),
            kick_matrix: arr2(&[[1f64, 0f64], [r65_kick, 1f64]]),
        }
    }
}

// TODO(#5): Instead of sorting the particles by z all the time, perhaps only do it here?
impl Tracking for AccCav {
    fn track(&self, beam: &mut Beam) {
        *beam = beam.dot(&self.drift_matrix.t());
        *beam = beam.dot(&self.kick_matrix.t());
        *beam = beam.dot(&self.drift_matrix.t());
    }

    fn ele_type(&self) -> String {
        format!(
            "AccCav ({}: l->{}, v->{}, freq->{}, phi->{})",
            self.name, self.l, self.v, self.freq, self.phi
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::{assert_relative_eq, assert_ulps_eq}; // for floating point tests
    const GAMMA0: f64 = 3000f64;

    #[test]
    fn dipole_does_not_affect_energy_error() {
        let b_field = 2.0;
        let angle = 0.7;
        let dipole = Dipole::new("dipole".to_string(), b_field, angle, GAMMA0);
        for e_error in [-0.01, -0.005, -0.001, 0.0, 0.001, 0.005, 0.01] {
            for z in [-5e-3, -1e-3, 0.0, 1e-3, 5e-3] {
                let mut beam_vec = Array2::from(vec![[z, (1f64 / gamma_2_beta(GAMMA0)) * e_error]]);
                dipole.track(&mut beam_vec);
                assert_eq!(beam_vec[[0, 1]], (1f64 / gamma_2_beta(GAMMA0)) * e_error);
            }
        }
    }

    #[test]
    fn dipole_alters_z_correctly() {
        let length = 0.75;
        let angle = 0.7;
        let dipole = Dipole::new("dipole".to_string(), length, angle, GAMMA0);
        let beta0 = gamma_2_beta(GAMMA0);
        for rel_e_err in [-0.01, -0.005, -0.001, 0.0, 0.001, 0.005, 0.01] {
            let gamma_delta = rel_e_err;
            let omega = angle / length;
            let omega_l = angle.abs();
            let first_term = length * (gamma_delta / (GAMMA0.powi(2) * beta0.powi(3)));
            let second_term = (omega_l - omega_l.sin()) / (omega * beta0.powi(2));
            let delta_z = (first_term - second_term) * rel_e_err / beta0;
            for z in [-5e-3, -1e-3, 0.0, 1e-3, 5e-3] {
                let mut beam_vec = Array2::from(vec![[z, (1f64 / beta0) * rel_e_err]]);
                dipole.track(&mut beam_vec);
                // TODO(#6): Why does this test need max_relative = 1e-5 ?
                assert_relative_eq!(
                    beam_vec[[0, 0]],
                    z + delta_z,
                    max_relative = 1e-5,
                    epsilon = f64::EPSILON
                );
            }
        }
    }

    #[test]
    fn drift_does_not_affect_energy_error() {
        let drift = Drift::new("drift".to_string(), 2f64, 10f64);
        for e_error in [-0.01, -0.005, -0.001, 0.0, 0.001, 0.005, 0.01] {
            for z in [-5e-3, -1e-3, 0.0, 1e-3, 5e-3] {
                let mut beam_vec = Array2::from(vec![[z, e_error]]);
                drift.track(&mut beam_vec);
                assert_eq!(beam_vec[[0, 1]], e_error);
            }
        }
    }

    #[test]
    fn drift_alters_z_correctly() {
        let drift_l = 1f64;
        let beta0 = gamma_2_beta(GAMMA0);
        let drift = Drift::new("drift".to_string(), drift_l, GAMMA0);
        for rel_e_err in [-0.01, -0.005, -0.001, 0.0, 0.001, 0.005, 0.01] {
            let gamma_delta = rel_e_err;
            let delta_z = drift_l * (gamma_delta / (GAMMA0.powi(2) * beta0.powi(3)));
            for z in [-5e-3, -1e-3, 0.0, 1e-3, 5e-3] {
                let mut beam_vec = Array2::from(vec![[z, (1f64 / beta0) * rel_e_err]]);
                drift.track(&mut beam_vec);
                assert_ulps_eq!(
                    beam_vec[[0, 0]],
                    z + delta_z,
                    epsilon = f64::EPSILON,
                    max_ulps = 1
                );
            }
        }
    }

    #[test]
    fn quad_does_not_affect_energy_error() {
        let quad = Quad::new("quad".to_string(), 2f64, 10f64);
        for e_error in [-0.01, -0.005, -0.001, 0.0, 0.001, 0.005, 0.01] {
            for z in [-5e-3, -1e-3, 0.0, 1e-3, 5e-3] {
                let mut beam_vec = Array2::from(vec![[z, e_error]]);
                quad.track(&mut beam_vec);
                assert_eq!(beam_vec[[0, 1]], e_error);
            }
        }
    }

    #[test]
    fn quad_alters_z_correctly() {
        let quad_l = 1f64;
        let beta0 = gamma_2_beta(GAMMA0);
        let quad = Quad::new("quad".to_string(), quad_l, GAMMA0);
        for rel_e_err in [-0.01, -0.005, -0.001, 0.0, 0.001, 0.005, 0.01] {
            let gamma_delta = rel_e_err;
            let delta_z = quad_l * (gamma_delta / (GAMMA0.powi(2) * beta0.powi(3)));
            for z in [-5e-3, -1e-3, 0.0, 1e-3, 5e-3] {
                let mut beam_vec = Array2::from(vec![[z, (1f64 / beta0) * rel_e_err]]);
                quad.track(&mut beam_vec);
                assert_ulps_eq!(
                    beam_vec[[0, 0]],
                    z + delta_z,
                    epsilon = f64::EPSILON,
                    max_ulps = 1
                );
            }
        }
    }
}
