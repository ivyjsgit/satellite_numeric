//Most of this code is temporarily copied from Dr. Ferrer's Block-World Code until I can get the project up and running
use super::operators::*;
use anyhop::{Atom, Method, Task, MethodResult, Goal};
use crate::operators::SatelliteOperator::{SwitchOff, SwitchOn, TurnTo, Calibrate, TakeImage};
use std::ptr::null;
use anyhop::MethodResult::{TaskLists, PlanFound};
use crate::methods::SatelliteMethod::{ScheduleOne, ScheduleAll};
use anyhop::Task::Operator;

// pub fn is_done<B:Atom>(b1: B, state: &BlockState<B>, goal: &BlockGoals<B>) -> bool {
//     let pos = state.get_pos(b1);
//     pos == goal.get_pos(b1) && match pos {
//         BlockPos::On(b2) => is_done(b2, state, goal),
//         BlockPos::Table => true
//     }
// }
//
// #[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
// pub enum Status<B:Atom> {
//     Done(B),
//     Inaccessible(B),
//     Move(B,BlockPos<B>),
//     Waiting(B)
// }
//
// impl <B:Atom> Status<B> {
//     pub fn new(b: B, state: &BlockState<B>, goal: &BlockGoals<B>) -> Self {
//         if is_done(b, state, goal) {
//             Status::Done(b)
//         } else if !state.clear(b) {
//             Status::Inaccessible(b)
//         } else {
//             match goal.get_pos(b) {
//                 BlockPos::Table => Status::Move(b, BlockPos::Table),
//                 BlockPos::On(b2) => if is_done(b2, state, goal) && state.clear(b2) {
//                     Status::Move(b, BlockPos::On(b2))
//                 } else {
//                     Status::Waiting(b)
//                 }
//             }
//         }
//     }
// }
//
// #[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
// pub enum BlockMethod<B:Atom> {
//     MoveBlocks,
//     MoveOne(B, BlockPos<B>),
//     Get(B),
//     Put(BlockPos<B>)
// }
//
// impl <B:Atom> Method for BlockMethod<B> {
//     type S = BlockState<B>;
//     type G = BlockGoals<B>;
//     type O = BlockOperator<B>;
//     type T = BlockMethod<B>;
//
//     fn apply(&self, state: &BlockState<B>, goal: &BlockGoals<B>) -> MethodResult<BlockOperator<B>, BlockMethod<B>> {
//         use BlockMethod::*;
//         match self {
//             MoveBlocks => move_blocks(state, goal),
//             MoveOne(block, pos) => move_one(*block, *pos),
//             Get(block) => get(state, *block),
//             Put(pos) => put(state, *pos)
//         }
//     }
// }
// //schedule_all()
// fn move_blocks<B:Atom>(state: &BlockState<B>, goal: &BlockGoals<B>) -> MethodResult<BlockOperator<B>, BlockMethod<B>> {
//     use BlockMethod::*; use MethodResult::*; use Task::*;
//     let status: Vec<Status<B>> = state.all_blocks().iter().map(|b| Status::new(*b, state, goal)).collect();
//     for stat in status.iter() {
//         if let Status::Move(b, pos) = stat {
//             return TaskLists(vec![vec![MethodTag(MoveOne(*b, *pos)), MethodTag(MoveBlocks)]])
//         }
//     }
//
//     let waiting: Vec<Vec<Task<BlockOperator<B>, BlockMethod<B>>>> = status.iter()
//         .filter_map(|s| match s {
//             Status::Waiting(b) => Some(vec![MethodTag(MoveOne(*b, BlockPos::Table)),MethodTag(MoveBlocks)]),
//             _ => None
//         })
//         .collect();
//     if waiting.len() == 0 {PlanFound} else {TaskLists(waiting)}
// }
// //schedule_one
// fn move_one<B:Atom>(block: B, pos: BlockPos<B>) -> MethodResult<BlockOperator<B>, BlockMethod<B>> {
//     use BlockMethod::*; use MethodResult::*; use Task::*;
//     TaskLists(vec![vec![MethodTag(Get(block)), MethodTag(Put(pos))]])
// }
// //Switching
// fn get<'a, B:Atom>(state: &BlockState<B>, block: B) -> MethodResult<BlockOperator<B>, BlockMethod<B>> {
//     use BlockOperator::*; use MethodResult::*; use Task::*; use BlockPos::*;
//     if state.clear(block) {
//         TaskLists(match state.get_pos(block) {
//             Table => vec![vec![Operator(PickUp(block))]],
//             On(block2) => vec![vec![Operator(Unstack(block, block2))]]
//         })
//     } else {
//         Failure
//     }
// }
//
// fn put<'a, B:Atom>(state: &BlockState<B>, pos: BlockPos<B>) -> MethodResult<BlockOperator<B>, BlockMethod<B>> {
//     use BlockOperator::*; use MethodResult::*; use Task::*; use BlockPos::*;
//     if let Some(b) = state.get_holding() {
//         TaskLists(match pos {
//             Table => vec![vec![Operator(PutDown(b))]],
//             On(b2) => vec![vec![Operator(Stack(b, b2))]]
//         })
//     } else {
//         Failure
//     }
// }
//
// impl <B:Atom> MethodTag for BlockMethod<B> {
//     type S = BlockState<B>;
//     type G = BlockGoals<B>;
//     type M = BlockMethod<B>;
//
//     fn candidates(&self, _state: &BlockState<B>, _goal: &BlockGoals<B>) -> Vec<BlockMethod<B>> {
//         vec![*self]
//     }
// }
//
// impl <B:Atom> Goal for BlockGoals<B> {
//     type O = BlockOperator<B>;
//     type T = BlockMethod<B>;
//
//     fn starting_tasks(&self) -> Vec<Task<BlockOperator<B>, BlockMethod<B>>> {
//         vec![Task::MethodTag(BlockMethod::MoveBlocks)]
//     }
// }

