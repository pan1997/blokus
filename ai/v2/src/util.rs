use std::{
  any::TypeId,
  fmt::{self, Debug, Display},
  fs::File,
  io::Write,
  ops::{AddAssign, SubAssign},
};

use graphviz_rust::{
  attributes::{bgcolor, EdgeAttributes, NodeAttributes},
  dot_structures::{
    Edge as GEdge, EdgeTy, Graph, Id, Node as GNode, NodeId as GNid, Port, Stmt, Vertex,
  },
  printer::{DotPrinter, PrinterContext},
};
use num_traits::Float;

use crate::node::{DualType, Node, NodeLink, NodeStore, Trajectory};

fn propogate<N, E, P, ND, R, K>(
  nd: &mut ND,
  mut trajectory: Trajectory<E, P, R>,
  last_node: P,
  mut value: R,
) where
  P: NodeLink + Clone,
  E: Ord,
  ND: NodeStore<N, E, P, R, K>,
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

fn render<NS: NodeStore<N, E, P, R, K>, N, E: Ord + Debug, P: NodeLink + Debug, R: Display, K>(
  node_store: &NS,
  g: &mut Graph,
  root: &P,
  count: &mut u32,
) -> GNid {
  let id = *count;
  *count += 1;
  let label = node_label(node_store.deref(root));
  let shape = if root.is_nil() {
    graphviz_rust::attributes::shape::point
  } else {
    graphviz_rust::attributes::shape::plaintext
  };
  let n = GNode::new(
    GNid(Id::Plain(format!("{id}")), None),
    vec![NodeAttributes::label(label), NodeAttributes::shape(shape)],
  );
  g.add_stmt(Stmt::Node(n));

  if !root.is_nil() {
    let data = node_store.deref(root);
    for (ix, o) in data.children.keys().enumerate() {
      let edge = &data.children[o];
      let cid = render(node_store, g, &edge.link, count);
      let e = GEdge {
        ty: EdgeTy::Pair(
          Vertex::N(GNid(
            Id::Plain(format!("{id}")),
            Some(Port(Some(Id::Plain(format!("{ix}"))), None)),
          )),
          Vertex::N(cid),
        ),
        attributes: vec![], //vec![EdgeAttributes::label(format!("\"{o:?}\""))],
      };
      g.add_stmt(Stmt::Edge(e));
    }
  }

  GNid(Id::Plain(format!("{id}")), None)
}

pub fn save<NS: NodeStore<N, E, P, R, K>, N, E: Ord + Debug, P: NodeLink + Debug, R: Display, K>(
  node_store: &NS,
  mut f: File,
  root: &P,
) {
  let mut g = Graph::DiGraph {
    id: Id::Plain("T".to_string()),
    strict: false,
    stmts: vec![],
  };
  let mut count = 0;
  render(node_store, &mut g, root, &mut count);
  let mut ctx = PrinterContext::default();
  write!(f, "{}", g.print(&mut ctx)).unwrap();
}

fn node_label<N, E: Ord + Debug, P, R: Display>(node: &Node<N, E, P, R>) -> String {
  let out_row = if node.children.is_empty() {
    "".to_string()
  } else {
    let mut bgcolor = "gold";
    let mut result =
      format!("<table bgcolor=\"{bgcolor}\" border=\"0\" cellspacing=\"0\" cellborder=\"1\"><tr>")
        .to_string();
    for (ix, o) in node.children.keys().enumerate() {
      let e = &node.children[o];
      result.push_str(&format!(
        "<td port=\"{ix}\">{o:?}<BR/>{}<BR/>{}</td>",
        e.select_count, e.value.mean
      ));
    }
    result.push_str("</tr></table>");
    result
  };
  format!(
    r#"<
<table border="0" cellspacing="0" cellborder="1">
<tr><td>{}</td></tr>
<tr><td>{:.4}, {}</td></tr>
<tr><td>{out_row}</td></tr>
</table>
    >"#,
    node.select_count, node.value.mean, node.value.count,
  )
}
