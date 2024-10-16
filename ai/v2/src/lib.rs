mod forest;
mod node;
mod search;
mod util;

trait MarkovDecisionProcess: Discounted + Searchable<Self::State, Self::Action, Self::R> {
  type State;
  type Action;
  type R;
  fn start_state(&self) -> Self::State;
}

trait Pomdp:
  Discounted + Searchable<Self::HiddenState, Self::Action, (Self::Observation, Self::R)>
{
  type Observation;
  type ObservationSeq;
  type HiddenState;
  type Action;
  type R;
  fn start(&self) -> Self::ObservationSeq;
  fn update_observation_seq(&self, obs_seq: &mut Self::ObservationSeq, obs: &Self::Observation);
  fn sample_hidden_state(&self, obs_seq: &Self::ObservationSeq) -> Self::HiddenState;
}

trait MultiAgentMDP<const N: usize>:
  Discounted + Searchable<Self::State, [Self::Action; N], Self::R>
{
  type State;
  type Action;
  type R;

  fn start_state(&self) -> Self::State;
  fn get_agent_actions(&self, state: &Self::State, agent_index: usize) -> Vec<Self::Action>;
}

trait Searchable<N, E, T> {
  // Terminal states have no outgoing edges
  fn outgoing_edges(&self, n: &N) -> Vec<E>;

  // transitions the state n by applying the action e. Returns the transition result.
  fn transition(&self, n: &mut N, e: &E) -> T;

  fn is_terminal(&self, n: &N) -> bool {
    self.outgoing_edges(n).is_empty()
  }
}

trait Discounted {
  fn discount(&self) -> f32 {
    1.0
  }
}

pub struct MctsConfig {}

#[cfg(test)]
mod tests {
  use super::*;
}
