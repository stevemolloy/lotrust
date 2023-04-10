pub const MASS: f64 = 510998.9499961642f64;
pub const C: f64 = 299792458f64;

// TODO: Electrons may be better described as a simple array. Look at ndarray.
pub struct Electron {
    pub t: f64,
    pub ke: f64,
}

impl Electron {
    pub fn gamma(&self) -> f64 {
        ke_2_gamma(self.ke)
    }
}

pub type Beam = Vec<Electron>;

pub fn ke_2_gamma(ke: f64) -> f64 {
    ke / MASS + 1f64
}

pub fn gamma_2_beta(g: f64) -> f64 {
    (1f64 - (1f64 / g.powi(2))).sqrt()
}

#[test]
fn ke_of_restmass_has_gamma_two() {
    let electron = Electron { t: 0f64, ke: MASS };
    assert_eq!(electron.gamma(), 2.0);
}

#[test]
fn zero_ke_has_unity_gamma() {
    let electron = Electron { t: 0f64, ke: 0f64 };
    assert_eq!(electron.gamma(), 1.0);
}

#[test]
fn zero_ke_has_zero_beta() {
    let electron = Electron { t: 0f64, ke: 0f64 };
    assert_eq!(gamma_2_beta(electron.gamma()), 0.0);
}
