use std::fs::File;

use rustyai::{
  search::{
    bandits::Uct,
    eval::{RandomRolloutEval, ZeroEval},
    forest::{refcnt_forest::Node, TreeNode, TreeNodePtr},
    render::save,
    Search,
  },
  MaMdp,
};

use crate::wrap::Chess;

mod wrap;

fn main() {
  println!("Chess test");
  let g = Chess;
  let board = g.initial_state();
  let search = Search {
    tree_policy: Uct(2.4),
    base_eval: RandomRolloutEval::new(100),
  };
  let roots = [Node::new(), Node::new()];
  for _ in 0..500000 {
    search.step_mdp(&g, &board, [roots[0].clone(), roots[1].clone()]);
  }
  let g = roots[0].lock();
  let p = g.compute_policy();
  for (m, a, b) in p {
    println!("{m}, {a}, {b}")
  }
  //save([roots[0].clone()], File::create("chess.main.dot").unwrap(), 0, 2);
}
