use anyhop::{Atom, Goal, Method, MethodResult, Task};
use anyhop::MethodResult::{TaskLists};
use anyhop::Task::Operator;
use MethodResult::*;
use Task::*;
use log::{debug, error, info, trace, warn};


use SatelliteMethod::*;

use crate::methods::SatelliteMethod::{ScheduleAll, ScheduleOne};
use crate::operators::SatelliteOperator::{Calibrate, SwitchOff, SwitchOn, TakeImage, TurnTo};

use super::operators::*;
use fixed::types::I40F24;

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
//Checks to see if the planner is done by comparing the state and the goal.
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

//Turn an instrument on or off.
fn switching(state: &SatelliteState, satellite: SatelliteEnum, instrument: SatelliteEnum) -> MethodResult<SatelliteOperator<SatelliteEnum>, SatelliteMethod> {
    TaskLists(vec![if !state.power_on.is_empty() && !state.power_on.contains(&instrument) {
        let powered_on_instrument = find_powered_on_instruments(state, &satellite);
        debug!("Our powered_on instrument is: {:?}", powered_on_instrument);
        match powered_on_instrument{
            Some(n) => debug!("Performing switchoff on SwitchOff({:?}, {:?})",n, satellite ),
            None => debug!("None"),

        }
        match powered_on_instrument{

            Some(n)=> vec![Operator(SwitchOff(n, satellite)),
                     Operator(SwitchOn(instrument, satellite))],


            None =>
                {
                    if state.power_on.contains(&instrument){
                        vec![Operator(SwitchOff(instrument, satellite)),
                             Operator(SwitchOn(instrument, satellite))]
                    }else{
                        vec![Operator(SwitchOn(instrument, satellite))]
                    }

                }
        }
    } else if state.power_on.is_empty() {
        vec![Operator(SwitchOn(instrument, satellite))]
    } else {
        vec![]
    }])

}
//Schedules one step into the planner.
fn schedule_one(state: &SatelliteState, satellite: SatelliteEnum, instrument: SatelliteEnum, mode: SatelliteEnum, new_direction: SatelliteEnum, previous_direction: SatelliteEnum) -> MethodResult<SatelliteOperator<SatelliteEnum>, SatelliteMethod> {
    let is_instrument_powered_on = !state.power_avail.get(&satellite).unwrap();

    if is_satellite_pointing_in_direction(state, &satellite, &new_direction){ //Prevents short circuiting of the and from earlier
         if is_instrument_powered_on || state.power_on.is_empty(){
             debug!("Scheduling pointing with on instruments");
            return schedule_pointing_with_powered_on_instruments(satellite, instrument, mode, new_direction)
        } else {
             debug!("Scheduling pointing with off instruments");
             return schedule_pointing_with_powered_off_instruments(state, &satellite, instrument, mode, new_direction)

        }
    }else{
        if is_instrument_powered_on || state.power_on.is_empty(){
            debug!("Scheduling no pointing with on instruments");
            let calibration_target_direction = state.calibration_target.get(&instrument).unwrap();
            return schedule_not_pointing_with_powered_on_instruments(satellite, instrument, mode, new_direction, previous_direction, calibration_target_direction)
        }else{
            debug!("Scheduling no pointing with off instruments");
            let calibration_target_direction = state.calibration_target.get(&instrument).unwrap();
            return schedule_not_pointing_with_powered_off_instruments(state, &satellite, instrument, mode, new_direction, previous_direction, calibration_target_direction)
        }

    }
}

