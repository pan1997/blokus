mod utils;
pub use utils::{Bounds, RunningAverage};

use crate::MaPomdp;

trait MctsTree<A, O> {
  type Node;
  type NodePtr;
  fn new_root(&self) -> Self::NodePtr;
  fn get_child(&self, n: &Self::Node, o: &O) -> Self::NodePtr;
}

/*
 * State is hidden state
 * Cursor is pointer to node corresponding to current state in search tree
 *     for multi agent search, it's a multi pointer, pointing to the node
 *     corresponding to the current belief state for each agent in their
 *     search tree
 * Action is the
 */
trait MctsProblem<State, Cursor, Action> {
  fn sample_root_state(&self) -> State;
  fn select_transition(&self, state: &State, cursor: &Cursor) -> Action;
  fn advance(&self, state: &mut State, cursor: &mut Cursor, action: Action);
}

struct Search {}

//impl<M, ObservationSeq, Observation, State, Action, const N: usize> Search where M: MaPomdp<ObservationSeq, Observation, State, Action, N> {
impl Search {
  // selects a joint action for state
  fn select<State, Action, O, TNode, const N: usize>(
    &self,
    state: &State,
    // This needs to be mutable, because we want to increment select counts
    nodes: &[&mut TNode; N],
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
    nodes: &mut [&mut TNode; N],
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
      nodes[ix].add_action_sample(&joint_action[ix], transition_result.rewards[ix]);
      accumulated_reward[ix] += transition_result.rewards[ix];
      result[ix] = nodes[ix].get_child(&transition_result.observations[ix]);
    }
    result
  }
}

// A node in the mcts search tree for a single agent
trait TreeNode<A, O> {
  // Default is the nil ptr
  type TreeNodePtr: Default;

  fn select_count(&self) -> u32;
  fn expected_value(&self) -> f32;

  fn increment_select_count(&mut self, action: &A);
  fn add_action_sample(&mut self, action: &A, reward: f32);
  fn get_child(&mut self, obs: &O) -> Self::TreeNodePtr;
}
