//Most of this code is temporarily copied from Dr. Ferrer's Block-World Code until I can get the project up and running

use std::collections::{BTreeSet, BTreeMap};
use anyhop::{Atom, Operator};

//keep this basically
pub fn is_valid<B:Atom>(plan: &Vec<BlockOperator<B>>, start: &BlockState<B>, goal: &BlockGoals<B>) -> bool {
    let mut state = start.clone();
    let preconds_met = plan.iter().all(|step| step.attempt_update(&mut state));
    preconds_met && goal.all_met_in(&state)
}

#[derive(Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub struct BlockGoals<B:Atom> {
    stacks: BTreeMap<B,B>
}

impl <B:Atom> BlockGoals<B> {
    pub fn new(goals: Vec<(B,B)>) -> Self {
        let mut result = BlockGoals {stacks: BTreeMap::new()};
        for (top, bottom) in goals {
            result.stacks.insert(top, bottom);
        }
        result
    }

    pub fn get_pos(&self, block: B) -> BlockPos<B> {
        match self.stacks.get(&block) {
            Some(other) => BlockPos::On(*other),
            None => BlockPos::Table
        }
    }

    pub fn all_met_in(&self, state: &BlockState<B>) -> bool {
        self.stacks.iter()
            .all(|goal| state.get_pos(*goal.0) == BlockPos::On(*goal.1))
    }
}

#[derive(Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub struct BlockState<B:Atom> {
    stacks: BTreeMap<B,B>,
    table: BTreeSet<B>,
    clear: BTreeSet<B>,
    holding: Option<B>
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub enum BlockPos<B:Atom> {
    On(B), Table
}




impl <B:Atom> BlockState<B> {
    pub fn new(blocks: Vec<B>) -> Self {
        let mut state = BlockState {stacks: BTreeMap::new(), table: BTreeSet::new(), clear: BTreeSet::new(), holding: None};
        for block in blocks {
            state.table.insert(block);
            state.clear.insert(block);
        }
        state
    }

    pub fn from(table: Vec<B>, block_piles: Vec<(B,B)>) -> Self {
        let mut all_blocks = table;
        let mut piles: Vec<B> = block_piles.iter().map(|p| p.0).collect();
        all_blocks.append(&mut piles);
        let mut result = BlockState::new(all_blocks);

        for (top, bottom) in block_piles {
            result.stacks.insert(top, bottom);
            result.clear.remove(&bottom);
            result.table.remove(&top);
        }

        result
    }

    pub fn all_blocks(&self) -> Vec<B> {
        let mut result = Vec::new();
        self.stacks.iter().for_each(|entry| result.push(*entry.0));
        self.table.iter().for_each(|b| result.push(*b));
        result
    }

    pub fn get_pos(&self, block: B) -> BlockPos<B> {
        match self.stacks.get(&block) {
            Some(on) => BlockPos::On(*on),
            None => BlockPos::Table
        }
    }

    pub fn get_holding(&self) -> Option<B> {
        self.holding
    }

    pub fn clear(&self, block: B) -> bool {
        self.clear.contains(&block)
    }

    pub fn pick_up(&mut self, block: B) -> bool {
        if self.holding == None && self.table.contains(&block) && self.clear.contains(&block) {
            self.holding = Some(block);
            self.table.remove(&block);
            self.clear.remove(&block);
            true
        } else {false}
    }

    pub fn put_down(&mut self, block: B) -> bool {
        if self.holding == Some(block) {
            self.clear.insert(block);
            self.table.insert(block);
            self.holding = None;
            true
        } else {false}
    }

    pub fn unstack(&mut self, a: B, b: B) -> bool {
        if self.holding == None && self.get_pos(a) == BlockPos::On(b) && self.clear.contains(&a) {
            self.holding = Some(a);
            self.clear.insert(b);
            self.clear.remove(&a);
            self.stacks.remove(&a);
            true
        } else {false}
    }

    pub fn stack(&mut self, a: B, b: B) -> bool {
        if self.holding == Some(a) && self.clear.contains(&b) {
            self.holding = None;
            self.clear.remove(&b);
            self.clear.insert(a);
            self.stacks.insert(a, b);
            true
        } else {false}
    }
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum BlockOperator<B:Atom> {
    PickUp(B), PutDown(B), Stack(B,B), Unstack(B,B)
}

impl <B:Atom> Operator for BlockOperator<B> {
    type S = BlockState<B>;

