# LOTRust

A LOngitudinal Tracker in Rust.  Yeah, crappy name, but I like the LOTR thing.

# Quick start
Write a file like the following (`myfile.lotr`)
```
beam { // Beam definitions
    particles { // Define each particle individually
        // t (seconds) KE (eV)
        -10e-12 24.75e6
        -10e-12 25.00e6
        -10e-12 25.25e6
        0 24.75e6
        0 25.00e6
        0 25.25e6
        10e-12 24.75e6
        10e-12 25.00e6
        10e-12 25.25e6
    }
}

accelerator {
    initial_ke: 25e6 // KE used to scale parameters. Must come first.
    drift: 1.0
    acccav: 6.0 20e6 3e9 -0.25
    // First bend
    dipole: 2.0 1.0
    drift: 1.0
    dipole: 2.0 -1.0
    drift: 1.0
    dipole: 2.0 -1.0
    drift: 1.0
    dipole: 2.0 1.0
}
```

The run the code:
```bash
cargo run myfile.lotr
```

