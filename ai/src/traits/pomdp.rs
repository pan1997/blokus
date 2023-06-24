// MultiAgentPartiallyObservableMarkovDecisionProcess
pub trait MaPomdp<ObservationSeq, SampleKey, Observation, State, Action, const N: usize> {
  /*type ObservationSeq;
  type Observation;
  type State;
  type Action;*/

  fn start(&self) -> ObservationSeq;

  // samples a state from an agent's observation sequence
  // SampleKey is the hash of the ObservationSeq. It's used to find the search tree
  // corresponding to the observation seq for agents. Each player
  // gets one key, but the sample key corresponding to the agent in
  // argument is supposed to be unique for all samples, while the
  // key can vary based on the sampled state for other agents.
  //
  // For example, in a game of poker, the sample key for each agent
  // can be the value of hidden cards known to that player. This allows
  // for a better opponent model, as the opponents can play optimally
  // based on the "correct" amount of information known to them
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

/*
pub trait TurnBasedProblem<State, Action> {
  fn current_agent(&self, state: &State) -> usize;
  fn pass_action(&self) -> Action;
}*/


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