    fn attempt_update(&self, state: &mut BlockState<B>) -> bool {
        use BlockOperator::*;
        match self {
            PickUp(block) => state.pick_up(*block),
            PutDown(block) => state.put_down(*block),
            Stack(b1, b2) => state.stack(*b1, *b2),
            Unstack(b1, b2) => state.unstack(*b1, *b2)
        }
    }
}


/*
            SATELLITE STUFF!

 */







//These come from the predicates

type Direction = String;

#[derive(Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub enum SatelliteEnum{
    Instrument(String), Satellite(String), Mode(String), Direction(String)
}

#[derive(Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub struct SatelliteState {
    onboard: Vec<SatelliteEnum>,
    supports: Vec<SatelliteEnum>,
    pointing: SatelliteEnum,
    power_avail: Vec<SatelliteEnum>,
    power_on: Vec<SatelliteEnum>,
    calibrated: Vec<SatelliteEnum>,
    have_image: Vec<SatelliteEnum>,
    calibration_target: Vec<SatelliteEnum>,
    data_capacity: BTreeMap<SatelliteEnum, u32>,
    data_stored:  BTreeMap<(SatelliteEnum, SatelliteEnum), u32>, //This is incorrect
    satellite_fuel_capacity: BTreeMap<SatelliteEnum, u32>,
    slew_time: BTreeMap<(SatelliteEnum,SatelliteEnum), u32>,
    fuel_used: u32,
    fuel:u32
}

impl SatelliteState {
    pub fn new(onboard: Vec<SatelliteEnum>, supports: Vec<SatelliteEnum>, pointing: SatelliteEnum, power_avail: Vec<SatelliteEnum>, power_on: Vec<SatelliteEnum>, calibrated: Vec<SatelliteEnum>, have_image: Vec<SatelliteEnum>, calibration_target: Vec<SatelliteEnum>, data_capacity: BTreeMap<SatelliteEnum, u32>, data_stored: BTreeMap<(SatelliteEnum, SatelliteEnum), u32>, satellite_fuel_capacity: BTreeMap<SatelliteEnum, u32>, slew_time: BTreeMap<(SatelliteEnum, SatelliteEnum), u32>, fuel_used: u32, fuel: u32) -> Self {
        SatelliteState { onboard, supports, pointing, power_avail, power_on, calibrated, have_image, calibration_target, data_capacity, data_stored, satellite_fuel_capacity, slew_time, fuel_used, fuel }
    }
}


impl SatelliteState {


    //data_capacity
    pub fn set_data_capacity(&mut self, satellite: SatelliteEnum, capacity: u32){
        self.data_capacity.insert(satellite, capacity);
    }
    //data_stored
    pub fn set_data_stored(&mut self, direction: SatelliteEnum, mode: SatelliteEnum, size: u32){
        self.data_stored.insert((direction,mode),size);
    }
    //slew_time
    pub fn set_slew_time(&mut self, a: SatelliteEnum, b: SatelliteEnum, time: u32){
        self.slew_time.insert((a,b),time);
    }
    //fuel
    pub fn set_satellite_fuel(&mut self, satellite: SatelliteEnum, capacity: u32){
        self.satellite_fuel_capacity.insert(satellite, capacity);
    }
    //fuel-used
    pub fn set_fuel_used(&mut self, fuel:u32){
        self.fuel_used = fuel;
    }
    //action turn_to
    pub fn turn_to(&mut self, new_direction: SatelliteEnum, previous_direction: SatelliteEnum){
        if (self.pointing == previous_direction) && (new_direction != previous_direction) {
             match self.slew_time.get(&(new_direction, previous_direction)){
                Some(x)=>   self.turn_to_helper(&x, &new_direction, &previous_direction), //We have to use a helper here because matches are 1 liners.
                None => println!("Something bad happened!"),
            }
        }
    }

    fn turn_to_helper(&mut self, x:u32, new_direction: &SatelliteEnum, previous_direction: &SatelliteEnum) {
        if self.fuel>=x{
            if self.pointing == new_direction && self.pointing != previous_direction {
                self.slew_time((&new_direction, &previous_direction), self.fuel - 1);
                self.slew_time((&new_direction, &previous_direction), self.fuel_used + 1);
            }
        }
    }
}





