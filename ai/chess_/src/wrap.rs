use std::fmt::Display;

use ai::{MaMdp, TranstitionResult};
use chess::{Board, BoardStatus, ChessMove, Color, MoveGen};

pub struct Chess;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum MoveWrapper {
  Pass,
  Move(ChessMove),
}

// The action and observation for the mdp is Option<ChessMove>
// because of the constraint that every player needs to have
// a move (pass move if needed) for every non terminal state
impl MaMdp<Board, MoveWrapper, MoveWrapper, 2> for Chess {
  fn actions(&self, state: &Board, agent: usize) -> Vec<MoveWrapper> {
    let current_agent_index = state.side_to_move().to_index();
    if current_agent_index != agent {
      vec![MoveWrapper::Pass]
    } else {
      MoveGen::new_legal(state)
        .map(|m| MoveWrapper::Move(m))
        .collect()
    }
  }

  fn initial_state(&self) -> Board {
    Board::default()
  }

  fn transition(
    &self,
    state: &mut Board,
    joint_action: &[MoveWrapper; 2],
  ) -> TranstitionResult<MoveWrapper, 2> {
    let current_agent_index = state.side_to_move().to_index();
    debug_assert!(
      joint_action[current_agent_index] != MoveWrapper::Pass,
      "Current side to move cannot pass"
    );
    debug_assert!(
      joint_action[1 - current_agent_index] == MoveWrapper::Pass,
      "Only one side can move at a time"
    );
    if let MoveWrapper::Move(m) = joint_action[current_agent_index] {
      let result = state.make_move_new(m);
      *state = result;
      let rewards = match state.status() {
        BoardStatus::Ongoing | BoardStatus::Stalemate => [0.0, 0.0],
        BoardStatus::Checkmate => match state.side_to_move() {
          // black just moved and won
          Color::White => [0.0, 1.0],
          Color::Black => [1.0, 0.0],
        },
      };
      TranstitionResult {
        observations: [
          joint_action[current_agent_index],
          joint_action[current_agent_index],
        ],
        rewards,
      }
    } else {
      panic!()
    }
  }
}

impl Default for MoveWrapper {
  fn default() -> Self {
    MoveWrapper::Pass
  }
}

impl Display for MoveWrapper {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      MoveWrapper::Pass => write!(f, "Pass"),
      MoveWrapper::Move(m) => write!(f, "{m}"),
    }
  }
}
