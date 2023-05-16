// MultiAgentPartiallyObservableMarkovDecisionProcess
pub trait MaPomdp<ObservationSeq, SampleKey, Observation, State, Action, const N: usize> {
  /*type ObservationSeq;
  type Observation;
  type State;
  type Action;*/

  fn start(&self) -> ObservationSeq;

  // samples a state from an agent's observation sequence
  fn sample(
    &self,
    observation_seq: &ObservationSeq,
    agent: usize,
  ) -> SampleResult<State, SampleKey, N>;

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

pub trait BlockMaPomdp<ObservationSeq, SampleKey, Observation, State, Action, const N: usize>:
  MaPomdp<ObservationSeq, SampleKey, Observation, State, Action, N>
{
  fn transition_block<const B: usize>(
    &self,
    states: &mut [State; B],
    joint_actions: &[[Action; N]; B],
  ) -> [TranstitionResult<Observation, N>; B];

  fn sample_block<const B: usize>(
    observation_seq: &ObservationSeq,
    agent: usize,
  ) -> [SampleResult<State, SampleKey, N>; B];
}

pub struct TranstitionResult<Observation, const N: usize> {
  pub observations: [Observation; N],
  pub rewards: [f32; N],
}

pub struct SampleResult<State, SampleKey, const N: usize> {
  pub state: State,
  pub sample_keys: [SampleKey; N],
}
