use std::{
  cmp::{max, min},
  collections::{BTreeMap, BTreeSet},
  fmt::{Debug, Display},
};
use std::os::macos::raw::stat;

use rustyai::{MaPomdp, SampleResult, TranstitionResult};
use colored::Colorize;
use itertools::Itertools;
use rand::seq::IteratorRandom;

pub(crate) struct Qwirkle<const N: usize>;

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tile {
  // both shape and color start from 1,
  // and 0, 0 is an invalid tile
  shape: u8,
  color: u8,
}

pub struct State<const N: usize> {
  current_player: usize,
  // bag.. empty tiles moved to right
  hands: [[Tile; 6]; N],
  table: BTreeMap<(i16, i16), Tile>,
  bag: BTreeMap<Tile, u8>,
  bag_size: usize,
}

#[derive(Debug)]
pub struct ObservationSeq {
  player: usize,
  hand: [Tile; 6],
  table: BTreeMap<(i16, i16), Tile>,
  player_to_move: usize,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Move {
  // all players other than the current player pass
  Pass,
  Placement(Vec<(Tile, i16, i16)>),
  Exchange(Vec<Tile>),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct Observation {
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
    let state: State<N> = Default::default();
    let mut hand = [Tile::default(); 6];
    for (ix, tile) in state.tiles_from_bag(6).iter().enumerate() {
      hand[ix] = *tile;
    }

    ObservationSeq {
      player: agent,
      hand,
      table: Default::default(),
      player_to_move: 0,
    }
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

    let hands = state.hands.clone();
    SampleResult {
      state,
      sample_keys: hands,
    }
  }

  fn actions(&self, state: &State<N>, agent: usize) -> Vec<Move> {
    let mut result = vec![];
    if state.current_player == agent {
      if state.table.is_empty() {
        // if table is empty then its the first move
        // iterate over all powersets of hand
        // if valid (ie either chand or color match), then thats a valid placement
        // assumes all tiles are valid
        for mut combination in state.hands[state.current_player]
          .clone()
          .into_iter()
          .powerset()
        {
          // skip single length combinations
          if combination.len() > 1 {
            combination.sort();
            if Tile::valid(&combination) {
              for perm in combination.clone().into_iter().permutations(combination.len()) {
                result.push(Move::Placement(perm.into_iter().enumerate().map(|(x, tile)| (tile, x as i16, 0)).collect()))
              }
            }
          }
        }
      } else {
        // todo!("fill in placement computation");

        // iterate over the boundry of filled cells
        //   iterate over the tiles in hand
        //     check if placing this tile on this cell is legal (either shape or color matches on both horizontal & vertical, with no duplicates)


        for (x, y) in state.table.keys() {
          for direction in DIRECTIONS {
            let cell = (*x + direction.0, *y + direction.1);
            if !state.table.contains_key(&cell) {
              for tile in state.hands[state.current_player] {
                if state.is_placement_legal(cell, tile) {

                  // todo: more than one tile
                  result.push(Move::Placement(vec![(tile, cell.0, cell.1)]));
                }
              }
            }
          }
        }

        // exchanges

        // fill exchanges only if no placements
        if result.len() <= 1 {
          let mut hand = vec![];
          for ix in 0..6 {
            if state.hands[state.current_player][ix] != Tile::nil() {
              hand.push(state.hands[state.current_player][ix]);
            }
          }
          for combination in hand
              .clone()
              .into_iter()
              .powerset()
          {
            if !combination.is_empty() {
              result.push(Move::Exchange(combination));
            }
          }
        }
      }
    } else {
      result.push(Move::Pass)
    }
    result
  }

  fn transition(
    &self,
    state: &mut State<N>,
    joint_action: &[Move; N],
  ) -> rustyai::TranstitionResult<Observation, N> {
    let mut result = None;
    for (player, action) in joint_action.iter().enumerate() {
      if player == state.current_player {
        match action {
          Move::Pass => {
            unimplemented!("Pass for current player not implemented")
          }
          Move::Exchange(tiles) => {
            for tile in tiles {
              remove_from_hand(&mut state.hands[player], tile)
            }
            state.insert_into_bag(tiles);

            // get new tiles
            let new_tiles = state.tiles_from_bag(tiles.len());
            state.remove_from_bag(&new_tiles);
            for tile in new_tiles.iter() {
               insert_into_hand(&mut state.hands[player], tile)
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
          Move::Placement(placement) => {
            for (tile, x, y) in placement {
              // remove from hand
              remove_from_hand(&mut state.hands[player], tile);
              // place on table
              state.table.insert((*x, *y), *tile);
            }

            // get new tiles
            let new_tiles = state.tiles_from_bag(placement.len());
            state.remove_from_bag(&new_tiles);
            for tile in new_tiles.iter() {
              insert_into_hand(&mut state.hands[player], tile)
            }

            let mut tr = TranstitionResult {
              rewards: [0.0; N],
              observations: [0; N].map(|ix| Observation {
                pick: vec![None; placement.len()],
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
        }
      } else if *action != Move::Pass {
        panic!("All players other than current should pass")
      }
    }
    result.unwrap()
  }

  fn append(&self, observation_seq: &mut ObservationSeq, agent: usize, obs: Observation) {
    match obs.action {
      Move::Exchange(tiles) => {
        if agent != observation_seq.player {
          panic!("Exchange moves seen as pass for other players")
        }
        for tile in tiles {
          remove_from_hand(&mut observation_seq.hand, &tile);
          // no change to table
        }
      }
      Move::Pass => {
        // do nothing
      }
      Move::Placement(placements) => {
        // place on table
        // remove from hand if current player

        for (tile, x, y) in placements {
          observation_seq.table.insert((x, y), tile);
          if observation_seq.player_to_move == agent {
            remove_from_hand(&mut observation_seq.hand, &tile);
          }
        }
      }
    }

    observation_seq.player_to_move += 1;
    if observation_seq.player_to_move == N {
      observation_seq.player_to_move = 0;
    }
  }
}

impl<const N: usize> Default for State<N> {
  fn default() -> Self {
    let mut result = State {
      current_player: 0,
      hands: [[Tile::default(); 6]; N],
      table: Default::default(),
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

  pub fn current_player(&self) -> usize {
    self.current_player
  }
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

  fn is_placement_legal(&self, (x, y): (i16, i16), tile: Tile) -> bool {
    // placement is legal iff
    // 1. cell is originally empty
    // 2. both horizontal and vertical lines including this tile are valid

    if tile == Tile::nil() {
      return false
    }

    if self.table.contains_key(&(x, y)) {
      return false
    }

    {
      let mut horizontal = vec![tile];
      for ix in 1..7 {
        let cell = (x + ix, y);
        let tile_opt = self.table.get(&cell);
        if tile_opt.is_none() {
          break;
        }
        horizontal.push(tile_opt.unwrap().clone())
      }
      for ix in 1..7 {
        let cell = (x - ix, y);
        let tile_opt = self.table.get(&cell);
        if tile_opt.is_none() {
          break;
        }
        horizontal.push(tile_opt.unwrap().clone())
      }
      horizontal.sort();
      if !Tile::valid(&horizontal) {
        return false
      }
    }

    {
      let mut vertical = vec![tile];
      for ix in 1..7 {
        let cell = (x, y + ix);
        let tile_opt = self.table.get(&cell);
        if tile_opt.is_none() {
          break;
        }
        vertical.push(tile_opt.unwrap().clone())
      }
      for ix in 1..7 {
        let cell = (x, y - ix);
        let tile_opt = self.table.get(&cell);
        if tile_opt.is_none() {
          break;
        }
        vertical.push(tile_opt.unwrap().clone())
      }
      vertical.sort();
      if !Tile::valid(&vertical) {
        return false
      }
    }

    return true
  }
}

fn remove_from_hand(hand: &mut [Tile; 6], tile: &Tile) {
  if *tile == Tile::nil() {
    panic!("Cannot remove nil tile from hand");
  }
  for j in 0..6 {
    if hand[j] == *tile {
      hand[j] = Tile::nil();
      return;
    }
  }
}

fn insert_into_hand(hand: &mut [Tile; 6], tile: &Tile) {
  if *tile == Tile::nil() {
    panic!("Cannot insert nil tile to hand");
  }
  for j in 0..6 {
    if hand[j] == Tile::nil() {
      hand[j] = *tile;
      return;
    }
  }
}

fn next_player(player: &mut usize, player_count: usize) {
  *player += 1;
  if *player >= player_count {
    *player = 0;
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
      5 => "+",//"🟏",
      6 => "~",//"🞧",
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

  fn valid(tiles: &[Tile]) -> bool {
    // assumes sorted
    let mut color_match = true;
    let mut shape_match = true;
    for ix in 1..tiles.len() {
      color_match = color_match && tiles[ix].color == tiles[0].color;
      shape_match = shape_match && tiles[ix].shape == tiles[0].shape;
      if tiles[ix] == tiles[ix -1] || !color_match && !shape_match {
        return false;
      }
    }
    color_match || shape_match
  }
}

#[cfg(test)]
mod tests {
  use itertools::assert_equal;
  use rand::seq::SliceRandom;
  use rustyai::MaPomdp;

  use super::{Move, Qwirkle, State, Tile};

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
    assert_eq!(state.bag_size, 108 - 4 * 6);
    println!("{state}");
  }

  #[test]
  fn test_qwirkle_basic() {
    let g = Qwirkle::<2>;
    let agent = 0;
    let o_seq = g.start(agent);
    let sr = g.sample(&o_seq, agent);
    let state = sr.state;
    let keys = sr.sample_keys;
    assert_eq!(keys[agent], o_seq.hand);
    assert_eq!(state.hands, keys);

    let ac_0 = g.actions(&state, 0);
    let ac_1 = g.actions(&state, 1);

    assert_eq!(ac_1.len(), 1);
    assert_eq!(ac_1[0], Move::Pass);

    println!("{state}");
    println!("a0: {ac_0:?}");
    println!("a1: {ac_1:?}");
  }

  #[test]
  fn test_qwirkle() {
    let g = Qwirkle::<2>;

    let agent = 0;
    let mut o_seq = g.start(agent);

    let sample_result = g.sample(&o_seq, agent);
    let mut hidden_state = sample_result.state;

    loop {
      println!("observation Seq: \n{o_seq:?}");
      println!("Hidden state: \n{hidden_state}");
      let actions: [Vec<Move>; 2] = [g.actions(&hidden_state, 0), g.actions(&hidden_state, 1)];
      println!("Actions: {:?}", actions);
      if actions[0].is_empty() || actions[1].is_empty() {
        println!("Game over");
        break;
      }

      let current_player = hidden_state.current_player;
      let selected_action = actions[current_player].choose(&mut rand::thread_rng()).unwrap();
      let mut joint_action = [Move::Pass, Move::Pass];
      println!("Selected Action: {selected_action:?}");
      joint_action[current_player] = selected_action.clone();

      let tr = g.transition(&mut hidden_state, &joint_action);
      println!("Transition rewards: {:?}, result: {:?}", tr.rewards, tr.observations);
      g.append(&mut o_seq, current_player, tr.observations[agent].clone());
    }
  }
}
