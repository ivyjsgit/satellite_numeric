use std::collections::{BTreeMap, BTreeSet};

use anyhop::{Atom, Operator};
use strum_macros::*;
use fixed::types::I40F24;


use crate::methods::SatelliteStatus;
use crate::methods::SatelliteStatus::{Done, NotDone};

#[derive(Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Debug, Display)]
pub enum SatelliteEnum {
    //We will have to have a lookup table that goes from usize -> String for these.
    //This is just here because you can't have the copy attribute because Strings aren't copyable.
    Instrument(u32),
    Satellite(u32),
    Mode(u32),
    Direction(u32),
}

#[derive(Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub struct SatelliteState {
    //map satellite -> vec<instrument>
    pub onboard: BTreeMap<SatelliteEnum, Vec<SatelliteEnum>>,
    //instrument -> vec<modes>
    pub supports: BTreeMap<SatelliteEnum, Vec<SatelliteEnum>>,
    //map satellite -> direction
    pub pointing: BTreeMap<SatelliteEnum, SatelliteEnum>,
    pub power_avail: bool,
    //instrument
    pub power_on: Vec<SatelliteEnum>,
    //instrument
    pub calibrated: Vec<SatelliteEnum>,
    //direction -> mode
    pub have_image: BTreeMap<SatelliteEnum, SatelliteEnum>,
    //instrument -> direction
    pub calibration_target: BTreeMap<SatelliteEnum, SatelliteEnum>,
    //map satelite -> u32
    pub data_capacity: BTreeMap<SatelliteEnum, u32>,
    //needs to be u32
    pub total_data_stored: u32,
    pub satellite_data_stored: BTreeMap<(SatelliteEnum, SatelliteEnum), u32>,
    pub slew_time: BTreeMap<(SatelliteEnum, SatelliteEnum), I40F24>,
    pub fuel_used: u32,
    //satellite -> fuel
    pub fuel: BTreeMap<SatelliteEnum, u32>,
    pub status: SatelliteStatus,
}

impl SatelliteState {
    pub fn new(onboard: BTreeMap<SatelliteEnum, Vec<SatelliteEnum>>, supports: BTreeMap<SatelliteEnum, Vec<SatelliteEnum>>, pointing: BTreeMap<SatelliteEnum, SatelliteEnum>, power_avail: bool, power_on: Vec<SatelliteEnum>, calibrated: Vec<SatelliteEnum>, have_image: BTreeMap<SatelliteEnum, SatelliteEnum>, calibration_target: BTreeMap<SatelliteEnum, SatelliteEnum>, data_capacity: BTreeMap<SatelliteEnum, u32>, total_data_stored: u32, satellite_data_stored: BTreeMap<(SatelliteEnum, SatelliteEnum), u32>, slew_time: BTreeMap<(SatelliteEnum, SatelliteEnum), I40F24>, fuel_used: u32, fuel: BTreeMap<SatelliteEnum, u32>) -> Self {
        SatelliteState { onboard, supports, pointing, power_avail, power_on, calibrated, have_image, calibration_target, data_capacity, total_data_stored, satellite_data_stored, slew_time, fuel_used, fuel, status: (Done) }
    }
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
    pub fn set_slew_time(&mut self, a: &SatelliteEnum, b: &SatelliteEnum, time: I40F24) {
        // GJF: *** The ownership issues are resolved by cloning a and b at this stage.
        self.slew_time.insert((a.clone(), b.clone()), time);
    }
    //fuel
    pub fn set_satellite_fuel(&mut self, satellite: &SatelliteEnum, capacity: u32) {
        // GJF: *** Same idea here.
        self.fuel.insert(satellite.clone(), capacity);
    }
    //fuel-used
    pub fn set_fuel_used(&mut self, fuel: u32) {
        self.fuel_used = fuel;
    }
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
            self.turn_to_helper(satellite, (slew_time.to_num::<u32>()), new_direction, previous_direction);
            return true;
        }else {
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


        if self.fuel.get(satellite).unwrap() >= &x {
            if self.pointing_helper(satellite, new_direction) && !self.pointing_helper(satellite, previous_direction) {
                self.set_slew_time(new_direction, previous_direction, I40F24::from_num(self.fuel.get(satellite).unwrap() - 1));
                self.set_slew_time(new_direction, previous_direction, I40F24::from_num(self.fuel_used + 1));
                self.pointing.insert(satellite.clone(),new_direction.clone());
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
            self.does_instrument_support_mode(instrument, mode) &&
            self.power_on.contains(instrument) &&
            self.pointing_helper(satellite, &direction) &&
            satellite_capacity >= self.get_satellite_data_used(&direction, mode) {

            //reduce the capacity
            let subtracted_capacity = satellite_capacity - self.get_satellite_data_used(&direction, mode);
            self.data_capacity.insert(satellite.clone(), subtracted_capacity);
            //insert the image
            self.have_image.insert(direction.clone(), mode.clone());
            //update the capacity
            let old_capacity = self.get_satellite_data_used(&direction, mode);
            self.total_data_stored = old_capacity; //add old_capacity
            return true;
        } else {
            return false;
        }
    }
    pub fn does_instrument_support_mode(&self, instrument: &SatelliteEnum, mode: &SatelliteEnum) -> bool {
        return match self.supports.get(&instrument) {
            Some(x) => x.contains(mode), //If we have the correct instrument selected, we need to make sure that it is selected at the right direction.
            None => false, //If the lookup fails, the if statement should fail.
        };
    }
    fn get_satellite_data_used(&mut self, direction: &SatelliteEnum, mode: &SatelliteEnum) -> u32 {
        return match self.satellite_data_stored.get(&(*direction, *mode)) {
            Some(x) => *x,
            None => 0,
        };
    }

    // pub fn new(onboard: BTreeMap<SatelliteEnum, Vec<SatelliteEnum>>, supports: BTreeMap<SatelliteEnum, Vec<SatelliteEnum>>, pointing: BTreeMap<SatelliteEnum, SatelliteEnum>, power_avail: bool, power_on: Vec<SatelliteEnum>, calibrated: Vec<SatelliteEnum>, have_image: BTreeMap<SatelliteEnum, SatelliteEnum>, calibration_target: BTreeMap<SatelliteEnum, SatelliteEnum>, data_capacity: BTreeMap<SatelliteEnum, u32>, total_data_stored: u32, satellite_data_stored: BTreeMap<(SatelliteEnum, SatelliteEnum), u32>, satellite_fuel_capacity: BTreeMap<SatelliteEnum, u32>, slew_time: BTreeMap<(SatelliteEnum, SatelliteEnum), I40F24>, fuel_used: u32) -> Self {
    //     SatelliteState { onboard, supports, pointing, power_avail, power_on, calibrated, have_image, calibration_target, data_capacity, total_data_stored, satellite_data_stored, satellite_fuel_capacity, slew_time, fuel_used, fuel: (0), status: (Done) }
    // }


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