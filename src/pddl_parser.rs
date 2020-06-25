use std::{fs, io};
use std::collections::{BTreeMap, HashMap};
use std::process::exit;
use std::rc::{self, Rc};

//This is because the parsing library actually uses the fields.

use sexpy::*;

use crate::operators::{BlockGoals, BlockState, SatelliteEnum, SatelliteGoals, SatelliteState};
use crate::operators::SatelliteEnum::{Direction, Instrument, Mode, Satellite};

// pub fn make_block_problem_from(pddl_file: &str) -> io::Result<(BlockState<usize>, BlockGoals<usize>)> {
//     let contents = fs::read_to_string(pddl_file)?.to_lowercase();
//     match Define::parse(contents.as_str()) {
//         Ok(parsed) => Ok(parsed.init_and_goal()),
//         Err(e) => {println!("{}", e); Err(err!(Other, "oops"))}
//     }
// }
//
// #[derive(Sexpy)]
// struct Define {
//     problem: Problem,
//     domain: Domain,
//     objects: Objects,
//     init: Init,
//     goal: Goal
// }
//
// impl Define {
//     pub fn init_and_goal(&self) -> (BlockState<usize>, BlockGoals<usize>) {
//         let mut objects = HashMap::new();
//         for object in self.objects.objs.iter() {
//             objects.insert(String::from(object), objects.len());
//         }
//         let mut table = Vec::new();
//         let mut stacks = Vec::new();
//         for pred in self.init.predicates.iter() {
//             if pred.predicate_type == "ontable" {
//                 table.push(*objects.get(pred.predicate_args[0].as_str()).unwrap());
//             } else if pred.predicate_type == "on" {
//                 stacks.push(decode_on(&pred, &objects));
//             }
//         }
//
//         let mut goals = Vec::new();
//         for goal in self.goal.and.goals.iter() {
//             goals.push(decode_on(&goal, &objects));
//         }
//
//         (BlockState::from(table, stacks), BlockGoals::new(goals))
//     }
// }
//
//
// fn decode_on(p: &Predicate, objects: &HashMap<String,usize>) -> (usize, usize) {
//     let top = obj_get(p, objects, 0);
//     let bottom = obj_get(p, objects, 1);
//     (top, bottom)
// }
//
// fn obj_get(p: &Predicate, objects: &HashMap<String,usize>, i: usize) -> usize {
//     *objects.get(p.predicate_args[i].as_str()).unwrap()
// }
//
// #[derive(Sexpy)]
// struct Problem {
//     name: String
// }
//
// #[derive(Sexpy)]
// #[sexpy(head=":domain")]
// struct Domain {
//     name: String
// }
//
// #[derive(Sexpy)]
// #[sexpy(head=":objects")]
// struct Objects {
//     objs: Vec<String>
// }
//
// #[derive(Sexpy)]
// #[sexpy(head=":init")]
// struct Init {
//     predicates: Vec<Predicate>
// }
//
// #[derive(Sexpy)]
// #[sexpy(nohead)]
// struct Predicate {
//     predicate_type: String,
//     predicate_args: Vec<String>
// }
//
// #[derive(Sexpy)]
// #[sexpy(head=":goal")]
// struct Goal {
//     and: And
// }
//
// #[derive(Sexpy)]
// struct And {
//     goals: Vec<Predicate>
// }

/*

SATELLITE STUFF

 */

pub fn make_satellite_problem_from(pddl_file: &str) -> io::Result<(SatelliteState,SatelliteGoals)>{
    let contents = fs::read_to_string(pddl_file)?.to_lowercase();
    match Define::parse(contents.as_str()) {
        Ok(parsed) => Ok(parsed.init_and_goal()),
        Err(e) => {println!("{}", e); Err(err!(Other, "oops"))}
    }
}

#[derive(Sexpy)]
struct Define {
    problem: Problem,
    domain: Domain,
    objects: Objects,
    init: Init,
    goal: Goal
}

fn decode_on(p: &Predicate, objects: &HashMap<String,usize>) -> (usize, usize) {
    let top = obj_get(p, objects, 0);
    let bottom = obj_get(p, objects, 1);
    (top, bottom)
}
fn obj_get(p: &Predicate, objects: &HashMap<String,usize>, i: usize) -> usize {

    *objects.get(p.predicate_args[i].as_str()).unwrap()
}


impl Define{
    pub fn init_and_goal(&self)->(SatelliteState,SatelliteGoals){

        let mut objects = HashMap::new();
        for object in self.objects.objs.iter() {
            objects.insert(String::from(object), objects.len());
        }

        //The predicates we have are as follows:

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

        for pred in self.init.predicates.iter(){
            //Satellite -> Vec<Instrument>
            if pred.predicate_type == "on_board" {
                Define::map_onboard_to_numbers(&mut objects, &mut onboard, &mut u_32_holder, pred);
            }else if pred.predicate_type == "supports" {
                Define::map_supports_to_numbers(&mut objects, &mut supports, &mut u_32_holder, pred);
            }else if pred.predicate_type== "pointing" {
                //Because this is a Vec and not a BTreeMap, we have to handle it differently.
                pointing.insert(Satellite(u_32_holder.decode(pred, &objects, "pointing".parse().unwrap())), Direction(pred.predicate_type.len() as u32));
            }else if pred.predicate_type == "power_avail" {
                power_avail = true;
            } else if pred.predicate_type == "power_on" {
                power_on.push(Instrument(u_32_holder.decode(pred, &objects, "power_on".parse().unwrap())));
            }else if pred.predicate_type == "calibrated" {
                calibrated.push(Instrument(u_32_holder.decode(pred, &objects, "calibrated".parse().unwrap())));
            }else if pred.predicate_type == "have_image" {
                Define::map_have_image_to_numbers(&mut objects, &mut have_image,&mut u_32_holder, pred);
            }else if pred.predicate_type == "calibration_target" {
                Define::map_calibration_target_to_numbers(&mut objects, &mut calibration_target, &mut u_32_holder, &pred);
            }
        }


        return (SatelliteState::new(onboard,supports,pointing,power_avail,power_on,calibrated,have_image,calibration_target), SatelliteGoals::new(goals_images,  goal_fuel_used));
    }