// New function
fn remove_redundant_turns(tasks: Vec<Task<SatelliteOperator<SatelliteEnum>, SatelliteMethod>>) -> Vec<Task<SatelliteOperator<SatelliteEnum>, SatelliteMethod>> {
    tasks.iter().filter(|t| match t {
        Method(_) => true,
        Operator(op) => match op {
            TurnTo(_,dest,start) => dest != start,
            _ => true
        }
    }).map(|t| *t).collect()
}
// Replacement
fn schedule_not_pointing_with_powered_off_instruments(state: &SatelliteState, satellite: &SatelliteEnum, instrument: SatelliteEnum, mode: SatelliteEnum, new_direction: SatelliteEnum, previous_direction: SatelliteEnum, calibration_target_direction: &SatelliteEnum) -> MethodResult<SatelliteOperator<SatelliteEnum>, SatelliteMethod> {
    let powered_on_instrument= find_powered_on_instruments(state, &satellite);
    debug!("Our found powered on instrument is  is {:?}", instrument);
    debug!("Our satellite is {:?}", satellite);
    match powered_on_instrument {
        Some(instrument_to_power_off) =>
            {
                if calibration_target_direction == &previous_direction && &new_direction == calibration_target_direction{
                    TaskLists(vec![vec![Operator(SwitchOff(instrument_to_power_off, *satellite)),
                                        Method(Switching(*satellite, instrument)),
                                        Operator(Calibrate(*satellite, instrument, *calibration_target_direction)),
                                        Operator(TakeImage(*satellite, new_direction, instrument, mode))]])
                } else if calibration_target_direction == &previous_direction {
                    TaskLists(vec![remove_redundant_turns(vec![Operator(SwitchOff(instrument_to_power_off, *satellite)),
                                                               Method(Switching(*satellite, instrument)),
                                                               Operator(Calibrate(*satellite, instrument, *calibration_target_direction)),
                                                               Operator(TurnTo(*satellite, new_direction, *calibration_target_direction)),
                                                               Operator(TakeImage(*satellite, new_direction, instrument, mode))])])
                } else if &new_direction==calibration_target_direction {
                    TaskLists(vec![remove_redundant_turns(vec![Operator(SwitchOff(instrument_to_power_off, *satellite)),
                                                               Operator(TurnTo(*satellite, *calibration_target_direction, previous_direction)),
                                                               Method(Switching(*satellite, instrument)),
                                                               Operator(Calibrate(*satellite, instrument, *calibration_target_direction)),
                                                               Operator(TakeImage(*satellite, new_direction, instrument, mode))])])
                }else{
                    TaskLists(vec![remove_redundant_turns(vec![Operator(SwitchOff(instrument_to_power_off, *satellite)),
                                                               Operator(TurnTo(*satellite, *calibration_target_direction, previous_direction)),
                                                               Method(Switching(*satellite, instrument)),
                                                               Operator(Calibrate(*satellite, instrument, *calibration_target_direction)),
                                                               Operator(TurnTo(*satellite, new_direction, *calibration_target_direction)),
                                                               Operator(TakeImage(*satellite, new_direction, instrument, mode))])])
                }
            },
        None =>   if &new_direction==calibration_target_direction || new_direction==previous_direction{
            TaskLists(vec![vec![Method(Switching(*satellite, instrument)),
                                Operator(Calibrate(*satellite, instrument, *calibration_target_direction)),
                                Operator(TakeImage(*satellite, new_direction, instrument, mode))]])
        }else{
            TaskLists(vec![remove_redundant_turns(vec![Operator(TurnTo(*satellite, *calibration_target_direction, previous_direction)),
                                                       Method(Switching(*satellite, instrument)),
                                                       Operator(Calibrate(*satellite, instrument, *calibration_target_direction)),
                                                       Operator(TurnTo(*satellite, new_direction, *calibration_target_direction)),
                                                       Operator(TakeImage(*satellite, new_direction, instrument, mode))])])
        },
    }
}

