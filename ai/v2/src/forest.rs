use std::fmt::Debug;
use std::ops::AddAssign;

use num_traits::Float;

use crate::node::Node;
use crate::node::{NodeLink, NodeStore};

#[derive(Clone, Copy, Debug)]
pub struct NodeIndex(usize);

pub struct Forest<N, E, R> {
  nodes: Vec<Node<N, E, NodeIndex, R>>,
}

impl<N, E, R> Forest<N, E, R>
where
  N: Default,
  E: Ord,
  R: Float + AddAssign,
{
  pub fn new(capacity: usize) -> Self {
    let nil = Node::<N, E, NodeIndex, R>::default();
    let mut result = Self {
      nodes: Vec::with_capacity(capacity + 1),
    };
    result.nodes.push(nil);
    result
  }

  pub fn new_root(&mut self, data: N, outgoing: Vec<E>) -> NodeIndex {
    let index = self.nodes.len();
    self.nodes.push(Node::new(data, outgoing));
    NodeIndex(index)
  }
}

impl<N, E, R, K> NodeStore<N, E, NodeIndex, R, K> for Forest<N, E, R>
where
  N: Default,
  E: Ord,
  R: Float + AddAssign,
{
  fn deref(&self, link: &NodeIndex) -> &Node<N, E, NodeIndex, R> {
    &self.nodes[link.0]
  }
  fn deref_mut(&mut self, link: &NodeIndex) -> &mut Node<N, E, NodeIndex, R> {
    &mut self.nodes[link.0]
  }
  fn new_node(&mut self, data: N, _key: Option<&K>) -> NodeIndex {
    // key is ignored here
    return self.new_root(data, vec![]);
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

impl<N, E: Ord + Debug, R> Debug for Forest<N, E, R> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "[")?;
    for (ix, node) in self.nodes.iter().enumerate() {
      write!(f, "{ix}: [",)?;
      for e in node.children.keys() {
        write!(f, "({:?} {:?}), ", e, node.children[e].link)?;
      }
      write!(f, "], ")?;
    }
    write!(f, "]")
  }
}
