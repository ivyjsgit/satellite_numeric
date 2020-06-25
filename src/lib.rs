#![feature(option_result_contains)]
#![allow(dead_code)]
#![allow(unused_imports)]
#[macro_use]

extern crate io_error;
extern crate strum;
extern crate strum_macros;



pub mod methods;
pub mod operators;
pub mod pddl_parser;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
