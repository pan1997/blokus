use rand::seq::IteratorRandom;

use crate::{
  search::{forest::TreeNode, TreePolicy},
  MaPomdp,
};

pub struct Random;
impl<M, ObservationSeq, SampleKey, Observation, State, Action, TNode, const N: usize>
  TreePolicy<M, ObservationSeq, SampleKey, Observation, State, Action, TNode, N> for Random
where
  M: MaPomdp<ObservationSeq, SampleKey, Observation, State, Action, N>,
  TNode: TreeNode<Action, Observation>,
  Action: Clone,
{
  fn select_action(&self, problem: &M, state: &State, node: &TNode, agent: usize) -> Action {
    node
      .actions()
      .keys()
      .choose(&mut rand::thread_rng())
      .unwrap()
      .clone()
  }
}

pub struct Uct(pub f32);
impl<M, ObservationSeq, SampleKey, Observation, State, Action, TNode, const N: usize>
  TreePolicy<M, ObservationSeq, SampleKey, Observation, State, Action, TNode, N> for Uct
where
  M: MaPomdp<ObservationSeq, SampleKey, Observation, State, Action, N>,
  TNode: TreeNode<Action, Observation>,
  Action: Clone,
{
  fn select_action(&self, _problem: &M, _state: &State, node: &TNode, _agent: usize) -> Action {
    let actions = node.actions();
    let mut best_action = None;
    let mut best_action_score = f32::MIN;
    let lg_n = ((node.select_count() + 1) as f32).ln();
    for (action, info) in actions {
      if info.select_count == 0 {
        return action.clone();
      }
      let select_count = info.select_count as f32;
      let expected_value = info.action_value();
      let score = expected_value + (lg_n / select_count).sqrt() * self.0;
      //println!("ln_N: {lgN}, select_count: {select_count} score: {score}, best_score: {best_action_score}");
      if score > best_action_score {
        best_action_score = score;
        best_action = Some(action);
      }
    }
    best_action.unwrap().clone()
  }
}

// TODO
pub struct Puct;
