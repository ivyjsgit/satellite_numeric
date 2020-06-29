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
    let satellite_state = extract_state(&parsed,&objects);

    // let mut onboard: BTreeMap<SatelliteEnum, Vec<SatelliteEnum>>= BTreeMap::new();
    //     let mut supports: BTreeMap<SatelliteEnum, Vec<SatelliteEnum>> =BTreeMap::new();
    //     let mut pointing: BTreeMap<SatelliteEnum, SatelliteEnum> =BTreeMap::new();
    //     let mut power_avail = false;
    //     let mut power_on: Vec<SatelliteEnum> = vec![];
    //     let mut calibrated: Vec<SatelliteEnum> = vec![];
    //     let mut have_image: BTreeMap<SatelliteEnum, SatelliteEnum> = BTreeMap::new();
    //     let mut calibration_target: BTreeMap<SatelliteEnum, SatelliteEnum> =  BTreeMap::new();
    //
    //
    //     //Why do these never change???
    //     let mut goals_images: BTreeMap<SatelliteEnum, SatelliteEnum> = BTreeMap::new();
    //     let mut goal_fuel_used: u32 = 0;
    //
    //
    //     let mut u_32_holder = SatelliteToU32::new(BTreeMap::new(), BTreeMap::new(), BTreeMap::new(), BTreeMap::new(), BTreeMap::new(), BTreeMap::new(), BTreeMap::new());
    //
    //     for pred in parsed.bool_state.iter(){
    //         // //Satellite -> Vec<Instrument>
    //         // if pred.get_tag() == "on_board" {
    //         //     map_onboard_to_numbers(&mut objects, &mut onboard, &mut u_32_holder, pred);
    //         // }else if pred.get_tag() == "supports" {
    //         //     map_supports_to_numbers(&mut objects, &mut supports, &mut u_32_holder, pred);
    //         // }else if pred.get_tag()== "pointing" {
    //         //     //Because this is a Vec and not a BTreeMap, we have to handle it differently.
    //         //     pointing.insert(Satellite(u_32_holder.decode(pred, &objects, "pointing".parse().unwrap())), Direction(pred.get_tag().len() as u32));
    //         // }else if pred.get_tag() == "power_avail" {
    //         //     power_avail = true;
    //         // } else if pred.get_tag() == "power_on" {
    //         //     power_on.push(Instrument(u_32_holder.decode(pred, &objects, "power_on".parse().unwrap())));
    //         // }else if pred.get_tag() == "calibrated" {
    //         //     calibrated.push(Instrument(u_32_holder.decode(pred, &objects, "calibrated".parse().unwrap())));
    //         // }else if pred.get_tag() == "have_image" {
    //         //     map_have_image_to_numbers(&mut objects, &mut have_image,&mut u_32_holder, pred);
    //         // }else if pred.get_tag() == "calibration_target" {
    //         //     map_calibration_target_to_numbers(&mut objects, &mut calibration_target, &mut u_32_holder, &pred);
    //         // }
    //     }
    //
    //     for goal in parsed.goals.iter(){
    //         // goals_images.insert(Direction(u_32_holder.decode(goal, &objects, "have_image".parse().unwrap())), Mode(u_32_holder.decode(goal, &objects, "have_image".parse().unwrap())));
    //     }

    let goal_images = BTreeMap::new();
    let goal_fuel_used = 0;

        Ok((satellite_state, SatelliteGoals::new(goal_images,goal_fuel_used)))
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
            let onboard_inserter = decode_onboard(&pred, &objects);
            let turn_into_instrument = |n| Instrument(n); //This needs to be mapped onto onboard_inserted.1
            onboard.insert(Satellite(onboard_inserter.0), (onboard_inserter.1.into_iter().map(turn_into_instrument).collect()));
        } else if pred.get_tag() == "supports" {
            let supports_inserter = decode_supports(&pred, &objects);
            let turn_into_mode = |n| Mode(n);
            supports.insert(Instrument(supports_inserter.0), (supports_inserter.1.into_iter().map(turn_into_mode).collect()));
        }else if pred.get_tag() == "direction" {
            let decoded_pointing = decode_calibration_target(&pred, &objects);
            pointing.insert(Satellite(decoded_pointing.0), Direction(decoded_pointing.1));
        }else if pred.get_tag() == "power_avail" {
            power_avail = true;
        } else if pred.get_tag() == "calibrated" {
            calibrated.push(Instrument(*objects.get(pred.get_arg(0)).unwrap()));
        }else if pred.get_tag() == "have_image" {
            let decoded_have_image = decode_calibration_target(&pred, &objects);
            have_image.insert(Direction(decoded_have_image.0), Mode(decoded_have_image.1));
        }else if pred.get_tag() == "calibration_target" {
            let decoded_calibration_target = decode_calibration_target(&pred, &objects);
            calibration_target.insert(Instrument(decoded_calibration_target.0), Direction(decoded_calibration_target.1));
        }
    }
    return SatelliteState::new(onboard,supports,pointing,power_avail,power_on,calibrated,have_image,calibration_target);
}

fn decode_onboard(p: &Predicate, objects: &BTreeMap<String,u32>) -> (u32, Vec<u32>) {
    let satellite = obj_get(p.get_arg(0), objects);
    let mut instruments = vec![];

    for i in 1..p.num_args() {
        instruments.push(obj_get(p.get_arg(i), objects));
    }

    (satellite, instruments)
}

fn decode_supports(p: &Predicate, objects: &BTreeMap<String,u32>) -> (u32, Vec<u32>) {
    let instruments = obj_get(p.get_arg(0), objects);
    let mut modes = vec![];

    for i in 1..p.num_args() {
        modes.push(obj_get(p.get_arg(i), objects));
    }

    (instruments, modes)
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


fn extract_goals(parsed: &PddlProblem, objects: &BTreeMap<String,u32>) -> Vec<(u32,u32)> {
    let mut goals = Vec::new();
    for goal in parsed.goals.iter() {
        goals.push(decode_calibration_target(&goal, &objects));
    }
    goals
}