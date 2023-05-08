mod bandits;
pub mod forest;
mod utils;
use std::{collections::BTreeMap, ops::DerefMut};

pub use bandits::{Random, Uct};
pub use utils::{Bounds, RunningAverage};

use crate::{search::forest::ActionInfo, BlockMaPomdp, MaMdp, MaPomdp};

pub struct Search<T> {
  tree_policy: T,
}

impl<T> Search<T> {
  pub fn new(tree_policy: T) -> Self {
    Search { tree_policy }
  }

  // selects a joint action for state
  fn select_joint_action<M, ObservationSeq, Observation, State, Action, TNodePtr, const N: usize>(
    &self,
    problem: &M,
    state: &State,
    // This needs to be mutable, because we want to increment select counts
    nodes: &[TNodePtr; N],
  ) -> SelectResult<[Action; N]>
  where
    TNodePtr: TreeNodePtr<Action, Observation>,
    M: MaPomdp<ObservationSeq, Observation, State, Action, N>,
    Action: Default,
    T: TreePolicy<M, ObservationSeq, Observation, State, Action, TNodePtr::TreeNode, N>,
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

  fn advance<M, ObservationSeq, Observation, State, Action, TNodePtr, const N: usize>(
    &self,
    problem: &M,
    state: &mut State,
    nodes: &[TNodePtr; N],
    joint_action: &[Action; N],
  ) -> ([TNodePtr; N], [f32; N])
  where
    TNodePtr: TreeNodePtr<Action, Observation> + Default,
    M: MaPomdp<ObservationSeq, Observation, State, Action, N>,
  {
    let transition_result = problem.transition(state, joint_action);

    let mut result = [(); N].map(|_| Default::default());
    // add action rewards to the node
    // accumulate rewards for trajectory
    // get child pointers for each agent
    for ix in 0..N {
      let mut guard = nodes[ix].lock();
      guard.add_action_sample(&joint_action[ix], transition_result.rewards[ix]);
      result[ix] = guard.get_child(&transition_result.observations[ix]);
    }
    (result, transition_result.rewards)
  }

  fn advance_block<
    M,
    ObservationSeq,
    Observation,
    State,
    Action,
    TNodePtr,
    const N: usize,
    const B: usize,
  >(
    &self,
    problem: &M,
    states: &mut [State; B],
    nodes: &[[TNodePtr; N]; B],
    accumulated_reward: &mut [[f32; N]; B],
    joint_actions: [[Action; N]; B],
  ) -> [[TNodePtr; N]; B]
  where
    TNodePtr: TreeNodePtr<Action, Observation> + Default,
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

  fn propogate<Observation, Action, TNodePtr, const N: usize>(
    &self,
    trajectory: &[[TNodePtr; N]],
    actions: &[[Action; N]],
    rewards: &[[f32; N]],
    mut terminal_value: [f32; N],
  ) -> [f32; N]
  where
    TNodePtr: TreeNodePtr<Action, Observation>,
    Action: Ord,
    Observation: Ord,
  {
    debug_assert!(trajectory.len() == actions.len() + 1);
    debug_assert!(actions.len() == rewards.len());

    let len = trajectory.len();
    // TODO: update node value

    // -2 because the last entry in trajectory doesn't have any actions or rewards
    let mut depth = len.wrapping_sub(2);
    while depth < actions.len() {
      for ix in 0..N {
        let mut guard = trajectory[depth][ix].lock();
        // update reward
        let ai = guard.actions_mut().get_mut(&actions[depth][ix]).unwrap();
        ai.action_reward.add_sample(rewards[depth][ix], 1);
        ai.value_of_next_state.add_sample(terminal_value[ix], 1);

        // for next iter of depth
        terminal_value[ix] += rewards[depth][ix];
      }
      depth = depth.wrapping_sub(1);
    }
    terminal_value
  }

  fn expand<M, ObservationSeq, Observation, State, Action, TNodePtr, const N: usize>(
    &self,
    problem: &M,
    state: &State,
    nodes: &[TNodePtr; N],
  ) where
    M: MaPomdp<ObservationSeq, Observation, State, Action, N>,
    TNodePtr: TreeNodePtr<Action, Observation>,
    Action: Ord,
    Observation: Ord,
  {
    for ix in 0..N {
      let actions = problem.actions(state, ix);
      debug_assert!(actions.len() != 0, "Empty actions");
      let mut guard = nodes[ix].lock();
      debug_assert!(guard.actions().len() == 0);
      let mut am = guard.actions_mut();
      for action in actions {
        am.insert(action, ActionInfo::default());
      }
    }
  }

  pub fn step<M, Observation, State, Action, TNodePtr, const N: usize>(
    &self,
    problem: &M,
    state: &mut State,
    mut current_nodes: [TNodePtr; N],
  ) -> [f32; N]
  where
    M: MaMdp<State, Action, Observation, N>,
    T: TreePolicy<M, State, Observation, State, Action, TNodePtr::TreeNode, N>,
    TNodePtr: TreeNodePtr<Action, Observation> + Clone + Default,
    State: Clone,
    Action: Default + Ord,
    Observation: Ord,
  {
    let mut trajectory: Vec<[TNodePtr; N]> = vec![]; //Vec<[TNode::TreeNodePtr; N]>;
    let mut actions = vec![];
    let mut rewards = vec![];
    loop {
      match self.select_joint_action(problem, state, &current_nodes) {
        SelectResult::Terminal => {
          trajectory.push(current_nodes);
          return self.propogate(&trajectory, &actions, &rewards, [0.0; N]);
        }
        SelectResult::Leaf => {
          self.expand(problem, state, &current_nodes);
          trajectory.push(current_nodes);
          // TODO: find
          let terminal_value = [0.0; N];
          return self.propogate(&trajectory, &actions, &rewards, terminal_value);
        }
        SelectResult::Action(joint_action) => {
          let (_nodes, _rewards) = self.advance(problem, state, &current_nodes, &joint_action);
          actions.push(joint_action);
          rewards.push(_rewards);
          trajectory.push(current_nodes);
          current_nodes = _nodes;
        }
      }
    }
  }
}

// A node in the mcts search tree for a single agent
pub trait TreeNode<A, O>: Sized {
  // Default is the nil ptr
  type TreeNodePtr: Default;
  fn actions(&self) -> &BTreeMap<A, ActionInfo>;
  fn actions_mut(&mut self) -> &mut BTreeMap<A, ActionInfo>;

  // returns true if this node hasn't been visited before
  // also marks the node visited so never returns true again
  fn first_visit(&mut self) -> bool;

  fn select_count(&self) -> u32;
  fn expected_value(&self) -> f32;

  fn increment_select_count(&mut self, action: &A);
  fn add_action_sample(&mut self, action: &A, reward: f32);
  fn get_child(&mut self, obs: &O) -> Self::TreeNodePtr;
}

pub trait TreeNodePtr<A, O> {
  type TreeNode: TreeNode<A, O, TreeNodePtr = Self>;
  type Guard<'a>: DerefMut<Target = Self::TreeNode> + 'a
  where
    Self: 'a;
  fn lock<'a, 'b>(&'b self) -> Self::Guard<'a>
  where
    'b: 'a;
}

pub trait TreePolicy<M, ObservationSeq, Observation, State, Action, TNode, const N: usize>
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
