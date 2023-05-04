use rand::seq::IteratorRandom;

use super::{TreeNode, TreePolicy};
use crate::MaPomdp;

struct Uct(f32);

impl<M, ObservationSeq, Observation, State, Action, TNode, const N: usize>
  TreePolicy<M, ObservationSeq, Observation, State, Action, TNode, N> for Uct
where
  M: MaPomdp<ObservationSeq, Observation, State, Action, N>,
  TNode: TreeNode<Action, Observation>,
{
  fn select_action(&self, problem: &M, state: &State, node: &TNode, agent: usize) -> Action {
    unimplemented!()
  }
}

struct Random;
impl<M, ObservationSeq, Observation, State, Action, TNode, const N: usize>
  TreePolicy<M, ObservationSeq, Observation, State, Action, TNode, N> for Random
where
  M: MaPomdp<ObservationSeq, Observation, State, Action, N>,
  TNode: TreeNode<Action, Observation>,
  Action: Clone
{
  fn select_action(&self, problem: &M, state: &State, node: &TNode, agent: usize) -> Action {
    node.actions().keys().choose(&mut rand::thread_rng()).unwrap().clone()
  }
}
