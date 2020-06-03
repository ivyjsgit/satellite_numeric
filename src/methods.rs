<<<<<<< HEAD
//Most of this code is temporarily copied from Dr. Ferrer's Block-World Code until I can get the project up and running

=======
>>>>>>> master
use super::operators::*;
use anyhop::{Atom, Method, MethodTag, Task, MethodResult, Goal};

pub fn is_done<B:Atom>(b1: B, state: &BlockState<B>, goal: &BlockGoals<B>) -> bool {
    let pos = state.get_pos(b1);
    pos == goal.get_pos(b1) && match pos {
        BlockPos::On(b2) => is_done(b2, state, goal),
        BlockPos::Table => true
    }
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub enum Status<B:Atom> {
    Done(B),
    Inaccessible(B),
    Move(B,BlockPos<B>),
    Waiting(B)
}

impl <B:Atom> Status<B> {
    pub fn new(b: B, state: &BlockState<B>, goal: &BlockGoals<B>) -> Self {
        if is_done(b, state, goal) {
            Status::Done(b)
        } else if !state.clear(b) {
            Status::Inaccessible(b)
        } else {
            match goal.get_pos(b) {
                BlockPos::Table => Status::Move(b, BlockPos::Table),
                BlockPos::On(b2) => if is_done(b2, state, goal) && state.clear(b2) {
                    Status::Move(b, BlockPos::On(b2))
                } else {
                    Status::Waiting(b)
                }
            }
        }
    }
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Debug)]
pub enum BlockMethod<B:Atom> {
    MoveBlocks,
    MoveOne(B, BlockPos<B>),
    Get(B),
    Put(BlockPos<B>)
}

impl <B:Atom> Method for BlockMethod<B> {
    type S = BlockState<B>;
    type G = BlockGoals<B>;
    type O = BlockOperator<B>;
    type T = BlockMethod<B>;

    fn apply(&self, state: &BlockState<B>, goal: &BlockGoals<B>) -> MethodResult<BlockOperator<B>, BlockMethod<B>> {
        use BlockMethod::*;
        match self {
            MoveBlocks => move_blocks(state, goal),
            MoveOne(block, pos) => move_one(*block, *pos),
            Get(block) => get(state, *block),
            Put(pos) => put(state, *pos)
        }
    }
}

fn move_blocks<B:Atom>(state: &BlockState<B>, goal: &BlockGoals<B>) -> MethodResult<BlockOperator<B>, BlockMethod<B>> {
    use BlockMethod::*; use MethodResult::*; use Task::*;
    let status: Vec<Status<B>> = state.all_blocks().iter().map(|b| Status::new(*b, state, goal)).collect();
    for stat in status.iter() {
        if let Status::Move(b, pos) = stat {
            return TaskLists(vec![vec![MethodTag(MoveOne(*b, *pos)), MethodTag(MoveBlocks)]])
        }
    }

    let waiting: Vec<Vec<Task<BlockOperator<B>, BlockMethod<B>>>> = status.iter()
        .filter_map(|s| match s {
            Status::Waiting(b) => Some(vec![MethodTag(MoveOne(*b, BlockPos::Table)),MethodTag(MoveBlocks)]),
            _ => None
        })
        .collect();
    if waiting.len() == 0 {PlanFound} else {TaskLists(waiting)}
}

fn move_one<B:Atom>(block: B, pos: BlockPos<B>) -> MethodResult<BlockOperator<B>, BlockMethod<B>> {
    use BlockMethod::*; use MethodResult::*; use Task::*;
    TaskLists(vec![vec![MethodTag(Get(block)), MethodTag(Put(pos))]])
}

fn get<'a, B:Atom>(state: &BlockState<B>, block: B) -> MethodResult<BlockOperator<B>, BlockMethod<B>> {
    use BlockOperator::*; use MethodResult::*; use Task::*; use BlockPos::*;
    if state.clear(block) {
        TaskLists(match state.get_pos(block) {
            Table => vec![vec![Operator(PickUp(block))]],
            On(block2) => vec![vec![Operator(Unstack(block, block2))]]
        })
    } else {
        Failure
    }
}

fn put<'a, B:Atom>(state: &BlockState<B>, pos: BlockPos<B>) -> MethodResult<BlockOperator<B>, BlockMethod<B>> {
    use BlockOperator::*; use MethodResult::*; use Task::*; use BlockPos::*;
    if let Some(b) = state.get_holding() {
        TaskLists(match pos {
            Table => vec![vec![Operator(PutDown(b))]],
            On(b2) => vec![vec![Operator(Stack(b, b2))]]
        })
    } else {
        Failure
    }
}

impl <B:Atom> MethodTag for BlockMethod<B> {
    type S = BlockState<B>;
    type G = BlockGoals<B>;
    type M = BlockMethod<B>;

    fn candidates(&self, _state: &BlockState<B>, _goal: &BlockGoals<B>) -> Vec<BlockMethod<B>> {
        vec![*self]
    }
}

impl <B:Atom> Goal for BlockGoals<B> {
    type O = BlockOperator<B>;
    type T = BlockMethod<B>;

    fn starting_tasks(&self) -> Vec<Task<BlockOperator<B>, BlockMethod<B>>> {
        vec![Task::MethodTag(BlockMethod::MoveBlocks)]
    }
}