use std::ops::AddAssign;

use num_traits::Float;

use crate::node::Node;
use crate::node::{NodeDeref, NodeLink};
struct NodeIndex(usize);

struct Forest<N, E, R> {
  nodes: Vec<Node<N, E, NodeIndex, R>>,
}

impl<N, E, R> Forest<N, E, R>
where
  N: Default,
  E: Ord,
  R: Float + AddAssign,
{
  fn new(capacity: usize) -> Self {
    let nil = Node::<N, E, NodeIndex, R>::default();
    let mut result = Self {
      nodes: Vec::with_capacity(capacity + 1),
    };
    result.nodes.push(nil);
    result
  }

  fn new_root(&mut self, data: N, outgoing: Vec<E>) -> NodeIndex {
    let index = self.nodes.len();
    self.nodes.push(Node::new(data, outgoing));
    NodeIndex(index)
  }
}

impl<N, E, R> NodeDeref<N, E, NodeIndex, R> for Forest<N, E, R> {
  fn deref(&self, link: &NodeIndex) -> &Node<N, E, NodeIndex, R> {
    &self.nodes[link.0]
  }
  fn deref_mut(&mut self, link: &NodeIndex) -> &mut Node<N, E, NodeIndex, R> {
    &mut self.nodes[link.0]
  }
}

impl NodeLink for NodeIndex {
  fn is_nil(&self) -> bool {
    self.0 == 0
  }
  fn nil() -> Self {
    Self(0)
  }
}