fn schedule_not_pointing_with_powered_on_instruments(satellite: SatelliteEnum, instrument: SatelliteEnum, mode: SatelliteEnum, new_direction: SatelliteEnum, previous_direction: SatelliteEnum, calibration_target_direction: &SatelliteEnum) -> MethodResult<SatelliteOperator<SatelliteEnum>, SatelliteMethod> {
    if calibration_target_direction == &previous_direction && &new_direction==calibration_target_direction{
        TaskLists(vec![vec![Method(Switching(satellite, instrument)),
                            Operator(Calibrate(satellite, instrument, *calibration_target_direction)),
                            Operator(TakeImage(satellite, new_direction, instrument, mode))]])
    }else if calibration_target_direction == &previous_direction{
        TaskLists(vec![vec![Method(Switching(satellite, instrument)),
                            Operator(Calibrate(satellite, instrument, *calibration_target_direction)),
                            Operator(TurnTo(satellite, new_direction, *calibration_target_direction)),
                            Operator(TakeImage(satellite, new_direction, instrument, mode))]])

    }else if &new_direction==calibration_target_direction{
        TaskLists(vec![vec![Operator(TurnTo(satellite, *calibration_target_direction, previous_direction)),
                            Method(Switching(satellite, instrument)),
                            Operator(Calibrate(satellite, instrument, *calibration_target_direction)),
                            Operator(TakeImage(satellite, new_direction, instrument, mode))]])

    }else{
        debug!("Taking the dangerous way!");
        TaskLists(vec![vec![Operator(TurnTo(satellite, *calibration_target_direction, previous_direction)),
                            Method(Switching(satellite, instrument)),
                            Operator(Calibrate(satellite, instrument, *calibration_target_direction)),
                            Operator(TurnTo(satellite, new_direction, *calibration_target_direction)),
                            Operator(TakeImage(satellite, new_direction, instrument, mode))]])
    }
}

fn schedule_pointing_with_powered_on_instruments(satellite: SatelliteEnum, instrument: SatelliteEnum, mode: SatelliteEnum, new_direction: SatelliteEnum) -> MethodResult<SatelliteOperator<SatelliteEnum>, SatelliteMethod> {
    TaskLists(vec![vec![Method(Switching(satellite, instrument)),
                        Operator(Calibrate(satellite, instrument, new_direction)),
                        Operator(TakeImage(satellite, new_direction, instrument, mode))]])
}

fn schedule_pointing_with_powered_off_instruments(state: &SatelliteState, satellite: &SatelliteEnum, instrument: SatelliteEnum, mode: SatelliteEnum, new_direction: SatelliteEnum) -> MethodResult<SatelliteOperator<SatelliteEnum>, SatelliteMethod> {
    match find_powered_on_instruments(state, &satellite) {
        Some(instrument_to_power_off) => TaskLists(vec![vec![Operator(SwitchOff(instrument_to_power_off, *satellite)),
                                                             Method(Switching(*satellite, instrument)),
                                                             Operator(Calibrate(*satellite, instrument, new_direction)),
                                                             Operator(TakeImage(*satellite, new_direction, instrument, mode))]]),
        None => TaskLists(vec![vec![Method(Switching(*satellite, instrument)),
                                    Operator(Calibrate(*satellite, instrument, new_direction)),
                                    Operator(TakeImage(*satellite, new_direction, instrument, mode))]])
    }
}

//Given a state, and a Satellite::SatelliteEnum, return an Instrument::Maybe<SatelliteEnum> containing any powered on instruments owned by the satellite.
fn find_powered_on_instruments(state: &SatelliteState, satellite: &SatelliteEnum) -> Option<SatelliteEnum>{
    debug!("Attempting to search the following {:?} ", state.onboard);
    for onboard_instrument_array in state.onboard.get(satellite) { //Get the instrument array for the satellite
        for onboard_instrument in onboard_instrument_array.into_iter() { //Loop over the instruments
            debug!("Seeing if contains: {:?}", onboard_instrument);
            if state.power_on.contains(onboard_instrument) { //Check if the instrument is powered on
                return Some(onboard_instrument.clone());
            }
        }
    }
    return None;
}



