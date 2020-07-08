use std::{fs, io};
use std::collections::{BTreeMap, HashMap};
use std::process::exit;
use std::rc::{self, Rc};

use pddl_problem_parser::{Predicate, PddlProblem};
use crate::operators::{SatelliteEnum, SatelliteGoals, SatelliteState};
use crate::operators::SatelliteEnum::{Direction, Instrument, Mode, Satellite};
use fixed::types::I40F24;

pub fn make_satellite_problem_from(pddl_file: &str) -> io::Result<(SatelliteState, SatelliteGoals)> {
    let contents = fs::read_to_string(pddl_file)?;
    let parsed = pddl_problem_parser::PddlParser::parse(contents.as_str())?;

    let objects = enumerate_objects(&parsed);

    println!("objects {:?}", objects);

    let satellite_state = extract_state(&parsed,&objects);

    let goals = extract_goals(&parsed, &objects);



        return Ok((satellite_state, goals));
}

fn enumerate_objects(parsed: &PddlProblem) -> BTreeMap<String,I40F24> {
    let mut objects: BTreeMap<String, I40F24> = BTreeMap::new();
    for object in parsed.obj2type.keys() {
        objects.insert(String::from(object), I40F24::from_num(objects.len()));
    }
    return objects
}
fn extract_state(parsed: &PddlProblem, objects: &BTreeMap<String,I40F24>) -> SatelliteState {

    //These are everything that don't start with an equal
    let mut onboard: BTreeMap<SatelliteEnum, Vec<SatelliteEnum>>= BTreeMap::new();
    let mut supports: BTreeMap<SatelliteEnum, Vec<SatelliteEnum>> = BTreeMap::new();
    let mut pointing: BTreeMap<SatelliteEnum, SatelliteEnum> = BTreeMap::new();
    let mut power_avail = false;
    let mut power_on: Vec<SatelliteEnum> = vec![];
    let mut calibrated: Vec<SatelliteEnum> = vec![];
    let mut have_image: BTreeMap<SatelliteEnum, SatelliteEnum> = BTreeMap::new();
    let mut calibration_target: BTreeMap<SatelliteEnum, SatelliteEnum> = BTreeMap::new();

    //These things begin with an equal.
    let mut data_capacity : BTreeMap<SatelliteEnum, I40F24> = BTreeMap::new();
    let mut satellite_data_stored: BTreeMap<(SatelliteEnum, SatelliteEnum), I40F24> = BTreeMap::new();
    let mut satellite_fuel_capacity: BTreeMap<SatelliteEnum, I40F24> = BTreeMap::new();
    let mut slew_time: BTreeMap<(SatelliteEnum, SatelliteEnum), I40F24> = BTreeMap::new();
    let mut fuel_used = 0;
    let mut fuel: BTreeMap<SatelliteEnum, I40F24>=  BTreeMap::new();

    let mut total_data_stored = 0;

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

    //Parse things with an equals in them
    for (pred, value) in parsed.i40f24_state.iter(){
        if pred.get_tag() == "data_capacity"{
            let satellite = Satellite(obj_get(pred.get_arg(0), objects));
            data_capacity.insert(satellite, value.to_num::<I40F24>());
        }else if pred.get_tag() == "fuel"{
            let satellite = Satellite(obj_get(pred.get_arg(0), objects));
            fuel.insert(satellite,value.to_num::<I40F24>());
        }else if pred.get_tag() == "slew_time" {
            let position_a = Direction(obj_get(pred.get_arg(0), objects));
            let position_b = Direction(obj_get(pred.get_arg(1), objects));
            slew_time.insert((position_a, position_b), *value);
        }else if pred.get_tag() == "data"{
            let position = Direction(obj_get(pred.get_arg(0), objects));
            let mode = Mode(obj_get(pred.get_arg(0), objects));
            satellite_data_stored.insert((position,mode), *value);
        }else if pred.get_tag() == "fuel_used"{
            fuel_used = value.to_num();
        }
    }
    for value in satellite_data_stored.values().into_iter(){
        let value_as_u32 = value.to_num::<u32>();
        total_data_stored+=value_as_u32;
    }

    return SatelliteState::new(onboard,supports,pointing,power_avail,power_on,calibrated,have_image,calibration_target, data_capacity, I40F24::from_num(total_data_stored),satellite_data_stored,slew_time,I40F24::from_num(fuel_used), fuel);
}

fn decode_onboard(p: &Predicate, objects: &BTreeMap<String,I40F24>) -> (I40F24, I40F24) {
    let instrument = obj_get(p.get_arg(0), objects);
    let satellite = obj_get(p.get_arg(1), objects);

    return (satellite, instrument);
}

fn decode_supports(p: &Predicate, objects: &BTreeMap<String,I40F24>) -> (I40F24, I40F24) {
    //instrument modes
    let instrument = obj_get(p.get_arg(0), objects);
    let mode = obj_get(p.get_arg(1), objects);

    return (instrument, mode);

}

fn decode_pointing(p: &Predicate, objects: &BTreeMap<String,I40F24>) -> (I40F24, I40F24) {
    let satellite = obj_get(p.get_arg(0), objects);
    let direction = obj_get(p.get_arg(1), objects);
    (satellite, direction)
}

fn decode_have_image(p: &Predicate, objects: &BTreeMap<String,I40F24>) -> (I40F24, I40F24) {
    let direction = obj_get(p.get_arg(0), objects);
    let mode = obj_get(p.get_arg(1), objects);
    (direction, mode)
}

fn decode_calibration_target(p: &Predicate, objects: &BTreeMap<String,I40F24>) -> (I40F24, I40F24) {
    let instrument = obj_get(p.get_arg(0), objects);
    let direction = obj_get(p.get_arg(1), objects);
    (instrument, direction)
}

fn obj_get(obj_name: &str, objects: &BTreeMap<String,I40F24>) -> I40F24 {
    *(objects.get(obj_name).unwrap())
}


fn extract_goals(parsed: &PddlProblem, objects: &BTreeMap<String,I40F24>) -> SatelliteGoals {
    let mut have_image: BTreeMap<SatelliteEnum, SatelliteEnum> = BTreeMap::new();
    let mut pointing: BTreeMap<SatelliteEnum, SatelliteEnum> = BTreeMap::new();

    let fuel_used = I40F24::from_num(0);
    for goal in parsed.goals.iter() {
        if goal.get_tag() == "have_image" {
            let decoded_have_image = decode_calibration_target(&goal, &objects);
            have_image.insert(Direction(decoded_have_image.0), Mode(decoded_have_image.1));
        }else if goal.get_tag() == "pointing"{
            let satellite = obj_get(goal.get_arg(0), objects);
            let direction = obj_get(goal.get_arg(1), objects);

            pointing.insert(Satellite(satellite), Direction(direction));
        }
    }
    return SatelliteGoals::new(have_image, pointing,fuel_used);
}
