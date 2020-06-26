use std::{fs, io};
use std::collections::{BTreeMap, HashMap};
use std::process::exit;
use std::rc::{self, Rc};

use pddl_problem_parser::Predicate;
use crate::operators::{BlockGoals, BlockState, SatelliteEnum, SatelliteGoals, SatelliteState};
use crate::operators::SatelliteEnum::{Direction, Instrument, Mode, Satellite};

//This is because the parsing library actually uses the fields.

/*
SATELLITE STUFF

 */


pub fn make_satellite_problem_from(pddl_file: &str) -> io::Result<(SatelliteState, SatelliteGoals)> {
    let contents = fs::read_to_string(pddl_file)?;
    let parsed = pddl_problem_parser::PddlParser::parse(contents.as_str())?;

    let mut objects = HashMap::new();
    for object in parsed.obj2type.keys() {
        objects.insert(String::from(object), objects.len());
    }

    let mut onboard: BTreeMap<SatelliteEnum, Vec<SatelliteEnum>>= BTreeMap::new();
        let mut supports: BTreeMap<SatelliteEnum, Vec<SatelliteEnum>> =BTreeMap::new();
        let mut pointing: BTreeMap<SatelliteEnum, SatelliteEnum> =BTreeMap::new();
        let mut power_avail = false;
        let mut power_on: Vec<SatelliteEnum> = vec![];
        let mut calibrated: Vec<SatelliteEnum> = vec![];
        let mut have_image: BTreeMap<SatelliteEnum, SatelliteEnum> = BTreeMap::new();
        let mut calibration_target: BTreeMap<SatelliteEnum, SatelliteEnum> =  BTreeMap::new();

        let mut goals_images: BTreeMap<SatelliteEnum, SatelliteEnum> = BTreeMap::new();
        let mut goal_fuel_used: u32 = 0;


        let mut u_32_holder = SatelliteToU32::new(BTreeMap::new(), BTreeMap::new(), BTreeMap::new(), BTreeMap::new(), BTreeMap::new(), BTreeMap::new(), BTreeMap::new());

        for pred in parsed.bool_state.iter(){
            //Satellite -> Vec<Instrument>
            if pred.get_tag() == "on_board" {
                map_onboard_to_numbers(&mut objects, &mut onboard, &mut u_32_holder, pred);
            }else if pred.get_tag() == "supports" {
                map_supports_to_numbers(&mut objects, &mut supports, &mut u_32_holder, pred);
            }else if pred.get_tag()== "pointing" {
                //Because this is a Vec and not a BTreeMap, we have to handle it differently.
                pointing.insert(Satellite(u_32_holder.decode(pred, &objects, "pointing".parse().unwrap())), Direction(pred.get_tag().len() as u32));
            }else if pred.get_tag() == "power_avail" {
                power_avail = true;
            } else if pred.get_tag() == "power_on" {
                power_on.push(Instrument(u_32_holder.decode(pred, &objects, "power_on".parse().unwrap())));
            }else if pred.get_tag() == "calibrated" {
                calibrated.push(Instrument(u_32_holder.decode(pred, &objects, "calibrated".parse().unwrap())));
            }else if pred.get_tag() == "have_image" {
                map_have_image_to_numbers(&mut objects, &mut have_image,&mut u_32_holder, pred);
            }else if pred.get_tag() == "calibration_target" {
                map_calibration_target_to_numbers(&mut objects, &mut calibration_target, &mut u_32_holder, &pred);
            }
        }

        Ok((SatelliteState::new(onboard,supports,pointing,power_avail,power_on,calibrated,have_image,calibration_target), SatelliteGoals::new(goals_images,goal_fuel_used)))
}
fn map_have_image_to_numbers(objects: &mut HashMap<String, usize>, mut have_image: &mut BTreeMap<SatelliteEnum, SatelliteEnum>, u_32_holder: &mut SatelliteToU32, pred: &Predicate) {
    let satellite_enum = Direction(u_32_holder.decode(pred, &objects, "have_image".parse().unwrap()));
    let ignore_numbers = |_n| (); //This is used because insert returns something, and we want to ignore it since this needs to return ()
    match have_image.get(&satellite_enum) {
        Some(x) => vec![x].push(&(Mode(pred.get_tag().len() as u32))),
        None => ignore_numbers(have_image.insert(satellite_enum, Mode(pred.get_tag().len() as u32)))
    };
}