/*

Satellite Stuff

 */

#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub enum SatelliteMethod {
    ScheduleAll,
    ScheduleOne(SatelliteEnum),
    //SatelliteState, Satellite, Instrument
    Switching(SatelliteEnum, SatelliteEnum)
}

#[derive(Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub enum SatelliteStatus{
    Done,
    //state, satellite, instrument, mode, new_direction, previous_direction
    NotDone(u32,SatelliteEnum,SatelliteEnum,SatelliteEnum,SatelliteEnum,SatelliteEnum)
}



impl SatelliteStatus{
    pub fn new(identifier: u32, state: SatelliteState, satellite:SatelliteEnum, instrument:SatelliteEnum, mode:SatelliteEnum, new_direction:SatelliteEnum, previous_direction:SatelliteEnum) -> SatelliteStatus{
        if is_satellite_done(state){
            return SatelliteStatus::Done
        }else{
            return SatelliteStatus::NotDone(identifier,satellite,instrument,mode,new_direction,previous_direction)
        }
    }
}

fn is_satellite_done(satellite_state:SatelliteState) -> bool{
    return false;
}

fn switching(state: &SatelliteState, satellite:SatelliteEnum, instrument: SatelliteEnum) -> Vec<SatelliteOperator<SatelliteEnum>>{
    return if !state.power_on.is_empty() && !state.power_on.contains(&instrument) {
        vec![SwitchOff(instrument, satellite), SwitchOn(instrument, satellite)]
    } else if state.power_on.is_empty() {
        vec![SwitchOn(instrument, satellite)]
    } else {
        vec![]
    }
}

fn schedule_one(state: &SatelliteState, satellite: SatelliteEnum, instrument: SatelliteEnum, mode: SatelliteEnum, new_direction: SatelliteEnum, previous_direction: SatelliteEnum) -> MethodResult<SatelliteOperator<SatelliteEnum>, SatelliteMethod> {
    use SatelliteMethod::*; use MethodResult::*; use Task::*;
    TaskLists(vec![vec![Operator(TurnTo(satellite, new_direction, previous_direction)),
                        Method(Switching(satellite, instrument)),
                        Operator(Calibrate(satellite,instrument,new_direction)),
                        Operator(TakeImage(satellite,new_direction,instrument,mode))]])
}

fn schedule_all(state:SatelliteState, goal: SatelliteGoals) -> MethodResult<SatelliteOperator<SatelliteEnum>, SatelliteMethod>{
    let mut tasks : Vec<Vec<Task<SatelliteOperator<SatelliteEnum>,SatelliteMethod>>> = vec![vec![]];
    let mut completed_tasks: Vec<SatelliteEnum> = vec![];

    for goalImage in goal.have_image.keys(){
        if !(state.have_image.get(goalImage) == goal.have_image.get(goalImage)){
            let goal_image_clone = goalImage.clone();
            tasks.push(vec![TaskList[ScheduleOne(goal_clone),ScheduleAll]])
        }else{
            let image_clone = goalImage.clone();
            completed_tasks.push(image_clone);
        }
    }

    if goal.have_image.keys().eq(&completed_tasks){
        return PlanFound
    }else {
        return TaskLists(tasks);
    }
}
