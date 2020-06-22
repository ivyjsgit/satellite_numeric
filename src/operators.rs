//Most of this code is temporarily copied from Dr. Ferrer's Block-World Code until I can get the project up and running
use strum_macros::*;

use std::collections::{BTreeSet, BTreeMap};
use anyhop::{Atom, Operator};
use crate::methods::SatelliteStatus;

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


#[derive(Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Debug, Display)]
pub enum SatelliteEnum {
    //We will have to have a lookup table that goes from usize -> String for these.
    //This is just here because you can't have the copy attribute because Strings aren't copyable.
    Instrument(usize),
    Satellite(usize),
    Mode(usize),
    Direction(usize),
}

#[derive(Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub struct SatelliteState {
    //map satellite -> vec<instrument>
    pub onboard: BTreeMap<SatelliteEnum, Vec<SatelliteEnum>>,
    //instrument -> mode
    pub supports: BTreeMap<SatelliteEnum, SatelliteEnum>,
    //map satellite -> direction
    pub pointing: BTreeMap<SatelliteEnum, SatelliteEnum>,
    pub power_avail: bool,
    pub power_on: Vec<SatelliteEnum>,
    pub calibrated: Vec<SatelliteEnum>,
    pub have_image: BTreeMap<SatelliteEnum, SatelliteEnum>,
    pub calibration_target: BTreeMap<SatelliteEnum, SatelliteEnum>,
    //map satelite -> u32
    pub data_capacity: BTreeMap<SatelliteEnum, u32>,
    //needs to be u32
    pub total_data_stored: u32,
    pub satellite_data_stored: BTreeMap<SatelliteEnum, u32>,
    pub satellite_fuel_capacity: BTreeMap<SatelliteEnum, u32>,
    pub slew_time: BTreeMap<(SatelliteEnum, SatelliteEnum), u32>,
    pub fuel_used: u32,
    pub fuel: u32,
    pub status: SatelliteStatus,
}


impl SatelliteState {
    //data_capacity
    pub fn set_data_capacity(&mut self, satellite: SatelliteEnum, capacity: u32) {
        self.data_capacity.insert(satellite, capacity);
    }
    //data_stored
    pub fn set_data_stored(&mut self, size: u32) {
        self.total_data_stored = size;
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
    pub fn turn_to(&mut self, satellite: &SatelliteEnum, new_direction: &SatelliteEnum, previous_direction: &SatelliteEnum) -> bool {
        if (self.pointing_helper(satellite, previous_direction)) && (new_direction != previous_direction) {
            // GJF: *** I had to clone them here to create the key for the lookup.
            let key = (new_direction.clone(), previous_direction.clone());
            // GJF: *** Separate out the get to avoid the borrow conflict:
            let slew_time = match self.slew_time.get(&key) {
                Some(x) => *x,
                None => panic!(format!("Error while turning: The following key lookup failed in the slew_time table: {} {}", &key.0, &key.1))
            };
            self.turn_to_helper(satellite, slew_time, new_direction, previous_direction);
            return true;
        } else {
            return false;
        }
    }

    fn pointing_helper(&mut self, satellite: &SatelliteEnum, direction: &SatelliteEnum) -> bool {
        return match self.pointing.get(satellite) {
            Some(x) => x == direction, //If we have the correct instrument selected, we need to make sure that it is selected at the right direction.
            None => false, //If the lookup fails, the if statement should fail.
        };
    }


    fn turn_to_helper(&mut self, satellite: &SatelliteEnum, x: u32, new_direction: &SatelliteEnum, previous_direction: &SatelliteEnum) {
        if self.fuel >= x {
            if self.pointing_helper(satellite, new_direction) && !self.pointing_helper(satellite, previous_direction) {
                self.set_slew_time(new_direction, previous_direction, self.fuel - 1);
                self.set_slew_time(new_direction, previous_direction, self.fuel_used + 1);
            }
        }
    }
    fn switch_on(&mut self, instrument: &SatelliteEnum, satellite: &SatelliteEnum) -> bool {
        //precondition
        if self.onboard.get(satellite).unwrap().contains(instrument) && self.power_avail {
            //effect
            let instrument_clone = instrument.clone();

            self.power_on.push(instrument_clone);

            //See if the instrument is calibrated and remove the calibration
            //https://stackoverflow.com/a/37482592 Why doesn't Rust have indexOf????
            if self.calibrated.contains(instrument) {
                let index = self.calibrated.iter().position(|s| s == instrument).unwrap();
                self.calibrated.remove(index);
            }
            self.power_avail = false;
            return true;
        } else {
            return false;
        }
    }
    pub fn switch_off(&mut self, instrument: &SatelliteEnum, satellite: &SatelliteEnum) -> bool {
        if self.onboard.get(satellite).unwrap().contains(instrument) && self.power_on.contains(instrument) {
            //Remove instrument from the power on
            if self.power_on.contains(instrument) {
                let index = self.power_on.iter().position(|s| s == instrument).unwrap();
                self.power_on.remove(index);
            }
            self.power_avail = true;
            return true;
        } else {
            return false;
        }
    }

    pub fn calibrate(&mut self, satellite: &SatelliteEnum, instrument: &SatelliteEnum, direction: &SatelliteEnum) -> bool {
        if self.onboard.get(satellite).unwrap().contains(instrument) && self.calibrate_helper(&instrument, &direction) && self.pointing_helper(satellite, direction) && self.power_on.contains(instrument) {
            let instrument_clone = instrument.clone();
            self.calibrated.push(instrument_clone);
            return true;
        } else {
            return false;
        }
    }
    fn calibrate_helper(&mut self, instrument: &SatelliteEnum, direction: &SatelliteEnum) -> bool {
        return match self.calibration_target.get(instrument) {
            Some(x) => x == direction, //If we have the correct instrument selected, we need to make sure that it is selected at the right direction.
            None => false, //If the lookup fails, the if statement should fail.
        };
    }
    pub fn take_image(&mut self, satellite: &SatelliteEnum, direction: SatelliteEnum, instrument: &SatelliteEnum, mode: &SatelliteEnum) -> bool {
        // GJF: *** To solve the ownership problem, we store the capacity in a local
        //      variable where it is copied out of self. We then use that local variable
        //      wherever we need it.
        let satellite_capacity = *(self.data_capacity.get(satellite).unwrap());
        if self.calibrated.contains(instrument) &&
            self.onboard.get(satellite).unwrap().contains(instrument) &&
            self.supports_helper(instrument, mode) &&
            self.power_on.contains(instrument) &&
            self.pointing_helper(satellite, &direction) &&
            satellite_capacity >= self.get_satellite_data_used(&satellite) {

            //reduce the capacity
            let subtracted_capacity = satellite_capacity - self.get_satellite_data_used(&satellite);
            self.data_capacity.insert(satellite.clone(), subtracted_capacity);
            //insert the image
            self.have_image.insert(direction.clone(), mode.clone());
            //update the capacity
            let old_capacity = self.get_satellite_data_used(&satellite);
            self.total_data_stored = old_capacity; //add old_capacity
            return true;
        } else {
            return false;
        }
    }
    pub fn supports_helper(&self, instrument: &SatelliteEnum, mode: &SatelliteEnum) -> bool {
        return match self.supports.get(&instrument) {
            Some(x) => x == mode, //If we have the correct instrument selected, we need to make sure that it is selected at the right direction.
            None => false, //If the lookup fails, the if statement should fail.
        };
    }
    fn get_satellite_data_used(&mut self, satellite: &SatelliteEnum) -> u32 {
        return match self.satellite_data_stored.get(satellite) {
            Some(x) => *x,
            None => 0,
        };
    }
}

#[derive(Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub struct SatelliteGoals {
    //Have_image maps from location -> instrument
    pub have_image: BTreeMap<SatelliteEnum, SatelliteEnum>,
    pub fuel_used: u32,
}

