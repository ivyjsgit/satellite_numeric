use std::{fs, io};
use std::collections::{BTreeMap, HashMap};
use std::process::exit;
use std::rc::{self, Rc};

use pddl_problem_parser::{Predicate, PddlProblem};
use crate::operators::{SatelliteEnum, SatelliteGoals, SatelliteState};
use crate::operators::SatelliteEnum::{Direction, Instrument, Mode, Satellite};

//This is because the parsing library actually uses the fields.

/*
SATELLITE STUFF

 */


pub fn make_satellite_problem_from(pddl_file: &str) -> io::Result<(SatelliteState, SatelliteGoals)> {
    let contents = fs::read_to_string(pddl_file)?;
    let parsed = pddl_problem_parser::PddlParser::parse(contents.as_str())?;

    let objects = enumerate_objects(&parsed);

    println!("objects {:?}", objects);

    let satellite_state = extract_state(&parsed,&objects);

    let goals = extract_goals(&parsed, &objects);



        return Ok((satellite_state, goals));
}

fn enumerate_objects(parsed: &PddlProblem) -> BTreeMap<String,u32> {
    let mut objects: BTreeMap<String, u32> = BTreeMap::new();
    for object in parsed.obj2type.keys() {
        objects.insert(String::from(object), objects.len() as u32);
    }
    return objects
}

//onboard,supports,pointing,power_avail,power_on,calibrated,have_image,calibration_target
fn extract_state(parsed: &PddlProblem, objects: &BTreeMap<String,u32>) -> SatelliteState {

    let mut onboard: BTreeMap<SatelliteEnum, Vec<SatelliteEnum>>= BTreeMap::new();
    let mut supports: BTreeMap<SatelliteEnum, Vec<SatelliteEnum>> = BTreeMap::new();
    let mut pointing: BTreeMap<SatelliteEnum, SatelliteEnum> = BTreeMap::new();
    let mut power_avail = false;
    let mut power_on: Vec<SatelliteEnum> = vec![];
    let mut calibrated: Vec<SatelliteEnum> = vec![];
    let mut have_image: BTreeMap<SatelliteEnum, SatelliteEnum> = BTreeMap::new();
    let mut calibration_target: BTreeMap<SatelliteEnum, SatelliteEnum> = BTreeMap::new();

    for pred in parsed.bool_state.iter() {
        if pred.get_tag() == "on_board" {
            //map satellite -> vec<instrument>
            let onboard_inserter = decode_onboard(&pred, &objects);
            match onboard.get_mut(&Satellite(onboard_inserter.0)){
                None => {onboard.insert(Satellite(onboard_inserter.0),vec![Instrument(onboard_inserter.1)]);},
                Some(n) => n.push(Instrument(onboard_inserter.1)),
            };
        } else if pred.get_tag() == "supports" {
            let support_inserter = decode_supports(&pred, &objects);
            match supports.get_mut(&Instrument(support_inserter.0)){
                None => {supports.insert(Instrument(support_inserter.0), vec![Mode(support_inserter.1)]);},
                Some(n) => n.push(Mode(support_inserter.1)),
            };
        }else if pred.get_tag() == "pointing" {
            let decoded_pointing = decode_pointing(&pred, &objects);
            pointing.insert(Satellite(decoded_pointing.0), Direction(decoded_pointing.1));
        }else if pred.get_tag() == "power_avail" {
            power_avail = true;
        }else if pred.get_tag() == "power_on" {
            power_on.push(Instrument(*objects.get(pred.get_arg(0)).unwrap()));
        } else if pred.get_tag() == "calibrated" {
            calibrated.push(Instrument(*objects.get(pred.get_arg(0)).unwrap()));
        }else if pred.get_tag() == "have_image" {
            let decoded_have_image = decode_have_image(&pred, &objects);
            have_image.insert(Direction(decoded_have_image.0), Mode(decoded_have_image.1));
        }else if pred.get_tag() == "calibration_target" {
            let decoded_calibration_target = decode_calibration_target(&pred, &objects);
            calibration_target.insert(Instrument(decoded_calibration_target.0), Direction(decoded_calibration_target.1));
        }
    }
    return SatelliteState::new(onboard,supports,pointing,power_avail,power_on,calibrated,have_image,calibration_target);
}

fn decode_onboard(p: &Predicate, objects: &BTreeMap<String,u32>) -> (u32, u32) {
    let instrument = obj_get(p.get_arg(0), objects);
    let satellite = obj_get(p.get_arg(1), objects);

    return (satellite, instrument);
}

fn decode_supports(p: &Predicate, objects: &BTreeMap<String,u32>) -> (u32, u32) {
    //instrument modes
    let instrument = obj_get(p.get_arg(0), objects);
    let mode = obj_get(p.get_arg(1), objects);

    return (instrument, mode);

}

fn decode_pointing(p: &Predicate, objects: &BTreeMap<String,u32>) -> (u32, u32) {
    let satellite = obj_get(p.get_arg(0), objects);
    let direction = obj_get(p.get_arg(1), objects);
    (satellite, direction)
}

fn decode_have_image(p: &Predicate, objects: &BTreeMap<String,u32>) -> (u32, u32) {
    let direction = obj_get(p.get_arg(0), objects);
    let mode = obj_get(p.get_arg(1), objects);
    (direction, mode)
}

fn decode_calibration_target(p: &Predicate, objects: &BTreeMap<String,u32>) -> (u32, u32) {
    let instrument = obj_get(p.get_arg(0), objects);
    let direction = obj_get(p.get_arg(1), objects);
    (instrument, direction)
}

fn obj_get(obj_name: &str, objects: &BTreeMap<String,u32>) -> u32 {
    *(objects.get(obj_name).unwrap())
}


fn extract_goals(parsed: &PddlProblem, objects: &BTreeMap<String,u32>) -> SatelliteGoals {
    let mut have_image: BTreeMap<SatelliteEnum, SatelliteEnum> = BTreeMap::new();
    let fuel_used = 0;
    for goal in parsed.goals.iter() {
        if goal.get_tag() == "have_image" {
            let decoded_have_image = decode_calibration_target(&goal, &objects);
            have_image.insert(Direction(decoded_have_image.0), Mode(decoded_have_image.1));
        }
    }
    return SatelliteGoals::new(have_image,fuel_used);
}
