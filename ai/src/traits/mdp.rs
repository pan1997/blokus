use super::pomdp::{SampleResult, TranstitionResult};
use crate::MaPomdp;

// fully observable
pub trait MaMdp<State, Action, Observation, const N: usize> {
  fn start(&self) -> State;
  fn actions(&self, state: &State, agent: usize) -> Vec<Action>;
  fn transition(
    &self,
    state: &mut State,
    joint_action: &[Action; N],
  ) -> TranstitionResult<Observation, N>;
}

impl<M, State, Action, Observation, const N: usize>
  MaPomdp<State, (), Observation, State, Action, N> for M
where
  M: MaMdp<State, Action, Observation, N>,
  State: Clone,
{
  fn actions(&self, state: &State, agent: usize) -> Vec<Action> {
    self.actions(state, agent)
  }

  fn start(&self) -> State {
    self.start()
  }

  fn sample(&self, observation_seq: &State, agent: usize) -> SampleResult<State, (), N> {
    SampleResult {
      state: observation_seq.clone(),
      sample_keys: [(); N],
    }
  }

  fn transition(
    &self,
    state: &mut State,
    joint_action: &[Action; N],
  ) -> TranstitionResult<Observation, N> {
    self.transition(state, joint_action)
  }

  fn append(&self, observation_seq: &mut State, agent: usize, obs: Observation) {
    /*
    let mut joint_action = [(); N].map(|_| Default::default());
    joint_action[agent] = obs;
    self.transition(observation_seq, &joint_action);*/
    unimplemented!()
  }
}
