mod bandits;
mod forest;
mod utils;
use std::{collections::BTreeMap, ops::DerefMut};

pub use utils::{Bounds, RunningAverage};

use crate::{search::forest::ActionInfo, BlockMaPomdp, MaMdp, MaPomdp};

struct Search<T> {
  tree_policy: T,
}

impl<T> Search<T> {
  // selects a joint action for state
  fn select_joint_action<M, ObservationSeq, Observation, State, Action, TNode, const N: usize>(
    &self,
    problem: &M,
    state: &State,
    // This needs to be mutable, because we want to increment select counts
    nodes: &[TNode::TreeNodePtr; N],
  ) -> SelectResult<[Action; N]>
  where
    TNode: TreeNode<Action, Observation>,
    M: MaPomdp<ObservationSeq, Observation, State, Action, N>,
    Action: Default,
    T: TreePolicy<M, ObservationSeq, Observation, State, Action, TNode, N>,
  {
    let mut result = [(); N].map(|_| Default::default());
    for ix in 0..N {
      let mut guard = nodes[ix].lock();
      guard.increment_select_count(&result[ix]);

      if guard.first_visit() {
        // This node has never been visisted, so don't select an action
        return SelectResult::Leaf;
      }

      // we assume that the set of legal actions in all states sampled from
      // an observation state are same

      let action_count = guard.actions().len();
      if action_count == 0 {
        return SelectResult::Terminal;
      } else if action_count == 1 {
        // simple optimisation for single legal action for agent
        // todo: get this from the guard
        result[ix] = problem.actions(state, ix).into_iter().next().unwrap();
      } else {
        result[ix] = self.tree_policy.select_action(problem, state, &guard, ix)
      }
    }
    // Each node has at least one action, and all nodes have been visited at least once before
    SelectResult::Action(result)
  }

  fn advance<M, ObservationSeq, Observation, State, Action, TNode, const N: usize>(
    &self,
    problem: &M,
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

  fn advance_block<
    M,
    ObservationSeq,
    Observation,
    State,
    Action,
    TNode,
    const N: usize,
    const B: usize,
  >(
    &self,
    problem: &M,
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

  fn step<M, Observation, State, Action, TNode, const N: usize>(
    &self,
    problem: &M,
    state: &mut State,
    mut current_nodes: &[TNode::TreeNodePtr; N],
  ) where
    M: MaMdp<State, Action, Observation, N>,
    T: TreePolicy<M, State, Observation, State, Action, TNode, N>,
    TNode: TreeNode<Action, Observation>,
    TNode::TreeNodePtr: Clone,
    State: Clone,
    Action: Default,
  {
    let mut trajectory: Vec<[TNode::TreeNodePtr; N]> = vec![]; //Vec<[TNode::TreeNodePtr; N]>;
    let mut rewards = vec![];
    loop {
      trajectory.push(current_nodes.clone());
      match self.select_joint_action(problem, state, current_nodes) {
        SelectResult::Terminal => {}
        SelectResult::Leaf => {}
        SelectResult::Action(joint_action) => {
          let transition_result = problem.transition(state, &joint_action);
          let mut next_node_ptrs = [(); N].map(|_| Default::default());
          for ix in 0..N {
            next_node_ptrs[ix] = current_nodes[ix]
              .lock()
              .get_child(&transition_result.observations[ix]);
            // todo accumulate rewards
          }
          rewards.push(transition_result.rewards);
        }
      }
    }
  }
}

// A node in the mcts search tree for a single agent
trait TreeNode<A, O>: Sized {
  // Default is the nil ptr
  type TreeNodePtr: Default + TreeNodePtr<Self>;
  fn actions(&self) -> &BTreeMap<A, ActionInfo>;

  // returns true if this node hasn't been visited before
  // also marks the node visited so never returns true again
  fn first_visit(&mut self) -> bool;

  fn select_count(&self) -> u32;
  fn expected_value(&self) -> f32;

  fn increment_select_count(&mut self, action: &A);
  fn add_action_sample(&mut self, action: &A, reward: f32);
  fn get_child(&mut self, obs: &O) -> Self::TreeNodePtr;
}

trait TreeNodePtr<TN> {
  type Guard<'a>: DerefMut<Target = TN> + 'a
  where
    Self: 'a;
  fn lock<'a, 'b>(&'b self) -> Self::Guard<'a>
  where
    'b: 'a;
}

trait TreePolicy<M, ObservationSeq, Observation, State, Action, TNode, const N: usize>
where
  M: MaPomdp<ObservationSeq, Observation, State, Action, N>,
  TNode: TreeNode<Action, Observation>,
{
  fn select_action(&self, problem: &M, state: &State, node: &TNode, agent: usize) -> Action;
}

enum SelectResult<A> {
  Terminal, // The state is terminal (at least one agent has no legal moves)
  Leaf,     // Reached a leaf while descending
  Action(A),
}