fn is_satellite_pointing_in_direction(state: &SatelliteState, satellite: &SatelliteEnum, direction: &SatelliteEnum) -> bool {
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
    return if goal.have_image.keys().eq(&completed_tasks) {
        let pointing_tasks = pointing_needed(state, goal).iter()
            .map(|(sat, dir)| Task::Operator(TurnTo(*sat, *dir, *state.pointing.get(sat).unwrap())))
            .collect();
        TaskLists(vec![pointing_tasks])
    }else if tasks.len()>0{
        TaskLists(tasks)
    }else {
        debug!("ScheduleAll is returning failure");
        debug!("completed_tasks: {:?}", completed_tasks);
        debug!("goals:           {:?}", goal.have_image.keys());
        debug!("goal.pointing:   {:?}", goal.pointing);
        for satellite in state.pointing.iter() {
            debug!("pointing: {:?}", satellite);
        }
        Failure
    };
}
fn pointing_needed(state: &SatelliteState, goal: &SatelliteGoals) -> Vec<(SatelliteEnum,SatelliteEnum)> {
    goal.pointing.iter()
        .map(|(sat, dir)| (*sat, *dir))
        .filter(|(sat, dir)| dir != state.pointing.get(&sat).unwrap())
        .collect()
}

//This method returns a Maybe<Instrument> from a state, and a mode.
fn brute_force_instrument(state: &SatelliteState, mode: &SatelliteEnum)  -> Option<SatelliteEnum> {
    for instrument in state.supports.keys(){
        if state.supports.get(instrument)?.contains(mode){
            return Some(instrument.clone());
        }
    }
    return None;
}

//This method returns a Satellite::Maybe<SatelliteEnum> from a state, an instrument and a mode.
fn brute_force_satellite(state: &SatelliteState, instrument: &SatelliteEnum, mode: &SatelliteEnum) -> Option<SatelliteEnum> {
    for satellites in state.onboard.keys() {
        if state.does_instrument_support_mode(instrument, mode)&& is_onboard(state, satellites.clone(), instrument.clone()){
            return Some(satellites.clone());
        }
    }
    return None;
}

fn is_onboard(state: &SatelliteState, satellite: SatelliteEnum, instrument: SatelliteEnum) -> bool{
    match state.onboard.get(&satellite){
        Some(n) => n.contains(&instrument),
        None => false,
    }
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
    type C = I40F24;

    fn starting_tasks(&self) -> Vec<Task<SatelliteOperator<SatelliteEnum>, SatelliteMethod>> {
        vec![Task::Method(SatelliteMethod::ScheduleAll)]
    }
    fn accepts(&self, state: &Self::S) -> bool {
        for (location,instrument) in self.have_image.iter(){
            let state_instrument = state.have_image.get(location);

            if state_instrument == None || state_instrument != Some(instrument) {
                warn!("We have failed the have_image checker!");
                warn!("Goal have_image: {:?}", self.have_image);
                warn!("Actual have_image: {:?}", state.have_image);
                return false;
            }
        }

        for (satellite, direction) in self.pointing.iter(){
            let state_direction = state.pointing.get(satellite);

            if state_direction == None || state_direction != Some(direction){
                warn!("We have failed the pointing checker!");
                warn!("Goal pointing: {:?}", self.pointing);
                warn!("Actual pointing: {:?}", state.pointing);
                return false;
            }
        }
        debug!("This plan has been accepted by the checker!");
        return true;
    }

    fn distance_from(&self, state: &Self::S) -> Self::C {
        let mut unvisited = 0;
        let mut num_turns = 0;
        for goal_image in self.have_image.keys(){
            //If we haven't visited the image, add it to the unvisited count.
            if state.have_image.get(goal_image)!=self.have_image.get(goal_image){
                unvisited+=1;
            }
        }
        for goal_pointing in self.pointing.keys(){
            if state.pointing.get(goal_pointing)!=self.pointing.get(goal_pointing){
                num_turns+=1;
            }
        }

        return I40F24::from_num(unvisited+num_turns);
    }
}

