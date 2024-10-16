use std::ops::{AddAssign, SubAssign};

use num_traits::Float;

use crate::node::{NodeDeref, NodeLink, Trajectory};

fn propogate<N, E, P, ND, R>(
  nd: &mut ND,
  mut trajectory: Trajectory<E, P, R>,
  last_node: P,
  mut value: R,
) where
  P: NodeLink + Clone,
  E: Ord,
  ND: NodeDeref<N, E, P, R>,
  R: Float + AddAssign + SubAssign,
{
  nd.deref_mut(&last_node).value.add_sample(value, 1);

  while let Some(step) = trajectory.steps.pop() {
    let reward = step.reward;
    let edge = step.edge;

    let node = nd.deref_mut(&step.node);
    node
      .children
      .get_mut(&edge)
      .unwrap()
      .value
      .add_sample(value, 1);
    value -= reward;
    node.value.add_sample(value, 1);
  }
}

pub struct RunningAverage<R> {
  mean: R,
  count: u32,
}

impl<R> RunningAverage<R>
where
  R: Float + AddAssign,
{
  pub fn new() -> Self {
    Self {
      mean: R::zero(),
      count: 0,
    }
  }

  pub fn value(&self) -> R {
    self.mean
  }

  pub fn count(&self) -> u32 {
    self.count
  }

  pub fn add_sample(&mut self, mut v: R, c: u32) {
    let new_c = c + self.count;
    self.mean += (v - self.mean) * R::from(c).unwrap() / R::from(new_c).unwrap();
    self.count = new_c;
  }
}

#[derive(Debug, Clone)]
pub struct NormalizingBounds<R> {
  low: R,
  high: R,
}

impl<R> NormalizingBounds<R>
where
  R: Float,
{
  pub fn new_known(low: R, high: R) -> Self {
    Self { low, high }
  }

  pub fn new() -> Self {
    Self {
      low: R::one(),
      high: R::zero(),
    }
  }

  pub fn normalise(&self, v: R) -> R {
    if self.low >= self.high {
      R::zero()
    } else {
      (v - self.low) / (self.high - self.low)
    }
  }

  pub fn update_bounds(&mut self, v: R) {
    if self.low > self.high {
      self.low = v;
      self.high = v;
    } else {
      if v < self.low {
        self.low = v;
      }
      if v > self.high {
        self.high = v
      }
    }
  }
}
