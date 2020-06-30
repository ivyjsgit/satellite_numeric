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

fn schedule_one(_state: &SatelliteState, satellite: SatelliteEnum, instrument: SatelliteEnum, mode: SatelliteEnum, new_direction: SatelliteEnum, previous_direction: SatelliteEnum) -> MethodResult<SatelliteOperator<SatelliteEnum>, SatelliteMethod> {
    use SatelliteMethod::*;
    use MethodResult::*;
    use Task::*;
    TaskLists(vec![vec![Operator(TurnTo(satellite, new_direction, previous_direction)),
                        Method(Switching(satellite, instrument)),
                        Operator(Calibrate(satellite, instrument, new_direction)),
                        Operator(TakeImage(satellite, new_direction, instrument, mode))]])
}

fn schedule_all(state: &SatelliteState, goal: &SatelliteGoals) -> MethodResult<SatelliteOperator<SatelliteEnum>, SatelliteMethod> {
    let mut tasks: Vec<Vec<Task<SatelliteOperator<SatelliteEnum>, SatelliteMethod>>> = vec![vec![]];
    let mut completed_tasks: Vec<SatelliteEnum> = vec![];

    for goal_image in goal.have_image.keys() {
        if !(state.have_image.get(goal_image) == goal.have_image.get(goal_image)) {
            let goal_image_clone = goal_image.clone();
            let mode = goal.have_image.get(goal_image).unwrap();
            println!("\n\n");
            let instrument = brute_force_instrument(state, mode).unwrap(); //First look up the goal image to see which mode it should be in, and then look up which mode it should be in.
            let new_direction = goal_image_clone;

            let satellite = brute_force_satellite(state, &instrument, mode).unwrap();
            let previous_direction = state.pointing.get(&satellite.clone()).unwrap();

            // tasks.push(vec![Task::Method(ScheduleOne(goal_image_clone)),Task::Method(ScheduleAll)]);
            tasks.push(vec![Task::Method(ScheduleOne(satellite, instrument, mode.clone(), new_direction, previous_direction.clone())), Task::Method(ScheduleAll)]);
        } else {
            let image_clone = goal_image.clone();
            completed_tasks.push(image_clone);
        }
    }

    return if goal.have_image.keys().eq(&completed_tasks) {
        PlanFound
    } else {
        TaskLists(tasks)
    };
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

    fn starting_tasks(&self) -> Vec<Task<SatelliteOperator<SatelliteEnum>, SatelliteMethod>> {
        vec![Task::Method(SatelliteMethod::ScheduleAll)]
    }
}

