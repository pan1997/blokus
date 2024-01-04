use std::{collections::BTreeMap, fs::File};

use rand::{distributions::WeightedIndex, prelude::*};
use rstest::*;

use crate::{
  search::{eval::ZeroEval, forest::refcnt_forest::Node, render::save, Random, Search, Uct},
  traits::pomdp::{SampleResult, TranstitionResult},
  MaMdp, MaPomdp,
};

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

impl MaPomdp<ObservationSeq, (), Observation, State, Action, 1> for StaticMpomdp {
  fn actions(&self, state: &State, _agent: usize) -> Vec<Action> {
    assert_eq!(_agent, 0, "Invalid Agent: {_agent}");
    self.states[*state].outgoing.keys().map(|x| *x).collect()
  }

  // obs contains all the details that can be known to the agent, including the action taken
  fn append(&self, _observation_seq: &mut ObservationSeq, agent: usize, _obs: Observation) {
    assert_eq!(agent, 0, "Invalid Agent: {agent}");
    //observation_seq.append(obs)
    unimplemented!()
  }

  fn sample(&self, observation_seq: &ObservationSeq, _agent: usize) -> SampleResult<State, (), 1> {
    let dist = WeightedIndex::new(observation_seq).unwrap();
    SampleResult {
      state: dist.sample(&mut rand::thread_rng()),
      sample_keys: [()],
    }
  }

  fn start(&self, agent: usize) -> ObservationSeq {
    if agent != 0 {
      panic!("Invalid agent")
    }
    self.start.clone()
  }

  fn transition(
    &self,
    state: &mut State,
    joint_action: &[Action; 1],
  ) -> TranstitionResult<Observation, 1> {
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

struct StaticMdp {
  start: State,
  states: Vec<StateDef>,
}

impl StaticMdp {
  fn new() -> Self {
    StaticMdp {
      start: 0,
      states: vec![],
    }
  }

  fn add_state(&mut self) -> State {
    self.states.push(StateDef {
      outgoing: BTreeMap::new(),
    });
    self.states.len() - 1
  }

  fn add_transition(
    &mut self,
    src: State,
    action: Action,
    obs: Observation,
    dest: State,
    reward: f32,
    weight: f32,
  ) {
    let src_state_def = &mut self.states[src];
    if !src_state_def.outgoing.contains_key(&action) {
      src_state_def.outgoing.insert(
        action,
        ActionDef {
          weights: vec![],
          rewards: vec![],
          observations: vec![],
          next_state: vec![],
        },
      );
    }
    src_state_def.outgoing.get_mut(&action).map(|ad| {
      ad.weights.push(weight);
      ad.observations.push(obs);
      ad.next_state.push(dest);
      ad.rewards.push(reward);
    });
  }
}

impl MaMdp<State, Action, Observation, 1> for StaticMdp {
  fn actions(&self, state: &State, _agent: usize) -> Vec<Action> {
    debug_assert!(_agent == 0, "Invalid agent: {_agent}");
    self.states[*state].outgoing.keys().map(|x| *x).collect()
  }
  fn initial_state(&self) -> State {
    self.start
  }
  fn transition(
    &self,
    state: &mut State,
    joint_action: &[Action; 1],
  ) -> TranstitionResult<Observation, 1> {
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

#[fixture]
fn problem1() -> StaticMdp {
  let mut result = StaticMdp::new();
  let s0 = result.add_state();
  let s1 = result.add_state();
  let s2 = result.add_state();

  let a0 = 0;
  let a1 = 1;

  result.add_transition(s0, a0, s0, s0, 0.0, 0.5);
  result.add_transition(s0, a0, s2, s2, 0.0, 0.5);
  result.add_transition(s0, a1, s2, s2, 0.0, 1.0);

  result.add_transition(s1, a0, s1, s1, 0.0, 0.1);
  result.add_transition(s1, a0, s0, s0, 5.0, 0.7);
  result.add_transition(s1, a0, s2, s2, 0.0, 0.2);
  result.add_transition(s1, a1, s1, s1, 0.0, 0.95);
  result.add_transition(s1, a1, s2, s2, 0.0, 0.05);

  result.add_transition(s2, a0, s0, s0, 0.0, 0.4);
  result.add_transition(s2, a0, s2, s2, 0.0, 0.6);
  result.add_transition(s2, a1, s0, s0, -1.0, 0.3);
  result.add_transition(s2, a1, s1, s1, 0.0, 0.3);
  result.add_transition(s2, a1, s2, s2, 0.0, 0.4);
  result
}

#[rstest]
fn test_problem1_random_policy(problem1: StaticMdp) {
  let s = Search::new(Random, ZeroEval);
  let state = 0;
  let trees = [Node::new(); 1];
  for iter in 0..10000 {
    let n = [trees[0].clone(); 1];
    let x = s.step_mdp(&problem1, &state, n);
    println!("{iter}: rewards: {x:?}");
  }
  save(trees, File::create("test.t1.dot").unwrap(), 0, 3)
}

#[rstest]
fn test_problem1_uct_policy(problem1: StaticMdp) {
  let s = Search::new(Uct(4.8), ZeroEval);
  let state = 0;
  let trees = [Node::new(); 1];
  for iter in 0..10000 {
    let n = [trees[0].clone(); 1];
    let x = s.step_mdp(&problem1, &state, n);
    //println!("{iter}: rewards: {x:?}");
  }
  save(trees, File::create("test.t2.dot").unwrap(), 0, 3)
}
