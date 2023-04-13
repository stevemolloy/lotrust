use crate::beam::{gamma_2_beta, Beam, C, MASS};
use ndarray::{arr2, Array2};
// use std::f64::consts::PI;

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
        let rho = pc / (C * b);
        let omega = 1f64 / rho;
        let l = rho * angle;
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
    t_matrix: Array2<f64>,
}

impl AccCav {
    pub fn new(l: f64, v: f64, freq: f64, phi: f64) -> AccCav {
        let _blah = l + v + freq + phi;
        AccCav {
            t_matrix: arr2(&[[1f64, 0f64], [0f64, 1f64]]),
        }
    }
}

impl Tracking for AccCav {
    fn track(&self, beam: &mut Beam) {
        *beam = beam.dot(&self.t_matrix.t());
    }
}