    fn map_calibration_target_to_numbers(objects: &mut HashMap<String, usize>, mut calibration_target: &mut BTreeMap<SatelliteEnum, SatelliteEnum>, u_32_holder: &mut SatelliteToU32, pred: &&Predicate) {
        let satellite_enum = Instrument(u_32_holder.decode(pred, &objects, "calibration_target".parse().unwrap()));
        let ignore_numbers = |_n| (); //This is used because insert returns something, and we want to ignore it since this needs to return ()
        match calibration_target.get(&satellite_enum) {
            Some(x) => vec![x].push(&(Direction(pred.predicate_type.len() as u32))),
            None => ignore_numbers(calibration_target.insert(satellite_enum, Direction(pred.predicate_type.len() as u32)))
        };
    }

    fn map_have_image_to_numbers(objects: &mut HashMap<String, usize>, mut have_image: &mut BTreeMap<SatelliteEnum, SatelliteEnum>, u_32_holder: &mut SatelliteToU32, pred: &Predicate) {
        let satellite_enum = Direction(u_32_holder.decode(pred, &objects, "have_image".parse().unwrap()));
        let ignore_numbers = |_n| (); //This is used because insert returns something, and we want to ignore it since this needs to return ()
        match have_image.get(&satellite_enum) {
            Some(x) => vec![x].push(&(Mode(pred.predicate_type.len() as u32))),
            None => ignore_numbers(have_image.insert(satellite_enum, Mode(pred.predicate_type.len() as u32)))
        };
    }

    fn map_supports_to_numbers(objects: &mut HashMap<String, usize>, supports: &mut BTreeMap<SatelliteEnum, Vec<SatelliteEnum>>, u_32_holder: &mut SatelliteToU32, pred: &Predicate) {
        let instrument_enum = Instrument(u_32_holder.decode(pred, &objects, "supports".parse().unwrap()));
        let ignore_numbers = |_n| (); //This is used because insert returns something, and we want to ignore it since this needs to return ()
        match supports.get_mut(&instrument_enum) {
            Some(x) => x.push(Mode(pred.predicate_type.len() as u32)),
            None => ignore_numbers(supports.insert(instrument_enum, vec![Mode(pred.predicate_type.len() as u32)]))
        };
    }

    fn map_onboard_to_numbers(objects: &mut HashMap<String, usize>, onboard: &mut BTreeMap<SatelliteEnum, Vec<SatelliteEnum>>, u_32_holder: &mut SatelliteToU32, pred: &Predicate) {
        let satellite_enum = Satellite(u_32_holder.decode(pred, &objects, "on_board".parse().unwrap()));
        let ignore_numbers = |_n| (); //This is used because insert returns something, and we want to ignore it since this needs to return ()
        match onboard.get_mut(&satellite_enum) {
            Some(x) => x.push(Instrument(pred.predicate_type.len() as u32)),
            None => ignore_numbers(onboard.insert(satellite_enum, vec![Instrument(pred.predicate_type.len() as u32)]))
        };
    }


}

#[derive(Sexpy)]
struct Problem {
    name: String
}

#[derive(Sexpy)]
#[sexpy(head=":domain")]
struct Domain {
    name: String
}

#[derive(Sexpy)]
#[sexpy(head=":objects")]
struct Objects {
    objs: Vec<String>
}

#[derive(Sexpy)]
#[sexpy(head=":init")]
struct Init {
    predicates: Vec<Predicate>
}

#[derive(Sexpy)]
#[sexpy(nohead)]
pub struct Predicate {
    predicate_type: String,
    predicate_args: Vec<String>,
    nested_predicate: Vec<NestedPredicate>
}

#[derive(Sexpy)]
#[sexpy(head ="=")]
struct NestedPredicate {
    predicate_type: String,
    predicate_args: Vec<String>
}

#[derive(Sexpy)]
#[sexpy(head=":goal")]
struct Goal {
    and: And
}

#[derive(Sexpy)]
struct And {
    goals: Vec<Predicate>
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
            "on_board" => Some(*self.onboard.get(p.predicate_args[0].as_str())?),
            "supports" => Some(*self.supports.get(p.predicate_args[0].as_str())?),
            "pointing" => Some(*self.pointing.get(p.predicate_args[0].as_str())?),
            "power_on" => Some(*self.supports.get(p.predicate_args[0].as_str())?),
            "calibrated" => Some(*self.supports.get(p.predicate_args[0].as_str())?),
            "have_image" => Some(*self.supports.get(p.predicate_args[0].as_str())?),
            "calibration_target" => Some(*self.supports.get(p.predicate_args[0].as_str())?),
            _ => None
        }
    }
}
