use crate::beam::{gamma_2_beta, Beam};
use ndarray::{arr2, Array2};
// use std::f64::consts::PI;

pub trait Tracking {
    fn track(&self, beam: &mut Beam);
}

pub struct Drift {
    length: f64,
    gamma0: f64,
    t_matrix: Array2<f64>,
}

impl Drift {
    pub fn new(l: f64, g: f64) -> Drift {
        let beta_sq = gamma_2_beta(g).powi(2);
        let gamma_sq = g.powi(2);
        let r56 = l / (beta_sq * gamma_sq);
        Drift {
            length: l,
            gamma0: g,
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
    b_field: f64,
    theta: f64,
    gamma0: f64,
}

impl Dipole {
    pub fn new(b: f64, angle: f64, g: f64) -> Dipole {
        Dipole {
            b_field: b,
            theta: angle,
            gamma0: g,
        }
    }
}

impl Tracking for Dipole {
    // The commented lines in this function calculate the change in the angle
    // due to the different radius of curvature, but in practise the difference
    // this makes in the timing is *tiny*
    fn track(&self, beam: &mut Beam) {
        todo!();
    }
}

pub struct AccCav {
    length: f64,
    voltage: f64,
    freq: f64,
    phi: f64,
}

impl AccCav {
    pub fn new(l: f64, v: f64, freq: f64, phi: f64) -> AccCav {
        AccCav {
            length: l,
            voltage: v,
            freq,
            phi,
        }
    }
}

impl Tracking for AccCav {
    fn track(&self, beam: &mut Beam) {
        todo!();
    }
}
