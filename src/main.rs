const MASS: f64 = 510998.9499961642;

pub trait Tracking {
    fn track(&self, beam: Beam) -> Beam;
}

#[derive(Debug)]
struct Drift {
    length: f64,
    gamma0: f64,
}

impl Tracking for Drift {
    fn track(&self, beam: Beam) -> Beam {
        let mut output_beam: Beam = vec![];
        for electron in beam {
            let z = electron.z;
            let g0 = self.gamma0;
            let g = electron.ke / MASS;
            let g02_1 = g0.powi(2) - 1.0;
            let g2_1 = g.powi(2) - 1.0;
            let l = self.length;
            let new_z = z + l * ((g0 / g) * (g2_1 / g02_1).sqrt() - 1.0);

            output_beam.push(Electron {
                z: new_z,
                ke: electron.ke,
            });
        }
        output_beam
    }
}

#[derive(Debug)]
pub struct Electron {
    z: f64,
    ke: f64,
}

type Beam = Vec<Electron>;

fn main() {
    let design_ke = 1e8;

    let drift = Drift {
        length: 1.0,
        gamma0: design_ke / MASS,
    };

    let beam = vec![
        Electron {
            z: 0.0,
            ke: 0.99 * design_ke,
        },
        Electron {
            z: 0.0,
            ke: design_ke,
        },
        Electron {
            z: 0.001,
            ke: design_ke,
        },
        Electron {
            z: 0.0,
            ke: 1.01 * design_ke,
        },
    ];

    let out_beam = drift.track(beam);

    for electron in out_beam {
        println!("{:0.6} fs", electron.z / 3e8 * 1e15);
    }
}
