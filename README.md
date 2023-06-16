# LOTRust

A LOngitudinal Tracker in Rust.  Yeah, crappy name, but I like the LOTR thing.

# Important!
Please note that the physics of this code has not really been tested at all.  This code is not fit for use yet.

# REPL
Designing a REPL for this tracker, and thinking about how I would like it to work:

## Example 1

```bash
> load_lattice "filename.ele"
> load_beam "beamfile.lotr"
> track
Tracking 21 particles through 773 accelerator elements...
Tracking
Done!
> save output_file
```

## Example 2

```bash
> load_lattice "filename.ele"
> load_beam "beamfile.lotr"
> delta_phase "I.L15b_linspf" 0.01
> track
> save output_beam
> delta_phase "I.L15b_linspf" -0.01
> delta_phase "I.L15b_linspf" -0.01
> track
> save output_beam
```

# Commands I would like

- load_lattice
    - Load an accelerator lattice from an *.lotr file or an elegant (*.lte) file
- load_beam
    - Load a beam definition from an *.lotr file
- parameter_delta <element_name> <param_name> <delta_value>
    - Change the value of a particular parameter for a particular element by a delta
- track
    - Track the beam through the accelerator
- save output_beam
    - Save the output_beam data in an *.lotr file
- save accelerator
    - Save the accelerator lattice as an *.lotr file
- find_ele_type <element_type>
    - Returns the names of all elements of the given type
- find_ele_name <element_name>
    - Returns the details of 

# Quick start
Write a file like the following (`myfile.lotr`)
```
beam { // Beam definitions
    design_ke: 2.5e8 // KE used to scale parameters. Must come first.
    particles { // Define each particle individually
        // z (m) energy_error (eV)
        -3e-3 0e6
        0 0e6
        3e-3 0e6
    }
}

accelerator {
    initial_ke: 2.5e8 // KE used to scale parameters. Must come first.
    drift: 1.0
    acccav: 6.0 20e6 3e9 -0.085
    drift: 2.0
    dipole: 2.0 -1.0
    drift: 10.0
    dipole: 2.0 -1.0
    drift: 10.0
    dipole: 2.0 -1.0
    drift: 10.0
    dipole: 2.0 1.0
    drift: 2.0
}
```

The run the code:
```bash
cargo run myfile.lotr
```

This is a basic design for a bunch compressor.  The input particles have no energy error but are located at different `z` positions.  After off-crest acceleration and tracking through a dipole chicane the particles each have the same (roughly) longitudinal position, with non-zero energy spreads.

Optionally, you can provide an additional file for output. If provided, the `z` position and energy deviation of all particles between all components are written to this file as a three-dimensional numpy array. For example

```bash
cargo run myfile.lotr out.npy
```
