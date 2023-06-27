use crate::beam::{gamma_2_beta, C, MASS};
use core::fmt::Debug;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::process::exit;

// const E_CHARGE: f64 = 1.60217663e-19;

#[derive(Debug)]
pub enum EleType {
    Drift,
    Dipole,
    AccCav,
}

// TODO(#2): Beam should (?) be resorted when tracked by an element that may reorder things.
// Which elements could reorder particles? Dipoles.  AccCavs, but not in the linear approx.
// TODO(#3): Add various diag elements that act on the beam as drifts, but produce side-effects.
#[derive(Debug)]
pub struct Element {
    pub ele_type: EleType,
    pub name: String,
    pub gamma: f64,
    pub length: f64,
    #[allow(dead_code)]
    pub params: HashMap<String, f64>,
}

pub fn make_drift(name: String, length: f64, gamma: f64) -> Element {
    let beta_sq = gamma_2_beta(gamma).powi(2);
    let gamma_sq = gamma.powi(2);
    let r56 = length / (beta_sq * gamma_sq);
    let mut param_map = HashMap::new();
    param_map.insert("r56".to_string(), r56);
    Element {
        name,
        ele_type: EleType::Drift,
        length,
        gamma,
        params: param_map,
    }
}

pub fn make_quad(name: String, length: f64, gamma: f64) -> Element {
    make_drift(name, length, gamma)
}

pub fn make_dipole(name: String, length: f64, angle: f64, gamma: f64) -> Element {
    if length == 0f64 {
        eprintln!("Path length through a dipole should not be negative or zero");
        exit(1);
    }
    let angle_fixed = if angle == 0f64 {
        f64::MIN_POSITIVE
    } else {
        angle
    };
    let omega = angle_fixed / length;
    let beta_sq = gamma_2_beta(gamma).powi(2);
    let gamma_sq = gamma.powi(2);
    let r56 = length / (beta_sq * gamma_sq) - (angle_fixed - angle_fixed.sin()) / (omega * beta_sq);

    let mut param_map = HashMap::new();
    param_map.insert("angle".to_string(), angle);
    param_map.insert("r56".to_string(), r56);
    Element {
        name,
        ele_type: EleType::Dipole,
        length,
        gamma,
        params: param_map,
    }
}

// TODO(#4): Accelerating cavities need to have wakefields in their physics.
pub fn make_acccav(name: String, length: f64, v: f64, freq: f64, phi: f64, gamma: f64) -> Element {
    let beta_sq = gamma_2_beta(gamma).powi(2);
    let gamma_sq = gamma.powi(2);
    let r56_drift = length / (beta_sq * gamma_sq);

    let k = 2f64 * PI * freq / C;
    let r65_kick = -k * v * phi.sin() / ((gamma_sq - 1f64).powf(0.5) * MASS);

    let mut param_map = HashMap::new();
    param_map.insert("v".to_string(), v);
    param_map.insert("freq".to_string(), freq);
    param_map.insert("phi".to_string(), phi);
    param_map.insert("r56_drift".to_string(), r56_drift);
    param_map.insert("r65_kick".to_string(), r65_kick);
    Element {
        name,
        ele_type: EleType::AccCav,
        length,
        gamma,
        params: param_map,
    }
}

#[cfg(test)]
mod tests {
    use crate::beam::Beam;

    use super::*;
    use approx::assert_ulps_eq;
    use ndarray::Array2; // for floating point tests
    const GAMMA0: f64 = 3000f64;

    #[test]
    fn dipole_does_not_affect_energy_error() {
        let b_field = 2.0;
        let angle = 0.7;
        let dipole = make_dipole("dipole".to_string(), b_field, angle, GAMMA0);
        for e_error in [-0.01, -0.005, -0.001, 0.0, 0.001, 0.005, 0.01] {
            for z in [-5e-3, -1e-3, 0.0, 1e-3, 5e-3] {
                let beam_vec = Array2::from(vec![[z, (1f64 / gamma_2_beta(GAMMA0)) * e_error]]);
                let mut beam = Beam::new(beam_vec.clone());
                beam.track(&dipole);
                assert_eq!(beam_vec[[0, 1]], (1f64 / gamma_2_beta(GAMMA0)) * e_error);
            }
        }
    }

    #[test]
    fn drift_does_not_affect_energy_error() {
        let drift = make_drift("drift".to_string(), 2f64, 10f64);
        for e_error in [-0.01, -0.005, -0.001, 0.0, 0.001, 0.005, 0.01] {
            for z in [-5e-3, -1e-3, 0.0, 1e-3, 5e-3] {
                let beam_vec = Array2::from(vec![[z, e_error]]);
                let mut beam = Beam::new(beam_vec.clone());
                beam.track(&drift);
                assert_eq!(beam_vec[[0, 1]], e_error);
            }
        }
    }

    #[test]
    fn drift_alters_z_correctly() {
        let drift_l = 1f64;
        let beta0 = gamma_2_beta(GAMMA0);
        let drift = make_drift("drift".to_string(), drift_l, GAMMA0);
        for rel_e_err in [-0.01, -0.005, -0.001, 0.0, 0.001, 0.005, 0.01] {
            let gamma_delta = rel_e_err;
            let delta_z = drift_l * (gamma_delta / (GAMMA0.powi(2) * beta0.powi(3)));
            for z in [-5e-3, -1e-3, 0.0, 1e-3, 5e-3] {
                let beam_vec = Array2::from(vec![[z, (1f64 / beta0) * rel_e_err]]);
                let mut beam = Beam::new(beam_vec.clone());
                beam.track(&drift);
                assert_ulps_eq!(beam_vec[[0, 0]], z + delta_z, epsilon = 1e-8, max_ulps = 4);
            }
        }
    }

    #[test]
    fn quad_does_not_affect_energy_error() {
        let quad = make_quad("quad".to_string(), 2f64, 10f64);
        for e_error in [-0.01, -0.005, -0.001, 0.0, 0.001, 0.005, 0.01] {
            for z in [-5e-3, -1e-3, 0.0, 1e-3, 5e-3] {
                let beam_vec = Array2::from(vec![[z, e_error]]);
                let mut beam = Beam::new(beam_vec.clone());
                beam.track(&quad);
                assert_eq!(beam_vec[[0, 1]], e_error);
            }
        }
    }

    #[test]
    fn quad_alters_z_correctly() {
        let quad_l = 1f64;
        let beta0 = gamma_2_beta(GAMMA0);
        let quad = make_quad("quad".to_string(), quad_l, GAMMA0);
        for rel_e_err in [-0.01, -0.005, -0.001, 0.0, 0.001, 0.005, 0.01] {
            let gamma_delta = rel_e_err;
            let delta_z = quad_l * (gamma_delta / (GAMMA0.powi(2) * beta0.powi(3)));
            for z in [-5e-3, -1e-3, 0.0, 1e-3, 5e-3] {
                let beam_vec = Array2::from(vec![[z, (1f64 / beta0) * rel_e_err]]);
                let mut beam = Beam::new(beam_vec.clone());
                beam.track(&quad);
                assert_ulps_eq!(beam_vec[[0, 0]], z + delta_z, epsilon = 1e-8, max_ulps = 1);
            }
        }
    }
}
