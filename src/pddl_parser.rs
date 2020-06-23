use sexpy::*;
use crate::operators::{BlockState, BlockGoals, SatelliteState, SatelliteGoals, SatelliteEnum};
use std::{io,fs};
use std::collections::{HashMap, BTreeMap};

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


// pub fn make_block_problem_from(pddl_file: &str) -> io::Result<(BlockState<usize>, BlockGoals<usize>)> {
//     let contents = fs::read_to_string(pddl_file)?.to_lowercase();
//     match Define::parse(contents.as_str()) {
//         Ok(parsed) => Ok(parsed.init_and_goal()),
//         Err(e) => {println!("{}", e); Err(err!(Other, "oops"))}
//     }
// }

// pub fn make_satellite_problem_from(pddl_file: &str) -> io::Result<(SatelliteState,SatelliteGoals)>{
//     let contents = fs::read_to_string(pddl_file)?.to_lowercase();
//     match Define::parse(contents.as_str()) {
//         Ok(parsed) => Ok(parsed.init_and_goal()),
//         Err(e) => {println!("{}", e); Err(err!(Other, "oops"))}
//     }
// }

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

// struct


// impl Define{
//     pub fn init_and_goal(&self)->(SatelliteState,SatelliteGoals){
//
//
//
//         let mut objects = HashMap::new();
//         for object in self.objects.objs.iter() {
//             objects.insert(String::from(object), objects.len());
//         }
//
//         //The predicates we have are as follows:
//         let mut onboard: BTreeMap<SatelliteEnum, Vec<SatelliteEnum>>= BTreeMap::new();
//         let mut supports: BTreeMap<SatelliteEnum, SatelliteEnum> =BTreeMap::new();
//         let mut pointing: BTreeMap<SatelliteEnum, SatelliteEnum> =BTreeMap::new();
//         let mut power_avail = false;
//         let mut power_on: Vec<SatelliteEnum> = vec![];
//         let mut calibrated: Vec<SatelliteEnum> = vec![];
//         let mut have_image: BTreeMap<SatelliteEnum, SatelliteEnum> = BTreeMap::new();
//         let mut calibration_target: BTreeMap<SatelliteEnum, SatelliteEnum> =  BTreeMap::new();
//
//         let mut goals_images: BTreeMap<SatelliteEnum, SatelliteEnum> = BTreeMap::new();
//         let mut goal_fuel_used: u32 = 0;
//
//
//
//
//         for pred in self.init.predicates.iter(){
//             if pred.predicate_type == "onboard".parse().unwrap() {
//                 onboard.insert(SatelliteEnum::from(decode_on(pred, &objects)), pred.len());
//             }else if pred.predicate_type == "supports".parse().unwrap() {
//                 supports.insert(SatelliteEnum::from(pred), pred.len());
//             }else if pred.predicate_type== "pointing".parse().unwrap() {
//                 pointing.insert(SatelliteEnum::from(pred), pred.len());
//             }else if pred.predicate_type == "power_avail".parse().unwrap() {
//                 power_avail = true;
//             } else if pred.predicate_type == "power_on".parse().unwrap() {
//                 power_on.push(SatelliteEnum::from(pred));
//             }else if pred.predicate_type == "calibrated".parse().unwrap() {
//                 calibrated.push(SatelliteEnum::from(pred));
//             }else if pred.predicate_type == "have_image".parse().unwrap() {
//                 have_image.insert(SatelliteEnum::from(pred), pred.len());
//             }else if pred.predicate_type == "calibration_target".parse().unwrap() {
//                 calibration_target.insert(SatelliteEnum::from(pred), pred.len());
//             }
//         }
//
//
//         (SatelliteState::new(onboard,supports,pointing,power_avail,power_on,calibrated,have_image,calibration_target), SatelliteGoals::new(goals_images,  goal_fuel_used));
//     }
// }

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

pub struct SatelliteAsString{
    pub onboard: BTreeMap<String, u32>,
    pub supports: BTreeMap<String, u32>,
    pub pointing: BTreeMap<String, u32>,
    pub power_on:BTreeMap<String, u32>,
    pub calibrated: BTreeMap<String, u32>,
    pub have_image: BTreeMap<String, u32>,
    pub calibration_target: BTreeMap<String, u32>,
}
impl SatelliteAsString{
    /*
    fn decode_on(p: &Predicate, objects: &HashMap<String,usize>) -> (usize, usize) {
    let top = obj_get(p, objects, 0);
    let bottom = obj_get(p, objects, 1);
    (top, bottom)
}
fn obj_get(p: &Predicate, objects: &HashMap<String,usize>, i: usize) -> usize {

    *objects.get(p.predicate_args[i].as_str()).unwrap()
}
     */
    pub fn decode(self,p: &Predicate, objects: &HashMap<String, usize>, name: String) -> u32{
        match self.obj_get(p,objects, name) {
            Some(n) => n,
            None => 0
            }
        }
    pub fn obj_get(self, p: &Predicate, objects: &HashMap<String, usize>, name: String) -> Option<u32> {
        match name.as_str(){
            "onboard" => Some(*self.onboard.get(p.predicate_args[0].as_str())?),
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
