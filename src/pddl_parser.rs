use sexpy::*;
use crate::operators::{BlockState, BlockGoals};
use std::{io,fs};
use std::collections::HashMap;

pub fn make_block_problem_from(pddl_file: &str) -> io::Result<(BlockState<usize>, BlockGoals<usize>)> {
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

impl Define {
    pub fn init_and_goal(&self) -> (BlockState<usize>, BlockGoals<usize>) {
        let mut objects = HashMap::new();
        for object in self.objects.objs.iter() {
            objects.insert(String::from(object), objects.len());
        }
        let mut table = Vec::new();
        let mut stacks = Vec::new();
        for pred in self.init.predicates.iter() {
            if pred.predicate_type == "ontable" {
                table.push(*objects.get(pred.predicate_args[0].as_str()).unwrap());
            } else if pred.predicate_type == "on" {
                stacks.push(decode_on(&pred, &objects));
            }
        }

        let mut goals = Vec::new();
        for goal in self.goal.and.goals.iter() {
            goals.push(decode_on(&goal, &objects));
        }

        (BlockState::from(table, stacks), BlockGoals::new(goals))
    }
}

fn decode_on(p: &Predicate, objects: &HashMap<String,usize>) -> (usize, usize) {
    let top = obj_get(p, objects, 0);
    let bottom = obj_get(p, objects, 1);
    (top, bottom)
}

fn obj_get(p: &Predicate, objects: &HashMap<String,usize>, i: usize) -> usize {
    *objects.get(p.predicate_args[i].as_str()).unwrap()
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
struct Predicate {
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