fn map_supports_to_numbers(objects: &mut HashMap<String, usize>, supports: &mut BTreeMap<SatelliteEnum, Vec<SatelliteEnum>>, u_32_holder: &mut SatelliteToU32, pred: &Predicate) {
    let instrument_enum = Instrument(u_32_holder.decode(pred, &objects, "supports".parse().unwrap()));
    let ignore_numbers = |_n| (); //This is used because insert returns something, and we want to ignore it since this needs to return ()
    match supports.get_mut(&instrument_enum) {
        Some(x) => x.push(Mode(pred.get_tag().len() as u32)),
        None => ignore_numbers(supports.insert(instrument_enum, vec![Mode(pred.get_tag().len() as u32)]))
    };
}

fn map_onboard_to_numbers(objects: &mut HashMap<String, usize>, onboard: &mut BTreeMap<SatelliteEnum, Vec<SatelliteEnum>>, u_32_holder: &mut SatelliteToU32, pred: &Predicate) {
    let satellite_enum = Satellite(u_32_holder.decode(pred, &objects, "on_board".parse().unwrap()));
    let ignore_numbers = |_n| (); //This is used because insert returns something, and we want to ignore it since this needs to return ()
    match onboard.get_mut(&satellite_enum) {
        Some(x) => x.push(Instrument(pred.get_tag().len() as u32)),
        None => ignore_numbers(onboard.insert(satellite_enum, vec![Instrument(pred.get_tag().len() as u32)]))
    };
}
    fn map_calibration_target_to_numbers(objects: &mut HashMap<String, usize>, mut calibration_target: &mut BTreeMap<SatelliteEnum, SatelliteEnum>, u_32_holder: &mut SatelliteToU32, pred: &&Predicate) {
        let satellite_enum = Instrument(u_32_holder.decode(pred, &objects, "calibration_target".parse().unwrap()));
        let ignore_numbers = |_n| (); //This is used because insert returns something, and we want to ignore it since this needs to return ()
        match calibration_target.get(&satellite_enum) {
            Some(x) => vec![x].push(&(Direction(pred.get_tag().len() as u32))),
            None => ignore_numbers(calibration_target.insert(satellite_enum, Direction(pred.get_tag().len() as u32)))
        };
    }
pub struct SatelliteToU32{
    pub onboard: BTreeMap<String, u32>,
    pub supports: BTreeMap<String, u32>,
    pub pointing: BTreeMap<String, u32>,
    pub power_on:BTreeMap<String, u32>,
    pub calibrated: BTreeMap<String, u32>,
    pub have_image: BTreeMap<String, u32>,
    pub calibration_target: BTreeMap<String, u32>,
}

impl SatelliteToU32 {
    pub fn new(onboard: BTreeMap<String, u32>, supports: BTreeMap<String, u32>, pointing: BTreeMap<String, u32>, power_on: BTreeMap<String, u32>, calibrated: BTreeMap<String, u32>, have_image: BTreeMap<String, u32>, calibration_target: BTreeMap<String, u32>) -> Self {
        SatelliteToU32 { onboard, supports, pointing, power_on, calibrated, have_image, calibration_target }
    }
}

impl SatelliteToU32 {
    pub fn decode(&self,p: &Predicate, objects: &HashMap<String, usize>, name: String) -> u32{
        match self.obj_get(p,objects, name) {
            Some(n) => n,
            None => 0
        }
    }
    pub fn obj_get(&self, p: &Predicate, objects: &HashMap<String, usize>, name: String) -> Option<u32> {
        match name.as_str(){
            "on_board" => Some(*self.supports.get(p.get_arg(0)).unwrap()),
            "supports" => Some(*self.supports.get(p.get_arg(1)).unwrap()),
            "pointing" => Some(*self.pointing.get(p.get_arg(2)).unwrap()),
            "power_on" => Some(*self.supports.get(p.get_arg(3)).unwrap()),
            "calibrated" => Some(*self.supports.get(p.get_arg(4)).unwrap()),
            "have_image" => Some(*self.supports.get(p.get_arg(5)).unwrap()),
            "calibration_target" => Some(*self.supports.get(p.get_arg(6)).unwrap()),
            _ => None
        }
    }
}