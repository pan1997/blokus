pub mod bandits;
pub mod forest;
pub mod render;
mod utils;
use std::{collections::BTreeMap, fmt::Debug, ops::DerefMut};

pub use bandits::{Random, Uct};
pub use utils::{Bounds, RunningAverage};

use crate::{search::forest::ActionInfo, BlockMaPomdp, MaMdp, MaPomdp};

// TODO: score bounds
pub struct Search<T> {
  pub tree_policy: T,
}

impl<T> Search<T> {
  pub fn new(tree_policy: T) -> Self {
    Search { tree_policy }
  }

  // selects a joint action for state
  // We assume that all agents select their actions independently using the
  // tree_policy
  fn select_joint_action<
    M,
    ObservationSeq,
    SampleKey,
    Observation,
    State,
    Action,
    TNodePtr,
    const N: usize,
  >(
    &self,
    problem: &M,
    state: &State,
    nodes: &[TNodePtr; N],
  ) -> SelectResult<[Action; N]>
  where
    TNodePtr: TreeNodePtr<Action, Observation>,
    M: MaPomdp<ObservationSeq, SampleKey, Observation, State, Action, N>,
    // TODO: remove this default requirement
    Action: Default,
    T: TreePolicy<M, ObservationSeq, SampleKey, Observation, State, Action, TNodePtr::TreeNode, N>,
  {
    let mut result = [(); N].map(|_| Default::default());
    for ix in 0..N {
      let mut guard = nodes[ix].lock();
      if guard.first_visit() {
        // This node has never been visisted, so don't select an action
        drop(guard);
        // todo: mark other agent's visit too
        for jx in 0..N {
          nodes[jx].lock().first_visit();
        }
        return SelectResult::Leaf;
      }

      // we assume that the set of legal actions in all states sampled from
      // an observation state are same
      let action_count = guard.actions().len();
      if action_count == 0 {
        // This node has been expanded (first_visit was false), and still has
        // no legal moves
        return SelectResult::Terminal;
      } else if action_count == 1 {
        // simple optimisation for single legal action for agent
        // todo: get this from the guard
        result[ix] = problem.actions(state, ix).into_iter().next().unwrap();
      } else {
        result[ix] = self.tree_policy.select_action(problem, state, &guard, ix)
      }
      guard.increment_select_count(&result[ix]);
    }
    // Each node has at least one action, and all nodes have been visited at least once before
    SelectResult::Action(result)
  }

