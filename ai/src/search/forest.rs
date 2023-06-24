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
