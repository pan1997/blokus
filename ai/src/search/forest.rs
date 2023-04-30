use crate::search::RunningAverage;

mod arena_forest;
mod refcnt_forest;

struct ActionInfo {
  action_reward: RunningAverage,
  value_of_next_state: RunningAverage,
  select_count: u32,
  static_policy_score: f32,
}
