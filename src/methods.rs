use anyhop::{Atom, Goal, Method, MethodResult, Task};
use anyhop::MethodResult::{PlanFound, TaskLists};
use anyhop::Task::Operator;

use crate::methods::SatelliteMethod::{ScheduleAll, ScheduleOne};
use crate::operators::SatelliteOperator::{Calibrate, SwitchOff, SwitchOn, TakeImage, TurnTo};

use super::operators::*;

#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub enum SatelliteMethod {
    ScheduleAll,
    //state, satellite, instrument, mode, new_direction, previous_direction
    ScheduleOne(SatelliteEnum, SatelliteEnum, SatelliteEnum, SatelliteEnum, SatelliteEnum),
    //SatelliteState, Satellite, Instrument
    Switching(SatelliteEnum, SatelliteEnum),
}

#[derive(Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub enum SatelliteStatus {
    Done,
    //state, satellite, instrument, mode, new_direction, previous_direction
    NotDone(u32, SatelliteEnum, SatelliteEnum, SatelliteEnum, SatelliteEnum, SatelliteEnum),
}


impl SatelliteStatus {
    pub fn new(identifier: u32, state: SatelliteState, satellite: SatelliteEnum, instrument: SatelliteEnum, mode: SatelliteEnum, new_direction: SatelliteEnum, previous_direction: SatelliteEnum, goal: SatelliteGoals) -> SatelliteStatus {
        return if is_satellite_done(state, &goal) {
            SatelliteStatus::Done
        } else {
            SatelliteStatus::NotDone(identifier, satellite, instrument, mode, new_direction, previous_direction)
        }
    }
}

pub fn is_satellite_done(state: SatelliteState, goal: &SatelliteGoals) -> bool {
    for goal_image in goal.have_image.keys() {
        if !state.have_image.contains_key(goal_image) {
            return false;
        } else {
            if !(state.have_image.get(goal_image) == goal.have_image.get(goal_image)) {
                return false;
            }
        }
    }
    return true;
}

fn switching(state: &SatelliteState, satellite: SatelliteEnum, instrument: SatelliteEnum) -> MethodResult<SatelliteOperator<SatelliteEnum>, SatelliteMethod> {
    TaskLists(vec![if !state.power_on.is_empty() && !state.power_on.contains(&instrument) {
        vec![Operator(SwitchOff(instrument, satellite)),
             Operator(SwitchOn(instrument, satellite))]
    } else if state.power_on.is_empty() {
        vec![Operator(SwitchOn(instrument, satellite))]
    } else {
        vec![]
    }])
}

fn schedule_one(state: &SatelliteState, satellite: SatelliteEnum, instrument: SatelliteEnum, mode: SatelliteEnum, new_direction: SatelliteEnum, previous_direction: SatelliteEnum) -> MethodResult<SatelliteOperator<SatelliteEnum>, SatelliteMethod> {
    use SatelliteMethod::*;
    use MethodResult::*;
    use Task::*;

    let is_instrument_powered_on = state.power_on.contains(&instrument);

    if pointing_helper(state,&satellite, &new_direction){ //Prevents short circuiting of the and from earlier
         if is_instrument_powered_on || state.power_on.is_empty(){
            return TaskLists(vec![vec![Method(Switching(satellite, instrument)),
                                Operator(Calibrate(satellite, instrument, new_direction)),
                                Operator(TakeImage(satellite, new_direction, instrument, mode))]])
        } else {
            let last_instrument = state.power_on[state.power_on.len() - 1];

            return TaskLists(vec![vec![Operator(SwitchOff(last_instrument, satellite)),
                                Method(Switching(satellite, instrument)),
                                Operator(Calibrate(satellite, instrument, new_direction)),
                                Operator(TakeImage(satellite, new_direction, instrument, mode))]])
        }
    }else{
        if is_instrument_powered_on || state.power_on.is_empty(){
            let calibration_target_direction = state.calibration_target.get(&instrument).unwrap();

            TaskLists(vec![vec![Operator(TurnTo(satellite, *calibration_target_direction, previous_direction)),
                                Method(Switching(satellite, instrument)),
                                Operator(Calibrate(satellite, instrument, *calibration_target_direction)),
                                Operator(TurnTo(satellite, new_direction, *calibration_target_direction)),
                                Operator(TakeImage(satellite, new_direction, instrument, mode))]])
        }else{
            let calibration_target_direction = state.calibration_target.get(&instrument).unwrap();
            let last_instrument = state.power_on[state.power_on.len()-1];


            TaskLists(vec![vec![Operator(SwitchOff(last_instrument, satellite)),
                                Operator(TurnTo(satellite, *calibration_target_direction, previous_direction)),
                                Method(Switching(satellite, instrument)),
                                Operator(Calibrate(satellite, instrument, *calibration_target_direction)),
                                Operator(TurnTo(satellite, new_direction, *calibration_target_direction)),
                                Operator(TakeImage(satellite, new_direction, instrument, mode))]])
        }

    }
}

