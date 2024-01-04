use std::fmt::{Debug, Display};

use ai::{MaMdp, TranstitionResult};
use rand::{seq::IteratorRandom, Rng};

pub struct Tzf8;

#[derive(Clone)]
pub struct State {
  board: [[u32; 4]; 4],
  ongoing: bool,
}

#[derive(Copy, Clone)]
pub struct Agent;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Move {
  Left,
  Right,
  Up,
  Down,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Observation {
  End,
  Result {
    shift: Move,
    // 2 or 4
    v: u8,
    // x and y cord of the cell
    x: u8,
    y: u8,
  },
}

impl MaMdp<State, Move, Observation, 1> for Tzf8 {
  fn actions(&self, state: &State, agent: usize) -> Vec<Move> {
    if state.ongoing {
      vec![Move::Left, Move::Right, Move::Up, Move::Down]
    } else {
      vec![]
    }
  }
  fn initial_state(&self) -> State {
    let mut result = State::new();
    result.add_random_tile();
    result.add_random_tile();
    result
  }
  fn transition(
    &self,
    state: &mut State,
    joint_action: &[Move; 1],
  ) -> TranstitionResult<Observation, 1> {
    let changed = state.apply_move(&joint_action[0]);
    if !changed {
      state.ongoing = false;
      return TranstitionResult {
        rewards: [0.0],
        observations: [Observation::End],
      };
    } else {
      let (v, x, y) = state.add_random_tile();
      return TranstitionResult {
        rewards: [v as f32],
        observations: [Observation::Result {
          shift: joint_action[0],
          v,
          x,
          y,
        }],
      };
    }
  }
  /*

  fn belief_update(&self, b_state: &mut Self::BeliefState, obs: &Self::Observation) {
    match obs {
      Observation::End => {
        b_state.ongoing = false;
      }
      Observation::Result { shift, v, x, y } => {
        b_state.apply_move(shift);
        b_state.board[*x as usize][*y as usize] = *v as u32;
      }
    }
  }
   */
}

impl State {
  pub fn new() -> Self {
    Self {
      board: [[0; 4]; 4],
      ongoing: true,
    }
  }

  fn shift_left(&mut self) -> bool {
    let mut changed = false;
    for r in 0..4 {
      let row_compressed = compress(&mut self.board[r]);
      let row_merged = merge(&mut self.board[r]);
      if row_compressed || row_merged {
        compress(&mut self.board[r]);
        changed = true;
      }
    }
    changed
  }

  fn transpose(&mut self) {
    for r in 0..4 {
      for c in (r + 1)..4 {
        let t = self.board[r][c];
        self.board[r][c] = self.board[c][r];
        self.board[c][r] = t;
      }
    }
  }

  fn reverse_rows(&mut self) {
    for r in 0..4 {
      for c in 0..2 {
        let t = self.board[r][c];
        self.board[r][c] = self.board[r][3 - c];
        self.board[r][3 - c] = t;
      }
    }
  }

  fn empty_cells(&self) -> Vec<(usize, usize)> {
    let mut cells = vec![];
    for r in 0..4 {
      for c in 0..4 {
        if self.board[r][c] == 0 {
          cells.push((r, c));
        }
      }
    }
    cells
  }

  pub fn apply_move(&mut self, m: &Move) -> bool {
    match m {
      Move::Left => self.shift_left(),
      Move::Right => {
        self.reverse_rows();
        let changed = self.shift_left();
        self.reverse_rows();
        changed
      }
      Move::Up => {
        self.transpose();
        let changed = self.shift_left();
        self.transpose();
        changed
      }
      Move::Down => {
        self.transpose();
        self.reverse_rows();
        let changed = self.shift_left();
        self.reverse_rows();
        self.transpose();
        changed
      }
    }
  }

  fn add_random_tile(&mut self) -> (u8, u8, u8) {
    let empty_cells = self.empty_cells();
    let (r, c) = empty_cells
      .into_iter()
      .choose(&mut rand::thread_rng())
      .unwrap();
    let p: f32 = rand::thread_rng().gen();
    let v = if p < 0.9 {
      self.board[r][c] = 2;
      2
    } else {
      self.board[r][c] = 4;
      4
    };
    (v, r as u8, c as u8)
  }

  pub fn largest_tile(&self) -> u32 {
    let mut max = 0;
    for r in 0..4 {
      for c in 0..4 {
        if max < self.board[r][c] {
          max = self.board[r][c];
        }
      }
    }
    max
  }
}

fn compress(row: &mut [u32]) -> bool {
  let mut pos = 0;
  let mut changed = false;
  for index in 0..row.len() {
    if row[index] != 0 {
      row[pos] = row[index];
      if pos != index {
        changed = true;
      }
      pos += 1;
    }
  }
  while pos < row.len() {
    row[pos] = 0;
    pos += 1;
  }
  changed
}

fn merge(row: &mut [u32]) -> bool {
  let mut changed = false;
  for index in 0..(row.len() - 1) {
    if row[index] != 0 && row[index] == row[index + 1] {
      changed = true;
      row[index] *= 2;
      row[index + 1] = 0;
    }
  }
  changed
}

impl Display for Move {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl Default for Move {
  fn default() -> Self {
    Move::Left
  }
}

impl Display for Observation {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Observation::End => write!(f, "End"),
      Observation::Result { shift, v, x, y } => write!(f, "O({shift:?}, {v}, ({x}, {y}))",),
    }
  }
}

impl Debug for Observation {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self)
  }
}

impl Display for State {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:2}", ' ')?;
    for c in 0..4 {
      write!(f, "|{c:7}")?;
    }
    writeln!(f, "|\n")?;
    for r in 0..4 {
      write!(f, "{:2}", 3 - r)?;
      for c in 0..4 {
        write!(f, "|{:7}", self.board[r][c])?;
      }
      writeln!(f, "|")?;
    }
    Ok(())
  }
}

impl Debug for State {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self)
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;

  use ai::search::{eval::ZeroEval, forest::refcnt_forest::Node, render::save, Search, Uct};

  use super::*;

  #[test]
  fn test_compress() {
    let mut row = [2, 2, 4, 4, 0, 4, 3, 1, 3, 0, 0, 0];
    println!("{:?}", row);
    compress(&mut row);
    println!("{:?}", row);
    merge(&mut row);
    println!("{:?}", row);
    compress(&mut row);
    println!("{:?}", row);
  }

  #[test]
  fn t1() {
    let game = Tzf8;
    let state = game.initial_state();
    let search = Search {
      tree_policy: Uct(2.4),
      base_eval: ZeroEval,
    };
    let root = Node::new();
    for _ in 0..10000 {
      search.step_mdp(&game, &state, [root.clone()]);
    }
    save([root], File::create("tzf8.t1.dot").unwrap(), 0, 2);
  }
}
