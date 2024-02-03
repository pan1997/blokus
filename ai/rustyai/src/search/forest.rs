use std::{collections::BTreeMap, ops::DerefMut};

use crate::search::RunningAverage;

mod arena_forest;
pub mod refcnt_forest;

pub struct ActionInfo {
  pub action_reward: RunningAverage,
  pub value_of_next_state: RunningAverage,
  pub select_count: u32,
  pub static_policy_score: f32,
}

impl Default for ActionInfo {
  fn default() -> Self {
    ActionInfo {
      action_reward: RunningAverage::new(),
      value_of_next_state: RunningAverage::new(),
      select_count: 0,
      static_policy_score: 0.0,
    }
  }
}

impl ActionInfo {
  pub fn action_value(&self) -> f32 {
    self.action_reward.value() + self.value_of_next_state.value()
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

  fn compute_policy(&self) -> Vec<(&A, f32, f32)> {
    let count = self.select_count() as f32;
    self
      .actions()
      .iter()
      .map(|(a, info)| (a, info.select_count as f32 / count, info.action_value()))
      .collect()
  }
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
