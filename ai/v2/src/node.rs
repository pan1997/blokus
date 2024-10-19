use std::{collections::BTreeMap, ops::AddAssign};

use num_traits::Float;

use crate::util::{NormalizingBounds, RunningAverage};

// We assume that P is nillable
pub struct Node<N, E, P, R> {
  pub key: N,
  pub select_count: u32,
  pub bounds: NormalizingBounds<R>,
  pub value: RunningAverage<R>,
  pub children: BTreeMap<E, Edge<P, R>>,
}

pub struct Edge<P, R> {
  pub select_count: u32,
  // This refer's to the value of target node, or the expected total reward of all simulations
  pub value: RunningAverage<R>,
  pub link: P,
}

pub trait NodeLink: Sized {
  fn is_nil(&self) -> bool;
  fn nil() -> Self;
}

pub trait NodeStore<N, E, P: NodeLink, R, K> {
  fn deref(&self, link: &P) -> &Node<N, E, P, R>;
  fn deref_mut(&mut self, link: &P) -> &mut Node<N, E, P, R>;
  fn new_node(&mut self, data: N, key: Option<&K>) -> P;
}

impl<N, E, P, R> Node<N, E, P, R>
where
  E: Ord,
  P: NodeLink,
  R: Float + AddAssign,
{
  pub fn new(key: N, outgoing: Vec<E>) -> Self {
    Self {
      key,
      select_count: 0,
      bounds: NormalizingBounds::new(),
      value: RunningAverage::new(),
      children: BTreeMap::from_iter(outgoing.into_iter().map(|e| {
        (
          e,
          Edge {
            select_count: 0,
            value: RunningAverage::new(),
            link: P::nil(),
          },
        )
      })),
    }
  }

  pub fn set_outgoing(&mut self, outgoing: Vec<E>) {
    self.children = BTreeMap::from_iter(outgoing.into_iter().map(|e| {
      (
        e,
        Edge {
          select_count: 0,
          value: RunningAverage::new(),
          link: P::nil(),
        },
      )
    }));
  }
}

impl<N, E, P: NodeLink, R> Default for Node<N, E, P, R>
where
  N: Default,
  E: Ord,
  R: Float + AddAssign,
{
  fn default() -> Self {
    Node::new(N::default(), vec![])
  }
}

pub struct Step<E, P, R> {
  pub node: P,
  pub edge: E,
  pub reward: R,
}
pub struct Trajectory<E, P, R> {
  pub steps: Vec<Step<E, P, R>>,
  pub last_node: P,
}

impl<E, P: NodeLink, R> Trajectory<E, P, R> {
  pub fn new() -> Self {
    Self {
      steps: vec![],
      last_node: P::nil(),
    }
  }
}

pub struct MultiTreeTrajectory<const N: usize, E, P, R> {
  pub trajectories: [Trajectory<E, P, R>; N],
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub enum DualType<A, B> {
  A(A),
  B(B),
}
