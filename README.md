# Satellite Numeric

## What is this project?

This project is an [anytime algorithm](https://en.wikipedia.org/wiki/Anytime_algorithm) modeled around the Satellite Numeric domain in the [International Planning Competition 2002](http://ipc02.icaps-conference.org/). It is written in Rust, and makes use of the [AnyHop](https://github.com/gjf2a/anyhop) library written by Dr. Ferrer.

This project is made to demonstrate the capabilities of anytime algorithms in order to continually improve on solutions for NP-hard problems through the ability to be stopped at any time.

## How do I run this project? 

In order to run the project, you must have [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed. Once you have cargo installed, you must set up the Rust nightly build. This is needed for some features of the language.

Next, clone the [executable](https://github.com/ivyjsgit/satellite_numeric_expr) of the project. 

Finally, run 

> cargo run -- -5s ../Research_Project/SatelliteNumeric/Numeric/pfile3

This command will run the program for 5 seconds on the specified pddl file.