impl SatelliteGoals {
    pub fn new(have_image: BTreeMap<SatelliteEnum, SatelliteEnum>, fuel_used: u32) -> Self {
        SatelliteGoals { have_image, fuel_used }
    }
    pub fn all_met_in(&self, state: &SatelliteState) -> bool {
        for location in self.have_image.keys() {
            let goal_instrument = self.have_image.get(location).unwrap();
            match state.have_image.get(location) {
                Some(instrument) => if instrument != goal_instrument { return false; },
                None => return false //If the location isn't found.
            }
        }

        return true; //If we have gotten through the entire list, return true
    }
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum SatelliteOperator<SatelliteEnum> {
    TurnTo(SatelliteEnum, SatelliteEnum, SatelliteEnum),
    SwitchOn(SatelliteEnum, SatelliteEnum),
    SwitchOff(SatelliteEnum, SatelliteEnum),
    Calibrate(SatelliteEnum, SatelliteEnum, SatelliteEnum),
    TakeImage(SatelliteEnum, SatelliteEnum, SatelliteEnum, SatelliteEnum),
}

impl Operator for SatelliteOperator<SatelliteEnum> {
    type S = SatelliteState;

    fn attempt_update(&self, state: &mut SatelliteState) -> bool {
        use SatelliteOperator::*;
        match self {
            TurnTo(satellite, new_direction, previous_direction) => state.turn_to(&*satellite, &*new_direction, &*previous_direction),
            SwitchOn(instrument, satellite) => state.switch_on(&*instrument, &*satellite),
            SwitchOff(instrument, satellite) => state.switch_off(&*instrument, &*satellite),
            Calibrate(satellite, instrument, direction) => state.calibrate(&*satellite, &*instrument, &*direction),
            TakeImage(satellite, direction, instrument, mode) => state.take_image(&*satellite, *direction, &*instrument, &*mode)
        }
    }
}

pub fn is_satellite_valid(plan: &Vec<SatelliteOperator<SatelliteEnum>>, start: &SatelliteState, goal: &SatelliteGoals) -> bool {
    let mut state = start.clone();

    let preconds_met = plan.iter().all(|step| step.attempt_update(&mut state));
    preconds_met && goal.all_met_in(&state)
}