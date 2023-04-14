use crate::beam::{gamma_2_beta, Beam, C, MASS};
use ndarray::{arr2, Array2};
use std::f64::consts::PI;

pub trait Tracking {
    fn track(&self, beam: &mut Beam);
}

pub struct Drift {
    t_matrix: Array2<f64>,
}

impl Drift {
    pub fn new(l: f64, g: f64) -> Drift {
        let beta_sq = gamma_2_beta(g).powi(2);
        let gamma_sq = g.powi(2);
        let r56 = l / (beta_sq * gamma_sq);
        Drift {
            t_matrix: arr2(&[[1f64, r56], [0f64, 1f64]]),
        }
    }
}

impl Tracking for Drift {
    fn track(&self, beam: &mut Beam) {
        *beam = beam.dot(&self.t_matrix.t());
    }
}

pub type Corr = Drift;
pub type Quad = Drift;
pub type Sext = Drift;

pub struct Dipole {
    t_matrix: Array2<f64>,
}

impl Dipole {
    pub fn new(b: f64, angle: f64, g: f64) -> Dipole {
        let pc = (g.powi(2) - 1.0).sqrt() * MASS;
        let rho = pc / (C * b.abs());
        assert!(
            rho > 0f64,
            "Radius of curvature of a dipole should not be negative or zero"
        );
        let omega = 1f64 / rho;
        let l = rho * angle.abs();
        assert!(
            l > 0f64,
            "Path length through a dipole should not be negative or zero"
        );
        let omega_l = omega * l;
        let beta_sq = gamma_2_beta(g).powi(2);
        let gamma_sq = g.powi(2);
        let r56 = l / (beta_sq * gamma_sq) - (omega_l - omega_l.sin()) / (omega * beta_sq);
        Dipole {
            t_matrix: arr2(&[[1f64, r56], [0f64, 1f64]]),
        }
    }
}

impl Tracking for Dipole {
    fn track(&self, beam: &mut Beam) {
        *beam = beam.dot(&self.t_matrix.t());
    }
}

pub struct AccCav {
    drift_matrix: Array2<f64>,
    kick_matrix: Array2<f64>,
}

impl AccCav {
    pub fn new(l: f64, v: f64, freq: f64, phi: f64, g: f64) -> AccCav {
        let beta_sq = gamma_2_beta(g).powi(2);
        let gamma_sq = g.powi(2);
        let r56_drift = l / (beta_sq * gamma_sq);

        let k = 2f64 * PI * freq / C;
        let r65_kick = -k * l * v * phi.sin() / (g * MASS);
        AccCav {
            drift_matrix: arr2(&[[1f64, r56_drift], [0f64, 1f64]]),
            kick_matrix: arr2(&[[1f64, 0f64], [r65_kick, 1f64]]),
        }
    }
}

impl Tracking for AccCav {
    fn track(&self, beam: &mut Beam) {
        *beam = beam.dot(&self.drift_matrix.t());
        *beam = beam.dot(&self.kick_matrix.t());
        *beam = beam.dot(&self.drift_matrix.t());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_ulps_eq; // for floating point tests

    #[test]
    fn drift_does_not_affect_energy_error() {
        let drift = Drift::new(2f64, 10f64);
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
        let gamma0 = 3000f64;
        let beta0 = gamma_2_beta(gamma0);
        let drift = Drift::new(drift_l, gamma0);
        for rel_e_err in [-0.01, -0.005, -0.001, 0.0, 0.001, 0.005, 0.01] {
            let gamma_delta = rel_e_err;
            let delta_z = drift_l * (gamma_delta / (gamma0.powi(2) * beta0.powi(3)));
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
        let quad = Quad::new(2f64, 10f64);
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
        let gamma0 = 3000f64;
        let beta0 = gamma_2_beta(gamma0);
        let quad = Quad::new(quad_l, gamma0);
        for rel_e_err in [-0.01, -0.005, -0.001, 0.0, 0.001, 0.005, 0.01] {
            let gamma_delta = rel_e_err;
            let delta_z = quad_l * (gamma_delta / (gamma0.powi(2) * beta0.powi(3)));
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
