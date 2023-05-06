use std::collections::BTreeMap;

use rand::{distributions::WeightedIndex, prelude::*};

use crate::{traits::pomdp::TranstitionResult, MaPomdp};

type State = usize;
type Observation = usize;
// prob distribution of states
type ObservationSeq = Vec<f32>;
type Action = usize;

struct StateDef {
  outgoing: BTreeMap<Action, ActionDef>,
}

struct ActionDef {
  weights: Vec<f32>,
  rewards: Vec<f32>,
  observations: Vec<Observation>,
  next_state: Vec<State>,
}

struct StaticMpomdp {
  start: ObservationSeq,
  states: Vec<StateDef>,
}

impl MaPomdp<ObservationSeq, Observation, State, Action, 1> for StaticMpomdp {
  fn actions(&self, state: &State, agent: usize) -> Vec<Action> {
    self.states[*state].outgoing.keys().map(|x| *x).collect()
  }
  fn append(&self, observation_seq: &mut ObservationSeq, agent: usize, obs: Observation) {
    assert_eq!(agent, 0, "Invalid Agent: {agent}");
    //observation_seq.append(obs)
    unimplemented!()
  }
  fn sample(&self, observation_seq: &ObservationSeq, agent: usize) -> State {
    let dist = WeightedIndex::new(observation_seq).unwrap();
    dist.sample(&mut rand::thread_rng())
  }
  fn start(&self) -> ObservationSeq {
    self.start.clone()
  }
  fn transition(
    &self,
    state: &mut State,
    joint_action: &[Action; 1],
  ) -> crate::traits::pomdp::TranstitionResult<Observation, 1> {
    let action_def = &self.states[*state].outgoing[&joint_action[0]];
    let dist = WeightedIndex::new(&action_def.weights).unwrap();
    let ix = dist.sample(&mut rand::thread_rng());
    *state = action_def.next_state[ix];
    TranstitionResult {
      rewards: [action_def.rewards[ix]],
      observations: [action_def.observations[ix]],
    }
  }
}

#[test]
fn t1() {}
