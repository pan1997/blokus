use std::fmt::Display;

<<<<<<<< HEAD:ai/connection/src/connect4/mod.rs
use crate::connect4::Color::{Blue, Red};
use ai::TranstitionResult;
use ai::MaMdp;
========
use ai::{
  TranstitionResult,
  MaMdp,
};
>>>>>>>> d2959eb (Refactor connect4 into seperate crate):ai/connect4/src/lib.rs

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Color {
  Red = 0,
  Blue = 1,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Move {
  Drop { color: Color, column: u8 },
  Observe,
}

#[derive(Clone)]
struct State<const H: usize, const W: usize> {
  board: [[Option<Color>; H]; W],
  heights: [usize; W],
  // If the game has ended, then player_to_move is None
  player_to_move: Option<Color>,
  empty_cell_count: usize,
}

impl<const H: usize, const W: usize> State<H, W> {
  fn new() -> Self {
    State {
      board: [[None; H]; W],
      heights: [0; W],
      player_to_move: Some(Color::Red),
      empty_cell_count: H * W,
    }
  }

  fn count_ray(&self, (mut x, mut y): (usize, usize), (dx, dy): (usize, usize)) -> u8 {
    let center = self.board[x][y];
    for len in 0..3 {
      x = x.wrapping_add(dx);
      y = y.wrapping_add(dy);
      if x >= W || y >= H || self.board[x][y] != center {
        return len;
      }
    }
    3
  }
}

struct C4;

impl<const H: usize, const W: usize> MaMdp<State<H, W>, Move, Move, 2> for C4 {
  fn initial_state(&self) -> State<H, W> {
    State::new()
  }

  fn actions(&self, state: &State<H, W>, agent: usize) -> Vec<Move> {
    if state.player_to_move.is_none() {
      // Game over
      vec![]
    } else if state.player_to_move.unwrap() as usize == agent {
      let mut result = vec![];
      for column in 0..W {
        if state.heights[column] < H {
          result.push(Move::Drop {
            color: state.player_to_move.unwrap(),
            column: column as u8,
          })
        }
      }
      result
    } else {
      // opponent can only observe
      vec![Move::Observe]
    }
  }

  fn transition(
    &self,
    state: &mut State<H, W>,
    joint_action: &[Move; 2],
  ) -> TranstitionResult<Move, 2> {
    let agent_index = state.player_to_move.unwrap() as usize;
    let opponent_index = (agent_index + 1) % 2;
    debug_assert!(
      joint_action[opponent_index] == Move::Observe,
      "Invalid joint_action"
    );
    let Move::Drop { color, column } = joint_action[agent_index] else {
      panic!("current agent cannot observe");
    };
    debug_assert!(color as usize == agent_index, "Invalid player dropping");
    let column = column as usize;
    let mut rewards = [0.0, 0.0];

    state.board[column][state.heights[column]] = state.player_to_move;
    state.empty_cell_count -= 1;

    if state.empty_cell_count == 0 {
      state.player_to_move = None;
    } else {
      let l = state.count_ray((column, state.heights[column]), (usize::MAX, 0));
      let r = state.count_ray((column, state.heights[column]), (1, 0));
      let d = state.count_ray((column, state.heights[column]), (0, usize::MAX));
      // TODO: diagonal
      if l + r >= 4 || d >= 4 {
        rewards[agent_index] = 1.0;
        state.player_to_move = None
      } else {
        state.player_to_move = (opponent_index as u8).try_into().ok()
      }
    }
    state.heights[column] += 1;

    TranstitionResult {
      rewards,
      // both agents see the move played
      observations: [joint_action[agent_index], joint_action[agent_index]],
    }
  }
}

impl TryFrom<u8> for Color {
  type Error = ();
  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0 => Ok(Color::Red),
      1 => Ok(Color::Blue),
      _ => Err(()),
    }
  }
}

impl<const H: usize, const W: usize> Display for State<H, W> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for row in 0..H {
      for col in 0..W {}
    }
    Ok(())
  }
}

// TODO: remove
impl Default for Move {
  fn default() -> Self {
    Move::Observe
  }
}

impl Display for Move {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &self {
      Move::Observe => write!(f, "Pass"),
      Move::Drop { color, column } => write!(f, "Drop({color},{column})"),
    }
  }
}

impl Display for Color {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &self {
      Color::Blue => write!(f, "B"),
      Color::Red => write!(f, "R"),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;

  use super::*;
  use ai::search::{
    bandits::Random,
    eval::{RandomRolloutEval, ZeroEval},
    forest::{refcnt_forest::Node, TreeNode, TreeNodePtr},
    render::save,
    Search, Uct,
  };

  #[test]
  fn t1() {
    let game = C4;
    let state: State<6, 7> = game.initial_state();
    println!("{state}")
  }

  #[test]
  fn test_random() {
    let game = C4;
    let state: State<6, 7> = game.initial_state();
    let s = Search::new(Random, ZeroEval);
    let trees = [Node::new(), Node::new()];
    for iter in 0..1000 {
      let n = [trees[0].clone(), trees[1].clone()];
      let x = s.step_mdp(&game, &state, n);
      //println!("{iter}: rewards: {x:?}");
    }
    for ix in 0..2 {
      println!("player: {ix}");
      let guard = trees[ix].lock();
      let policy = guard.compute_policy();
      for (m, pi, q) in policy {
        println!("policy of {m} is {pi} and q is {q}")
      }
    }
    save(trees, File::create("c4.random.dot").unwrap(), 0, 4);
  }

  #[test]
  fn test_uct() {
    let game = C4;
    let state: State<6, 7> = game.initial_state();
    let s = Search::new(Uct(2.4), RandomRolloutEval::new(100));
    let trees = [Node::new(), Node::new()];
    for iter in 0..100000 {
      let n = [trees[0].clone(), trees[1].clone()];
      let x = s.step_mdp(&game, &state, n);
      //println!("{iter}: rewards: {x:?}");
    }
    for ix in 0..2 {
      println!("player: {ix}");
      let guard = trees[ix].lock();
      let policy = guard.compute_policy();
      for (m, pi, q) in policy {
        println!("policy of {m} is {pi} and q is {q}")
      }
    }
    save(trees, File::create("c4.uct.dot").unwrap(), 0, 4);
  }
}