  fn advance<M, ObservationSeq, SampleKey, Observation, State, Action, TNodePtr, const N: usize>(
    &self,
    problem: &M,
    state: &mut State,
    nodes: &[TNodePtr; N],
    joint_action: &[Action; N],
  ) -> ([TNodePtr; N], [f32; N])
  where
    TNodePtr: TreeNodePtr<Action, Observation> + Default,
    M: MaPomdp<ObservationSeq, SampleKey, Observation, State, Action, N>,
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
    SampleKey,
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
    joint_actions: [[Action; N]; B],
  ) -> [[TNodePtr; N]; B]
  where
    TNodePtr: TreeNodePtr<Action, Observation> + Default,
    M: BlockMaPomdp<ObservationSeq, SampleKey, Observation, State, Action, N>,
  {
    let transition_result = problem.transition_block(states, &joint_actions);

    let mut result = [(); B].map(|_| [(); N].map(|_| Default::default()));
    // add action rewards to the node
    // accumulate rewards for trajectory
    // get child pointers for each agent
    for b in 0..B {
      for ix in 0..N {
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
    for ix in 0..N {
      trajectory[len - 1][ix]
        .lock()
        .value_mut()
        .add_sample(terminal_value[ix], 1);
    }

    // -2 because the last entry in trajectory doesn't have any actions or rewards
    let mut depth = len.wrapping_sub(2);

    // depth is unsigned, so we are done after we process 0 and depth wraps around
    while depth < actions.len() {
      for ix in 0..N {
        let mut guard = trajectory[depth][ix].lock();
        // update reward
        let ai = guard.actions_mut().get_mut(&actions[depth][ix]).unwrap();
        ai.action_reward.add_sample(rewards[depth][ix], 1);
        ai.value_of_next_state.add_sample(terminal_value[ix], 1);

        // for next iter of depth
        terminal_value[ix] += rewards[depth][ix];
        // This node's value should include this action's reward
        guard.value_mut().add_sample(terminal_value[ix], 1);
      }
      depth = depth.wrapping_sub(1);
    }
    terminal_value
  }

  fn expand<M, ObservationSeq, SampleKey, Observation, State, Action, TNodePtr, const N: usize>(
    &self,
    problem: &M,
    state: &State,
    nodes: &[TNodePtr; N],
  ) where
    M: MaPomdp<ObservationSeq, SampleKey, Observation, State, Action, N>,
    TNodePtr: TreeNodePtr<Action, Observation>,
    Action: Ord,
    Observation: Ord,
  {
    for ix in 0..N {
      let actions = problem.actions(state, ix);
      let len = actions.len();
      //println!("returned actions len: {len}");
      // TODO: static policy
      // TODO: rollout/eval

      // actions returned here can be empty. We don't check for state termination
      // when we find a leaf
      // debug_assert!(actions.len() != 0, "Empty actions");
      let mut guard = nodes[ix].lock();
      // This should be empty, as we need to ensure that this node has never
      // been expanded before
      let len = guard.actions().len();
      //println!("guard action len: {len}");
      debug_assert!(guard.actions().len() == 0);
      let am = guard.actions_mut();
      for action in actions {
        am.insert(action, ActionInfo::default());
      }
    }
  }

  fn step_internal<
    M,
    ObservationSeq,
    SampleKey,
    Observation,
    State,
    Action,
    TNodePtr,
    const N: usize,
  >(
    &self,
    problem: &M,
    state: &mut State,
    mut current_nodes: [TNodePtr; N],
  ) -> [f32; N]
  where
    M: MaPomdp<ObservationSeq, SampleKey, Observation, State, Action, N>,
    T: TreePolicy<M, ObservationSeq, SampleKey, Observation, State, Action, TNodePtr::TreeNode, N>,
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
          //println!("Terminal");
          trajectory.push(current_nodes);
          return self.propogate(&trajectory, &actions, &rewards, [0.0; N]);
        }
        SelectResult::Leaf => {
          //println!("Leaf");
          self.expand(problem, state, &current_nodes);
          trajectory.push(current_nodes);
          // TODO: find
          let terminal_value = [0.0; N];
          return self.propogate(&trajectory, &actions, &rewards, terminal_value);
        }
        SelectResult::Action(joint_action) => {
          //println!("Advancing");
          let (_nodes, _rewards) = self.advance(problem, state, &current_nodes, &joint_action);
          actions.push(joint_action);
          rewards.push(_rewards);
          trajectory.push(current_nodes);
          current_nodes = _nodes;
        }
      }
    }
  }

  pub fn step_mdp<M, Observation, State, Action, TNodePtr, const N: usize>(
    &self,
    problem: &M,
    state: &State,
    mut current_nodes: [TNodePtr; N],
  ) -> [f32; N]
  where
    M: MaMdp<State, Action, Observation, N>,
    T: TreePolicy<M, State, (), Observation, State, Action, TNodePtr::TreeNode, N>,
    TNodePtr: TreeNodePtr<Action, Observation> + Clone + Default,
    State: Clone,
    Action: Default + Ord,
    Observation: Ord,
  {
    self.step_internal(problem, &mut state.clone(), current_nodes)
  }

  pub fn step_single_agent<M, ObservationSeq, Observation, State, Action, TNodePtr>(
    &self,
    problem: &M,
    obs_seq: &ObservationSeq,
    mut current_nodes: [TNodePtr; 1],
  ) -> [f32; 1]
  where
    M: MaPomdp<ObservationSeq, (), Observation, State, Action, 1>,
    T: TreePolicy<M, ObservationSeq, (), Observation, State, Action, TNodePtr::TreeNode, 1>,
    TNodePtr: TreeNodePtr<Action, Observation> + Clone + Default,
    State: Clone,
    Action: Default + Ord,
    Observation: Ord,
  {
    let mut m = problem.sample(obs_seq, 0);
    self.step_internal(problem, &mut m.state, current_nodes)
  }
}

// A node in the mcts search tree for a single agent
pub trait TreeNode<A, O>: Sized {
  // Default is the nil ptr
  type TreeNodePtr: Default;
  fn actions(&self) -> &BTreeMap<A, ActionInfo>;
  fn children(&self) -> &BTreeMap<O, Self::TreeNodePtr>;
  fn actions_mut(&mut self) -> &mut BTreeMap<A, ActionInfo>;

  // returns true if this node hasn't been visited before
  // also marks the node visited so never returns true again
  fn first_visit(&mut self) -> bool;

  fn select_count(&self) -> u32;
  fn value(&self) -> &RunningAverage;
  fn value_mut(&mut self) -> &mut RunningAverage;

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

pub trait TreePolicy<
  M,
  ObservationSeq,
  SampleKey,
  Observation,
  State,
  Action,
  TNode,
  const N: usize,
> where
  M: MaPomdp<ObservationSeq, SampleKey, Observation, State, Action, N>,
  TNode: TreeNode<Action, Observation>,
{
  fn select_action(&self, problem: &M, state: &State, node: &TNode, agent: usize) -> Action;
}

enum SelectResult<A> {
  Terminal, // The state is terminal (at least one agent has no legal moves)
  Leaf,     // Reached a leaf while descending
  Action(A),
}

// TODO
pub trait StaticPolicy {}
