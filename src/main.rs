use std::f64::consts::PI;
const MASS: f64 = 510998.9499961642f64;
const C: f64 = 299792458f64;

pub trait Tracking {
    fn track(&self, beam: Beam) -> Beam;
}

struct AccCav {
    length: f64,
    voltage: f64,
    freq: f64,
    phi: f64,
}

impl AccCav {
    fn new(l: f64, v: f64, freq: f64, phi: f64) -> AccCav {
        AccCav {
            length: l,
            voltage: v,
            freq: freq,
            phi: phi,
        }
    }
}

struct Drift {
    length: f64,
    gamma0: f64,
}

impl Drift {
    fn new(l: f64, g: f64) -> Drift {
        Drift {
            length: l,
            gamma0: g,
        }
    }
}

struct Dipole {
    b_field: f64,
    theta: f64,
    gamma0: f64,
}

impl Dipole {
    fn new(b: f64, angle: f64, g: f64) -> Dipole {
        Dipole {
            b_field: b,
            theta: angle,
            gamma0: g,
        }
    }
}

impl Tracking for Drift {
    fn track(&self, beam: Beam) -> Beam {
        let mut output_beam: Beam = vec![];
        for electron in beam {
            let t = electron.t;
            let l = self.length;

            let g0 = self.gamma0;
            let g = electron.ke / MASS;

            let beta = (1.0 - (1.0 / g.powi(2))).sqrt();
            let beta0 = (1.0 - (1.0 / g0.powi(2))).sqrt();

            let new_t = t + (l / C) * (1.0 / beta - 1.0 / beta0);

            output_beam.push(Electron {
                t: new_t,
                ke: electron.ke,
            });
        }
        output_beam
    }
}

impl Tracking for Dipole {
    fn track(&self, beam: Beam) -> Beam {
        let mut output_beam: Beam = vec![];
        for electron in beam {
            let g0 = self.gamma0;
            let g = electron.ke / MASS;

            let pc0 = (g0.powi(2) - 1.0).sqrt() * MASS;
            let pc = (g.powi(2) - 1.0).sqrt() * MASS;

            let rho0 = pc0 / (C * self.b_field);
            let rho = pc / (C * self.b_field);

            let l0 = rho0 * self.theta;
            let l = rho * self.theta;

            let delta_l = l - l0;
            let v = C * (1.0 - (1.0 / g.powi(2))).sqrt();

            let new_t = electron.t + delta_l / v;

            output_beam.push(Electron {
                t: new_t,
                ke: electron.ke,
            });
        }
        output_beam
    }
}

impl Tracking for AccCav {
    fn track(&self, beam: Beam) -> Beam {
        let mut output_beam: Beam = vec![];
        let egain = self.length * self.voltage;
        for electron in beam {
            let phase = self.phi + 2.0 * PI * (electron.t * self.freq);
            output_beam.push(Electron {
                t: electron.t,
                ke: electron.ke + egain * phase.cos(),
            });
        }
        output_beam
    }
}

struct Accelerator {
    pub elements: Vec<Box<dyn Tracking>>,
}

impl Accelerator {
    fn track(&self, beam: Beam) -> Beam {
        let mut output_beam = beam;
        for element in self.elements.iter() {
            output_beam = element.track(output_beam.clone());
        }
        output_beam
    }
}

// TODO: Electrons may be better described as a simple array. Look at ndarray.
#[derive(Copy, Clone)]
pub struct Electron {
    t: f64,
    ke: f64,
}

type Beam = Vec<Electron>;

fn main() {
    let design_ke = 1e8;
    let cav_length = 6.0;
    let cav_voltage = 20e6;
    let cav_freq = 3e9;
    let cav_phi: f64 = -1.56;
    let cav_egain = cav_voltage * cav_length * cav_phi.cos();
    let design_gamma = (design_ke + 4.0 * cav_egain) / MASS;

    let bunch_compressor = Accelerator {
        elements: vec![
            // Acceleration
            Box::new(AccCav::new(cav_length, cav_voltage, cav_freq, cav_phi)),
            Box::new(Drift::new(0.5, (design_ke + cav_egain) / MASS)),
            Box::new(AccCav::new(cav_length, cav_voltage, cav_freq, cav_phi)),
            Box::new(Drift::new(0.5, (design_ke + 2.0 * cav_egain) / MASS)),
            Box::new(AccCav::new(cav_length, cav_voltage, cav_freq, cav_phi)),
            Box::new(Drift::new(0.5, (design_ke + 3.0 * cav_egain) / MASS)),
            Box::new(AccCav::new(cav_length, cav_voltage, cav_freq, cav_phi)),
            Box::new(Drift::new(0.5, design_gamma)),
            // Bunch compressor chicane
            Box::new(Dipole::new(1.0, 2.0, design_gamma)),
            Box::new(Drift::new(1.0, design_gamma)),
            Box::new(Dipole::new(1.0, -2.0, design_gamma)),
            Box::new(Drift::new(1.0, design_gamma)),
            Box::new(Dipole::new(1.0, -2.0, design_gamma)),
            Box::new(Drift::new(1.0, design_gamma)),
            Box::new(Dipole::new(1.0, 2.0, design_gamma)),
            Box::new(Drift::new(1.0, design_gamma)),
        ],
    };

    let beam = vec![
        Electron {
            t: -10e-12,
            ke: design_ke,
        },
        Electron {
            t: 0.0,
            ke: design_ke,
        },
        Electron {
            t: 10e-12,
            ke: design_ke,
        },
    ];

    println!("---   INPUT  ---");
    for electron in &beam {
        println!(
            "{:0.6} ps :: {:0.3} MeV",
            electron.t * 1e12,
            electron.ke * 1e-6
        );
    }
    // let out_beam = drift.track(beam);
    // let out_beam = bend.track(beam);
    println!("--- TRACKING ---");
    let out_beam = bunch_compressor.track(beam);
    println!("---  OUTPUT  ---");

    for electron in out_beam {
        println!(
            "{:0.6} ps :: {:0.3} MeV",
            electron.t * 1e12,
            electron.ke * 1e-6
        );
    }
}
