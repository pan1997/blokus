mod utils;
mod arena_forest;
mod refcnt_forest;
use std::ops::DerefMut;

pub use utils::{Bounds, RunningAverage};

use crate::MaPomdp;
use crate::BlockMaPomdp;

struct Search {}

//impl<M, ObservationSeq, Observation, State, Action, const N: usize> Search where M: MaPomdp<ObservationSeq, Observation, State, Action, N> {
impl Search {
  // selects a joint action for state
  fn select<State, Action, O, TNode, const N: usize>(
    &self,
    state: &State,
    // This needs to be mutable, because we want to increment select counts
    nodes: &[TNode::TreeNodePtr; N],
  ) -> [Action; N]
  where
    TNode: TreeNode<Action, O>,
  {
    // let result = [; N]
    // for ix in 0..N {
    //  nodes[ix].increment_select_count(result[ix]);
    //}
    // return result
    unimplemented!()
  }

  fn advance<M, State, Action, Observation, ObservationSeq, TNode, const N: usize>(
    &self,
    problem: M,
    state: &mut State,
    nodes: &mut [TNode::TreeNodePtr; N],
    accumulated_reward: &mut [f32; N],
    joint_action: [Action; N],
  ) -> [TNode::TreeNodePtr; N]
  where
    TNode: TreeNode<Action, Observation>,
    M: MaPomdp<ObservationSeq, Observation, State, Action, N>,
  {
    let transition_result = problem.transition(state, &joint_action);

    let mut result = [(); N].map(|_| Default::default());
    // add action rewards to the node
    // accumulate rewards for trajectory
    // get child pointers for each agent
    for ix in 0..N {
      accumulated_reward[ix] += transition_result.rewards[ix];
      let mut guard = nodes[ix].lock();
      guard.add_action_sample(&joint_action[ix], transition_result.rewards[ix]);
      result[ix] = guard.get_child(&transition_result.observations[ix]);
    }
    result
  }

  fn advance_block<M, State, Action, Observation, ObservationSeq, TNode, const N: usize, const B: usize>(
    &self,
    problem: M,
    states: &mut [State; B],
    nodes: &[[TNode::TreeNodePtr; N]; B],
    accumulated_reward: &mut [[f32; N]; B],
    joint_actions: [[Action; N]; B],
  ) -> [[TNode::TreeNodePtr; N]; B]
  where
    TNode: TreeNode<Action, Observation>,
    M: BlockMaPomdp<ObservationSeq, Observation, State, Action, N>,
  {
    let transition_result = problem.transition_block(states, &joint_actions);

    let mut result = [(); B].map(|_| [(); N].map(|_| Default::default()));
    // add action rewards to the node
    // accumulate rewards for trajectory
    // get child pointers for each agent
    for b in 0..B {
      for ix in 0..N {
        accumulated_reward[b][ix] += transition_result[b].rewards[ix];
        let mut guard = nodes[b][ix].lock();
        guard.add_action_sample(&joint_actions[b][ix], transition_result[b].rewards[ix]);
        result[b][ix] = guard.get_child(&transition_result[b].observations[ix]);
      }
    }
    result
  }
}

// A node in the mcts search tree for a single agent
trait TreeNode<A, O>: Sized {
  // Default is the nil ptr
  type TreeNodePtr: Default + TreeNodePtr<Self>;

  fn select_count(&self) -> u32;
  fn expected_value(&self) -> f32;

  fn increment_select_count(&mut self, action: &A);
  fn add_action_sample(&mut self, action: &A, reward: f32);
  fn get_child(&mut self, obs: &O) -> Self::TreeNodePtr;
}

trait TreeNodePtr<TN> {
  type Guard: DerefMut<Target = TN>;
  fn lock(&self) -> Self::Guard;
}
