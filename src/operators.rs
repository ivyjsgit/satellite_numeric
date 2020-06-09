//Most of this code is temporarily copied from Dr. Ferrer's Block-World Code until I can get the project up and running
use strum_macros::*;

use std::collections::{BTreeSet, BTreeMap};
use anyhop::{Atom, Operator};

//keep this basically
pub fn is_valid<B: Atom>(plan: &Vec<BlockOperator<B>>, start: &BlockState<B>, goal: &BlockGoals<B>) -> bool {
    let mut state = start.clone();
    let preconds_met = plan.iter().all(|step| step.attempt_update(&mut state));
    preconds_met && goal.all_met_in(&state)
}

#[derive(Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub struct BlockGoals<B: Atom> {
    stacks: BTreeMap<B, B>
}

impl<B: Atom> BlockGoals<B> {
    pub fn new(goals: Vec<(B, B)>) -> Self {
        let mut result = BlockGoals { stacks: BTreeMap::new() };
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
pub struct BlockState<B: Atom> {
    stacks: BTreeMap<B, B>,
    table: BTreeSet<B>,
    clear: BTreeSet<B>,
    holding: Option<B>,
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub enum BlockPos<B: Atom> {
    On(B),
    Table,
}


impl<B: Atom> BlockState<B> {
    pub fn new(blocks: Vec<B>) -> Self {
        let mut state = BlockState { stacks: BTreeMap::new(), table: BTreeSet::new(), clear: BTreeSet::new(), holding: None };
        for block in blocks {
            state.table.insert(block);
            state.clear.insert(block);
        }
        state
    }

    pub fn from(table: Vec<B>, block_piles: Vec<(B, B)>) -> Self {
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
        } else { false }
    }

    pub fn put_down(&mut self, block: B) -> bool {
        if self.holding == Some(block) {
            self.clear.insert(block);
            self.table.insert(block);
            self.holding = None;
            true
        } else { false }
    }

    pub fn unstack(&mut self, a: B, b: B) -> bool {
        if self.holding == None && self.get_pos(a) == BlockPos::On(b) && self.clear.contains(&a) {
            self.holding = Some(a);
            self.clear.insert(b);
            self.clear.remove(&a);
            self.stacks.remove(&a);
            true
        } else { false }
    }

    pub fn stack(&mut self, a: B, b: B) -> bool {
        if self.holding == Some(a) && self.clear.contains(&b) {
            self.holding = None;
            self.clear.remove(&b);
            self.clear.insert(a);
            self.stacks.insert(a, b);
            true
        } else { false }
    }
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum BlockOperator<B: Atom> {
    PickUp(B),
    PutDown(B),
    Stack(B, B),
    Unstack(B, B),
}

impl<B: Atom> Operator for BlockOperator<B> {
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


#[derive(Clone, PartialOrd, PartialEq, Ord, Eq, Debug, Display)]
pub enum SatelliteEnum {
    Instrument(String),
    Satellite(String),
    Mode(String),
    Direction(String),
}

#[derive(Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub struct SatelliteState {
    //map satellite -> vec<instrument>
    onboard: BTreeMap<SatelliteEnum,Vec<SatelliteEnum>>,
    supports: BTreeMap<SatelliteEnum, SatelliteEnum>,
    //map satellite -> direction
    pointing: BTreeMap<SatelliteEnum,SatelliteEnum>,
    power_avail: bool,
    power_on: Vec<SatelliteEnum>,
    calibrated: Vec<SatelliteEnum>,
    have_image: BTreeMap<SatelliteEnum, SatelliteEnum>,
    calibration_target: BTreeMap<SatelliteEnum, SatelliteEnum>,
    //map satelite -> u32
    data_capacity: BTreeMap<SatelliteEnum,u32>,
    //needs to be u32
    data_stored: u32,
    satellite_fuel_capacity: BTreeMap<SatelliteEnum, u32>,
    slew_time: BTreeMap<(SatelliteEnum, SatelliteEnum), u32>,
    fuel_used: u32,
    fuel: u32,
}



impl SatelliteState {
    //data_capacity
    pub fn set_data_capacity(&mut self, satellite : SatelliteEnum, capacity: u32) {
        self.data_capacity.insert(satellite,capacity);
    }
    //data_stored
    pub fn set_data_stored(&mut self, size: u32) {
        self.data_stored = size;
    }
    //slew_time
    pub fn set_slew_time(&mut self, a: &SatelliteEnum, b: &SatelliteEnum, time: u32) {
        // GJF: *** The ownership issues are resolved by cloning a and b at this stage.
        self.slew_time.insert((a.clone(), b.clone()), time);
    }
    //fuel
    pub fn set_satellite_fuel(&mut self, satellite: &SatelliteEnum, capacity: u32) {
        // GJF: *** Same idea here.
        self.satellite_fuel_capacity.insert(satellite.clone(), capacity);
    }
    //fuel-used
    pub fn set_fuel_used(&mut self, fuel: u32) {
        self.fuel_used = fuel;
    }
    //action turn_to
    // GJF: *** Because SatelliteEnum is not a Copy type, we need to either lend it to
    //  turn_to_helper() or clone it when passing it to turn_to_helper(). I have reworked
    //  things so that we are lending it.
    pub fn turn_to(&mut self, satellite : &SatelliteEnum, new_direction: &SatelliteEnum, previous_direction: &SatelliteEnum) {
        if (self.pointing_helper(satellite,previous_direction)) && (new_direction != previous_direction) {
            // GJF: *** I had to clone them here to create the key for the lookup.
            let key = (new_direction.clone(), previous_direction.clone());
            match self.slew_time.get(&key) {
                Some(x) => self.turn_to_helper(satellite, *x, new_direction, previous_direction), //We have to use a helper here because matches are 1 liners.
                None => eprintln!("Error while turning: The following key lookup failed in the slew_time table: {} {}", &key.0, &key.1),
            }
        }
    }

    fn pointing_helper(&mut self, satellite: &SatelliteEnum, direction: &SatelliteEnum)-> bool{
        return match self.pointing.get(satellite) {
            Some(x) => x == direction, //If we have the correct instrument selected, we need to make sure that it is selected at the right direction.
            None => false, //If the lookup fails, the if statement should fail.
        }
    }


    fn turn_to_helper(&mut self, satellite: &SatelliteEnum, x: u32, new_direction: &SatelliteEnum, previous_direction: &SatelliteEnum) {
        if self.fuel >= x {
            if self.pointing_helper(satellite,new_direction) && !self.pointing_helper(satellite,previous_direction) {
                self.set_slew_time(new_direction, previous_direction, self.fuel - 1);
                self.set_slew_time(new_direction, previous_direction, self.fuel_used + 1);
            }
        }
    }
    fn switch_on(&mut self, instrument : &SatelliteEnum, satellite : &SatelliteEnum){
        //precondition
        if self.onboard.get(satellite).unwrap().contains(instrument) && self.power_avail{
            //effect
            let instrument_clone = instrument.clone();

            self.power_on.push(instrument_clone);

            //See if the instrument is calibrated and remove the calibration
            //https://stackoverflow.com/a/37482592 Why doesn't Rust have indexOf????
            if self.calibrated.contains(instrument){
                let index = self.calibrated.iter().position(|s| s==instrument).unwrap();
                self.calibrated.remove(index);
            }
            self.power_avail = false;
        }
    }
    pub fn switch_off(&mut self, instrument: &SatelliteEnum, satellite : &SatelliteEnum){
        if self.onboard.get(satellite).unwrap().contains(instrument) && self.power_on.contains(instrument){
            //Remove instrument from the power on
            if self.power_on.contains(instrument){
                let index = self.power_on.iter().position(|s| s==instrument).unwrap();
                self.power_on.remove(index);
            }
            self.power_avail = true;
        }
    }

    pub fn calibrate(&mut self, satellite : &SatelliteEnum,instrument: &SatelliteEnum, direction: &SatelliteEnum){
        if self.onboard.get(satellite).unwrap().contains(instrument) && self.calibrate_helper(&instrument, &direction) && self.pointing_helper(satellite, direction) && self.power_on.contains(instrument){
            let instrument_clone = instrument.clone();
            self.calibrated.push(instrument_clone);
        }
    }
    fn calibrate_helper(&mut self, instrument: &SatelliteEnum, direction: &SatelliteEnum)-> bool{
        return match self.calibration_target.get(instrument) {
            Some(x) => x == direction, //If we have the correct instrument selected, we need to make sure that it is selected at the right direction.
            None => false, //If the lookup fails, the if statement should fail.
        }
    }
    pub fn take_image(&mut self, satellite : &SatelliteEnum, direction: SatelliteEnum, instrument: &SatelliteEnum, mode: &SatelliteEnum){
        if self.calibrated.contains(instrument) && self.onboard.get(satellite).unwrap().contains(instrument) && self.supports_helper(instrument, mode) && self.power_on.contains(instrument) && (self.pointing_helper(satellite,&direction)) && (self.power_on.contains(instrument)) && (self.data_capacity.get(satellite).unwrap() >= &self.data_used_helper(&direction, &mode)){

            //reduce the capacity
            let subtracted_capacity = self.data_capacity.get(satellite).unwrap() - self.data_used_helper(&direction, &mode);
            self.data_capacity.insert(satellite.clone(), subtracted_capacity);
            //insert the image
            self.have_image.insert(direction.clone(), mode.clone());

            //update the capacity
            let old_capacity = self.data_used_helper(&direction, mode);
            let mut pair = (direction.clone(), mode.clone());
            self.data_stored = old_capacity //add old_capacity

        }
    }
    fn supports_helper(&mut self, instrument: &SatelliteEnum, mode: &SatelliteEnum) -> bool{
        return match self.supports.get(&instrument) {
            Some(x) => x == mode, //If we have the correct instrument selected, we need to make sure that it is selected at the right direction.
            None => false, //If the lookup fails, the if statement should fail.
        }

    }
    fn data_used_helper(&mut self, direction : &SatelliteEnum, mode: &SatelliteEnum) -> u32 {
        let pair = &(direction.clone(), mode.clone());
        return match self.data_stored.get(pair){
            Some(x) => *x,
            None => 0,
        }
    }
}





