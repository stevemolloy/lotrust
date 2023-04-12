use ndarray::Array2;
pub const MASS: f64 = 510998.9499961642f64;
pub const C: f64 = 299792458f64;

pub type Beam = Array2<f64>;

pub fn ke_2_gamma(ke: f64) -> f64 {
    ke / MASS + 1f64
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
