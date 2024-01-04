use std::{
  cmp::{max, min},
  collections::{BTreeMap, BTreeSet},
  fmt::{Debug, Display},
};

use ai::{MaPomdp, SampleResult, TranstitionResult};
use colored::Colorize;
use itertools::Itertools;
use rand::seq::IteratorRandom;

struct Qwirkle<const N: usize>;

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Tile {
  // both shape and color start from 1,
  // and 0, 0 is an invalid tile
  shape: u8,
  color: u8,
}

struct State<const N: usize> {
  current_player: usize,
  // bag.. empty tiles moved to right
  hands: [[Tile; 6]; N],
  table: BTreeMap<(i16, i16), Tile>,
  boundry: BTreeSet<(i16, i16)>,
  bag: BTreeMap<Tile, u8>,
  bag_size: usize,
}

struct ObservationSeq {
  player: usize,
  hand: [Tile; 6],
  table: BTreeMap<(i16, i16), Tile>,
  player_to_move: usize,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Move {
  // all players other than the current player pass
  Pass,
  Placement(Vec<(Tile, i16, i16)>),
  Exchange(Vec<Tile>),
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Observation {
  // all players see the move if it's a placement move
  // for exchange moves, players only see Pass here,
  // but the pick
  action: Move,
  // only the current player sees the tiles that were picked
  // for other players, this is a vector of None(s)
  pick: Vec<Option<Tile>>,
}

const DIRECTIONS: [(i16, i16); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

impl<const N: usize> MaPomdp<ObservationSeq, [Tile; 6], Observation, State<N>, Move, N>
  for Qwirkle<N>
{
  fn start(&self, agent: usize) -> ObservationSeq {
    // todo: refactor to avoid state creation
    let mut state: State<N> = Default::default();
    let mut hand = [Tile::default(); 6];
    for (ix, tile) in state.tiles_from_bag(6).iter().enumerate() {
      hand[ix] = *tile;
    }
    ObservationSeq {
      player: agent,
      hand: hand,
      table: Default::default(),
      player_to_move: 0,
    }
  }

  fn actions(&self, state: &State<N>, agent: usize) -> Vec<Move> {
    let mut result = vec![];
    if state.current_player == agent {
      if state.hands[agent][0].shape != 0 {
        // current player has at least one tile
        // placements

        if state.table.is_empty() {
          // if table is empty then its the first move
          todo!("Implement first move");
        } else {
          // todo!("fill in placement computation");

          // iterate over the boundry of filled cells
          //   iterate over the tiles in hand
          //     check if placing this tile on this cell is legal (either shape or color matches on both horizontal & vertical, with no duplicates)
          //       if legal
          //          iterate over directions
          //            if the table is empty in this direction,
          /*
          for (x, y) in state.boundry {
              for tile in state.hands[agent] {
                  if state.is_tile_legal((x, y), tile) {
                      for (dx, dy) in DIRECTIONS {

                      }
                  }
              }
          }
          todo!("complete")*/

          // exchanges
          for combination in state.hands[state.current_player]
            .clone()
            .into_iter()
            .powerset()
          {
            result.push(Move::Exchange(combination));
          }
        }
      }
    } else {
      result.push(Move::Pass)
    }
    result
  }

  fn sample(
    &self,
    observation_seq: &ObservationSeq,
    agent: usize,
  ) -> SampleResult<State<N>, [Tile; 6], N> {
    if agent != observation_seq.player {
      panic!("Invalid agent sampling")
    }
    let mut state = State::default();
    state.current_player = observation_seq.player_to_move;

    state.table = observation_seq.table.clone();
    state.remove_from_bag(&state.table.values().map(|x| *x).collect_vec());

    for player in 0..N {
      let tiles = if player == agent {
        observation_seq.hand.clone()
      } else {
        state.tiles_from_bag(6).try_into().unwrap()
      };
      state.remove_from_bag(&tiles);
      for ix in 0..6 {
        state.hands[player][ix] = tiles[ix]
      }
    }

    state.compute_table_boundry();

    let hands = state.hands.clone();
    SampleResult {
      state,
      sample_keys: hands,
    }
  }

  fn append(&self, observation_seq: &mut ObservationSeq, agent: usize, obs: Observation) {
    if observation_seq.player == agent {
      unimplemented!("implement this")
    } else {
      unimplemented!("update table")
    }
  }

  fn transition(
    &self,
    state: &mut State<N>,
    joint_action: &[Move; N],
  ) -> ai::TranstitionResult<Observation, N> {
    let mut result = None;
    for (player, action) in joint_action.iter().enumerate() {
      if player == state.current_player {
        match action {
          Move::Pass => {
            unimplemented!("Pass for current player not implemented")
          }
          Move::Exchange(tiles) => {
            // remove from hand
            for tile in tiles {
              if *tile == Tile::nil() {
                panic!("Cannot exchange nil tile");
              }
              for j in 0..6 {
                if *tile == state.hands[state.current_player][j] {
                  state.hands[state.current_player][j] = Tile::nil();
                  break; // inner
                }
              }
            }
            // put in bag
            state.insert_into_bag(tiles);

            // get new tiles
            let new_tiles = state.tiles_from_bag(tiles.len());
            state.remove_from_bag(&new_tiles);
            for tile in new_tiles.iter() {
              for j in 0..6 {
                if state.hands[state.current_player][j] == Tile::nil() {
                  state.hands[state.current_player][j] = *tile;
                  break; //inner
                }
              }
            }
            let mut tr = TranstitionResult {
              rewards: [0.0; N],
              observations: [0; N].map(|ix| Observation {
                pick: vec![None; tiles.len()],
                action: Move::Pass,
              }),
            };
            tr.observations[state.current_player].action = action.clone();
            let new_tiles_op: Vec<_> = new_tiles.into_iter().map(|x| Some(x)).collect();
            tr.observations[state.current_player]
              .pick
              .clone_from(&new_tiles_op);
            result.replace(tr);
          }
          Move::Placement(x) => {
            unimplemented!()
          }
        }
      } else if *action != Move::Pass {
        panic!("All players other than current should pass")
      }
    }
    result.unwrap()
  }
}

impl<const N: usize> Default for State<N> {
  fn default() -> Self {
    let mut result = State {
      current_player: 0,
      hands: [[Tile::default(); 6]; N],
      table: Default::default(),
      boundry: Default::default(),
      bag: Default::default(),
      bag_size: 108,
    };
    for shape in 1..7 {
      for color in 1..7 {
        result.bag.insert(Tile { shape, color }, 3);
      }
    }
    result
  }
}

impl<const N: usize> State<N> {
  fn initialize_hands(&mut self) {
    for player in 0..N {
      let tiles = self.tiles_from_bag(6);
      println!("bag:   {:?}", self.bag);
      println!("tiles: {tiles:?}");
      self.remove_from_bag(&tiles);
      for ix in 0..6 {
        self.hands[player][ix] = tiles[ix]
      }
    }
  }

  fn next_player(&mut self) {
    self.current_player += 1;
    if self.current_player >= N {
      self.current_player = 0;
    }
  }

  fn bounding_rectangle(&self) -> (i16, i16, i16, i16) {
    self
      .table
      .keys()
      .fold((0, 0, 0, 0), |(lx, ly, hx, hy), (x, y)| {
        (min(lx, *x), min(ly, *y), max(hx, *x), max(hy, *y))
      })
  }

  fn tiles_from_bag(&self, count: usize) -> Vec<Tile> {
    let mut rng = rand::thread_rng();
    let indexes = (0..self.bag_size).choose_multiple(&mut rng, count);
    let tiles: Vec<_> = indexes
      .iter()
      .map(|index| {
        let mut ix = *index;
        for shape in 1..7 {
          for color in 1..7 {
            let t = Tile { shape, color };
            let tix = *self.bag.get(&t).unwrap_or(&0) as usize;
            if tix > ix {
              return t;
            } else {
              ix -= tix;
            }
          }
        }
        Tile::default()
      })
      .collect();
    tiles
  }

  fn remove_from_bag(&mut self, tiles: &[Tile]) {
    for tile in tiles {
      if tile.shape != 0 && tile.color != 0 {
        self.bag.insert(
          *tile,
          self.bag.get(tile).map(|x| *x).unwrap_or_default() - 1,
        );
        self.bag_size -= 1;
      }
    }
  }

  fn insert_into_bag(&mut self, tiles: &[Tile]) {
    for tile in tiles {
      if tile.shape != 0 && tile.color != 0 {
        self.bag.insert(
          *tile,
          self.bag.get(tile).map(|x| *x).unwrap_or_default() + 1,
        );
        self.bag_size += 1;
      }
    }
  }

  fn compute_first_move_for_hand(hand: &[Tile]) {
    let mut same_color = vec![0; hand.len()];
    let mut same_shape = vec![0; hand.len()];
    for ix in 0..hand.len() {
      for jx in 0..ix {
        // exclude duplicate tiles
        if hand[ix].shape == hand[jx].shape && hand[ix].color != hand[jx].color {
          same_shape[ix] += 1;
          same_shape[jx] += 1;
        }
        if hand[ix].shape != hand[jx].shape && hand[ix].color == hand[jx].color {
          same_color[ix] += 1;
          same_color[jx] += 1;
        }
      }
    }
    unimplemented!()
  }

  fn is_tile_legal(&self, (x, y): (i16, i16), tile: Tile) -> bool {
    if !self.boundry.contains(&(x, y)) {
      return false;
    }
    true
  }

  fn compute_table_boundry(&mut self) {
    unimplemented!("todo");
  }
}

impl Display for Tile {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let c = match self.shape {
      0 => " ", // nil tile
      1 => "●",
      2 => "✖",
      3 => "◆",
      4 => "■",
      5 => "🟏",
      6 => "🞧",
      _ => panic!("djdjd"),
    };
    let cc = match self.color {
      0 => c.white(),
      1 => c.truecolor(255, 0, 0),
      2 => c.truecolor(255, 187, 51),
      3 => c.truecolor(255, 255, 0),
      4 => c.truecolor(0, 255, 0),
      5 => c.truecolor(0, 0, 255),
      6 => c.truecolor(191, 64, 191),
      _ => panic!("djdjd"),
    };
    write!(f, "{cc}")
  }
}

impl Debug for Tile {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{self}")
  }
}

impl<const N: usize> Display for State<N> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let (low_x, low_y, high_x, high_y) = self.bounding_rectangle();
    for x in (low_x - 3)..(high_x + 4) {
      for y in (low_y - 3)..(high_y + 4) {
        let t = self
          .table
          .get(&(x - low_x, y - low_y))
          .map(|x| *x)
          .unwrap_or_default();
        write!(f, "{t} ")?;
      }
      writeln!(f, "|")?;
    }
    for player in 0..N {
      write!(f, "P{player} [")?;
      for t in self.hands[player].iter() {
        write!(f, "{t} ")?;
      }
      writeln!(f, "]")?;
    }

    writeln!(f, "{:?}", self.bag)?;
    Ok(())
  }
}

impl Tile {
  fn nil() -> Tile {
    Tile { shape: 0, color: 0 }
  }
}

#[cfg(test)]
mod tests {
  use super::{State, Tile};

  #[test]
  fn test_tile_display() {
    for i in 1..7 {
      for j in 1..7 {
        let t = Tile { shape: j, color: i };
        print!("{t}")
      }
      println!()
    }
  }

  #[test]
  fn test_state_display() {
    let mut state = State::<4>::default();
    state.initialize_hands();
    println!("{state}");
  }
}
