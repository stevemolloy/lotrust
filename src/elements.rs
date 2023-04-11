use crate::beam::{gamma_2_beta, Beam, C, MASS};
use std::f64::consts::PI;

pub trait Tracking {
    fn track(&self, beam: &mut Beam);
}

pub struct Drift {
    length: f64,
    gamma0: f64,
}

impl Drift {
    pub fn new(l: f64, g: f64) -> Drift {
        Drift {
            length: l,
            gamma0: g,
        }
    }
}

impl Tracking for Drift {
    fn track(&self, beam: &mut Beam) {
        for electron in beam.iter_mut() {
            let t = electron.t;
            let l = self.length;

            let g0 = self.gamma0;
            let g = electron.gamma();

            let beta = gamma_2_beta(g);
            let beta0 = gamma_2_beta(g0);

            let new_t = t + (l / C) * (1.0 / beta - 1.0 / beta0);

            electron.t = new_t;
        }
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
        for electron in beam.iter_mut() {
            let g0 = self.gamma0;
            let g = electron.gamma();

            let pc0 = (g0.powi(2) - 1.0).sqrt() * MASS;
            let pc = (g.powi(2) - 1.0).sqrt() * MASS;

            let rho0 = pc0 / (C * self.b_field);
            let rho = pc / (C * self.b_field);
            // let delta_rho = rho - rho0;
            // let new_theta = self.theta - (delta_rho/rho)*self.theta.sin();

            let l0 = rho0 * self.theta;
            let l = rho * self.theta;
            // let l = rho * new_theta;

            let delta_l = l - l0;
            let beta = gamma_2_beta(g);

            let new_t = electron.t + delta_l / (beta * C);

            electron.t = new_t;
        }
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
        let egain = self.length * self.voltage;
        for electron in beam.iter_mut() {
            let phase = self.phi + 2.0 * PI * (electron.t * self.freq);
            electron.ke += egain * phase.cos();
        }
    }
}