fn pointing_helper(state: &SatelliteState, satellite: &SatelliteEnum, direction: &SatelliteEnum) -> bool {
    return match state.pointing.get(satellite) {
        Some(x) => x == direction, //If we have the correct instrument selected, we need to make sure that it is selected at the right direction.
        None => false, //If the lookup fails, the if statement should fail.
    };
}

fn schedule_all(state: &SatelliteState, goal: &SatelliteGoals) -> MethodResult<SatelliteOperator<SatelliteEnum>, SatelliteMethod> {
    let mut tasks: Vec<Vec<Task<SatelliteOperator<SatelliteEnum>, SatelliteMethod>>> = vec![];
    let mut completed_tasks: Vec<SatelliteEnum> = vec![];

    for goal_image in goal.have_image.keys() {

        if !(state.have_image.get(goal_image) == goal.have_image.get(goal_image)) {
            let goal_image_clone = goal_image.clone();
            let mode = goal.have_image.get(goal_image).unwrap();
            let instrument = brute_force_instrument(state, mode).unwrap(); //First look up the goal image to see which mode it should be in, and then look up which mode it should be in.
            let new_direction = goal_image_clone;

            let satellite = brute_force_satellite(state, &instrument, mode).unwrap();
            let previous_direction = state.pointing.get(&satellite.clone()).unwrap();

            tasks.push(vec![Task::Method(ScheduleOne(satellite, instrument, mode.clone(), new_direction, previous_direction.clone())), Task::Method(ScheduleAll)]);
        } else {
            let image_clone = goal_image.clone();
            completed_tasks.push(image_clone);
        }

    }

    return if goal.have_image.keys().eq(&completed_tasks) && does_pass_pointing_check(state, goal){
        PlanFound
    } else {
        TaskLists(tasks)
    };
}

fn does_pass_pointing_check (state: &SatelliteState, goal: &SatelliteGoals) -> bool{
    for satellite in goal.pointing.keys(){
        let gotten_direction = state.pointing.get(satellite);
        if gotten_direction == None{
            return false;
        }else if gotten_direction != goal.pointing.get(satellite){
            return false;
        }
    }
    return true;
}

fn brute_force_instrument(state: &SatelliteState, mode: &SatelliteEnum)  -> Option<SatelliteEnum> {
    for instrument in state.supports.keys(){
        if state.supports.get(instrument)?.contains(mode){
            return Some(instrument.clone());
        }
    }
    return None;
}

fn brute_force_satellite(state: &SatelliteState, instrument: &SatelliteEnum, mode: &SatelliteEnum) -> Option<SatelliteEnum> {
    for satellites in state.onboard.keys() {
        if state.does_instrument_support_mode(instrument, mode){
            return Some(satellites.clone());
        }
    }
    return None;
}

impl Method for SatelliteMethod {
    type S = SatelliteState;
    type G = SatelliteGoals;
    type O = SatelliteOperator<SatelliteEnum>;

    fn apply(&self, state: &SatelliteState, goal: &SatelliteGoals) -> MethodResult<SatelliteOperator<SatelliteEnum>, SatelliteMethod> {
        use SatelliteMethod::*;
        match self {
            ScheduleAll => schedule_all(state, goal),
            ScheduleOne(satellite, instrument, mode, new_direction, previous_direction) => schedule_one(state, satellite.clone(), instrument.clone(), mode.clone(), new_direction.clone(), previous_direction.clone()),
            Switching(satellite, instrument) => switching(state, satellite.clone(), instrument.clone()),
        }
    }
}

impl Goal for SatelliteGoals {
    type O = SatelliteOperator<SatelliteEnum>;
    type M = SatelliteMethod;
    type S = SatelliteState;

    fn starting_tasks(&self) -> Vec<Task<SatelliteOperator<SatelliteEnum>, SatelliteMethod>> {
        vec![Task::Method(SatelliteMethod::ScheduleAll)]
    }
    fn accepts(&self, state: &Self::S) -> bool {
        for (location,instrument) in self.have_image.iter(){
            let state_instrument = state.have_image.get(location);

            if state_instrument == None || state_instrument != Some(instrument) {
                println!("!!!We have failed the have_image checker!");
                println!("!!!Goal have_image: {:?}", self.have_image);
                println!("!!!Actual have_image: {:?}", state.have_image);
                return false;
            }
        }

        for (satellite, direction) in self.pointing.iter(){
            let state_direction = state.pointing.get(satellite);

            if state_direction == None || state_direction != Some(direction){
                println!("We have failed the pointing checker!");
                println!("!!!Goal pointing: {:?}", self.pointing);
                println!("!!!Actual pointing: {:?}", state.pointing);
                return false;
            }
        }
        println!("This plan has been accepted by the checker!");
        return true;
    }
}

