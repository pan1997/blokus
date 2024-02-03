use std::marker::PhantomData;

use rand::seq::SliceRandom;

use crate::MaPomdp;

pub trait BaseEval<M, State, Action, const N: usize> {
  // todo: maybe add nodeptr/actions
  fn evaluate<'a>(&self, problem: &M, state: &'a mut State) -> EvaluationResult<'a, Action, N>;
}

pub struct EvaluationResult<'a, A, const N: usize> {
  pub values: [f32; N],
  pub policies: [Vec<(&'a A, f32)>; N],
}

pub struct ZeroEval;
impl<M, S, A, const N: usize> BaseEval<M, S, A, N> for ZeroEval {
  fn evaluate<'a>(&self, _problem: &M, _state: &'a mut S) -> EvaluationResult<'a, A, N> {
    EvaluationResult {
      values: [0.0; N],
      policies: [(); N].map(|_| vec![]),
    }
  }
}

pub struct RandomRolloutEval<M, ObservationSeq, SampleKey, Observation> {
  horizon: u32,
  phantom_data: PhantomData<(M, ObservationSeq, SampleKey, Observation)>,
}

impl<M, Os, S, O> RandomRolloutEval<M, Os, S, O> {
  pub fn new(horizon: u32) -> Self {
    RandomRolloutEval {
      horizon,
      phantom_data: Default::default(),
    }
  }
}

impl<M, ObservationSeq, SampleKey, Observation, State, Action, const N: usize>
  BaseEval<M, State, Action, N> for RandomRolloutEval<M, ObservationSeq, SampleKey, Observation>
where
  M: MaPomdp<ObservationSeq, SampleKey, Observation, State, Action, N>,
  Action: Default + Clone,
{
  fn evaluate<'a>(&self, problem: &M, state: &'a mut State) -> EvaluationResult<'a, Action, N> {
    let mut accum = [0.0; N];
    'outer: for _ in 0..self.horizon {
      let mut joint_action = [(); N].map(|_| Default::default());
      for agent in 0..N {
        let agent_actions = problem.actions(state, agent);
        if agent_actions.len() == 0 {
          break 'outer;
        }
        joint_action[agent] = agent_actions
          .choose(&mut rand::thread_rng())
          .unwrap()
          .clone();
      }
      let transition_result = problem.transition(state, &joint_action);
      for ix in 0..N {
        accum[ix] += transition_result.rewards[ix];
      }
    }
    EvaluationResult {
      values: accum,
      policies: [(); N].map(|_| vec![]),
    }
  }
}
