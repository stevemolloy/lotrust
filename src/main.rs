use core::fmt::Debug;

const MASS: f64 = 510998.9499961642;
const C: f64 = 299792458.0;

pub trait Tracking {
    fn track(&self, beam: Beam) -> Beam;
}

#[derive(Debug)]
struct Drift {
    length: f64,
    gamma0: f64,
}

struct Dipole {
    b_field: f64,
    theta: f64,
    gamma0: f64,
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

struct Accelerator {
    pub elements: Vec<Box<dyn Tracking>>,
}

impl Debug for dyn Tracking {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
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

#[derive(Debug, Copy, Clone)]
pub struct Electron {
    t: f64,
    ke: f64,
}

type Beam = Vec<Electron>;

fn main() {
    let design_ke = 1e8;

    let bunch_compressor = Accelerator {
        elements: vec![
            Box::new(Drift {
                length: 1.0,
                gamma0: design_ke / MASS,
            }),
            Box::new(Dipole {
                b_field: 1.0,
                theta: 1.0,
                gamma0: design_ke / MASS,
            }),
            Box::new(Drift {
                length: 1.0,
                gamma0: design_ke / MASS,
            }),
            Box::new(Dipole {
                b_field: 1.0,
                theta: -1.0,
                gamma0: design_ke / MASS,
            }),
            Box::new(Drift {
                length: 1.0,
                gamma0: design_ke / MASS,
            }),
            Box::new(Dipole {
                b_field: 1.0,
                theta: -1.0,
                gamma0: design_ke / MASS,
            }),
            Box::new(Drift {
                length: 1.0,
                gamma0: design_ke / MASS,
            }),
            Box::new(Dipole {
                b_field: 1.0,
                theta: 1.0,
                gamma0: design_ke / MASS,
            }),
            Box::new(Drift {
                length: 1.0,
                gamma0: design_ke / MASS,
            }),
        ],
    };

    let beam = vec![
        Electron {
            t: -10e-15,
            ke: 0.99*design_ke,
        },
        Electron {
            t: 0.0,
            ke: design_ke,
        },
        Electron {
            t: 10e-15,
            ke: 1.01*design_ke,
        },
    ];

    println!("---   INPUT  ---");
    for electron in &beam {
        println!("{:0.6} fs", electron.t * 1e15);
    }
    // let out_beam = drift.track(beam);
    // let out_beam = bend.track(beam);
    println!("--- TRACKING ---");
    let out_beam = bunch_compressor.track(beam);
    println!("---  OUTPUT  ---");

    for electron in out_beam {
        println!("{:0.6} fs", electron.t * 1e15);
    }
}
