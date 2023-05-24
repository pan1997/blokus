use std::{fmt::Display};

use crate::{
  connection::connect4::Color::{Blue, Red},
  traits::pomdp::TranstitionResult,
  MaMdp,
};

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
enum Color {
  Red = 0,
  Blue = 1,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Move {
  Drop { color: Color, column: u8 },
  Observe,
}

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
      player_to_move: Some(Red),
      empty_cell_count: H * W,
    }
  }

  fn count_ray(&self, (mut x, mut y): (usize, usize), (dx, dy): (usize, usize)) -> u8 {
    let center = self.board[x][y];
    for len in 0..3 {
      x = x.wrapping_add(dx);
      y = y.wrapping_add(dy);
      if x > W || y > H || self.board[x][y] != center {
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
      let l = state.count_ray((column, state.heights[column]), (usize::MAX,0));
      let r = state.count_ray((column, state.heights[column]), (1,0));
      let d = state.count_ray((column, state.heights[column]), (0,usize::MAX));
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
      0 => Ok(Red),
      1 => Ok(Blue),
      _ => Err(()),
    }
  }
}

impl<const H: usize, const W: usize> Display for State<H, W> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for row in 0..H {
      for col in 0..W {
        
      }
    }
      Ok(())
  }
}

#[cfg(test)]
mod tests {
    use super::*;

  #[test]
  fn t1() {
    let game = C4;
    let state: State<6, 7> = game.initial_state();
    println!("{state}")
  }
}