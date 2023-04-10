pub const MASS: f64 = 510998.9499961642f64;
pub const C: f64 = 299792458f64;

// TODO: Electrons may be better described as a simple array. Look at ndarray.
pub struct Electron {
    pub t: f64,
    pub ke: f64,
}

pub type Beam = Vec<Electron>;
