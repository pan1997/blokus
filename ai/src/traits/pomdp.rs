// MultiAgentPartiallyObservableMarkovDecisionProcess
pub trait MaPomdp<ObservationSeq, Observation, State, Action, const N: usize> {
  /*type ObservationSeq;
  type Observation;
  type State;
  type Action;*/

  fn start(&self) -> ObservationSeq;

  // samples a state from an agent's observation sequence
  fn sample(&self, observation_seq: &ObservationSeq, agent: usize) -> State;

  // Returns the set of actions for agent in state
  // every non terminal state needs to have an action for
  // every agent. Create a dummy action to represent pass
  // A state is terminal, if any agent's action list is empty
  fn actions(&self, state: &State, agent: usize) -> Vec<Action>;

  fn transition(
    &self,
    state: &mut State,
    joint_action: &[Action; N],
  ) -> TranstitionResult<Observation, N>;

  fn append(&self, observation_seq: &mut ObservationSeq, agent: usize, obs: Observation);
}

pub struct TranstitionResult<Observation, const N: usize> {
  pub observations: [Observation; N],
  pub rewards: [f32; N],
}
