use std::{fmt::Display, collections::BTreeMap, cmp::min, cmp::max};
use ai::{MaPomdp, SampleResult};
use colored::Colorize;
use rand::{Rng, seq::IteratorRandom};


struct Qwirkle<const N: usize>;


#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Tile {
    // both shape and color start from 1,
    // and 0, 0 is an invalid tile
    shape: u8,
    color: u8
}

struct State<const N: usize> {
    current_player: usize,
    // bag.. empty tiles moved to right
    hands: [[Tile; 6]; N],
    table: BTreeMap<(i16, i16), Tile>,
    bag: BTreeMap<Tile, u8>,
    bag_size: usize,
}

struct ObservationSeq {
    player: usize,
    hand: [Tile; 6],
    table: BTreeMap<(i16, i16), Tile>
}



enum Move {
    // all players other than the current player pass
    Pass,
    Placement(Vec<(Tile, i16, i16)>),
    Exchange(Vec<Tile>),
}

struct Observation {
    // all players see the move
    action: Move,
    // only the current player sees the tiles that were picked
    // for other players, this is a vector of None(s)
    pick: Vec<Option<Tile>>
}

impl<const N: usize> MaPomdp<ObservationSeq, [Tile; 6], Observation, State<N>, Move, N> for Qwirkle<N> {
    fn start(&self, agent: usize) -> ObservationSeq {
        // todo: refactor to avoid state creation
        let mut state: State<N> = Default::default();
        let mut hand = [Tile::default(); 6];
        for (ix, tile) in state.tiles_from_bag(6).iter().enumerate() {
            hand[ix] = *tile;
        }
        ObservationSeq { player: agent, hand: hand, table: Default::default() }
    }

    fn actions(&self, state: &State<N>, agent: usize) -> Vec<Move> {
        let mut result = vec![];
        if state.current_player == agent {
            if state.hands[agent][0].shape != 0 {
                // current player has at least one tile
                // placements
                todo!("fill in placement computation");
                // exchanges
                todo!("fill in exchange computation");
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
        unimplemented!()
    }

    fn append(&self, observation_seq: &mut ObservationSeq, agent: usize, obs: Observation) {
        unimplemented!()
    }

    fn transition(
        &self,
        state: &mut State<N>,
        joint_action: &[Move; N],
      ) -> ai::TranstitionResult<Observation, N> {
        unimplemented!()
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
                result.bag.insert(Tile{shape, color}, 3);
            }
        }
        result
    }
}

impl<const N: usize> State<N> {
    fn initialize_hands(&mut self) {
        for player in 0..N {
            let tiles = self.tiles_from_bag(6);
            self.remove_from_bag(&tiles);
            for ix in 0..6 {
                self.hands[player][ix] = tiles[ix]
            }
        }
    }

    fn boundry(&self) -> (i16, i16, i16, i16) {
        self.table.keys().fold((0, 0, 0, 0), |(lx, ly, hx, hy), (x, y)| (min(lx, *x), min(ly, *y), max(hx, *x), max(hy, *y)))
    }

    fn tiles_from_bag(&self, count: usize) -> Vec<Tile> {
        let mut rng = rand::thread_rng();
        let indexes = (0..self.bag_size).choose_multiple(&mut rng, count);
        let tiles: Vec<_> = indexes.iter().map(|index| {
            let mut ix = *index;
            for shape in 1..7 {
                for color in 1..7 {
                    let t = Tile { shape, color};
                    let tix = *self.bag.get(&t).unwrap_or(&0) as usize;
                    if tix > ix {
                        return t;
                    } else {
                        ix -= tix;
                    }
                }
            }
            Tile::default()
        }).collect();
        tiles
    }

    fn remove_from_bag(&mut self, tiles: &Vec<Tile>) {
        for tile in tiles {
            self.bag.insert(*tile, self.bag.get(tile).map(|x| *x).unwrap_or_default() - 1);
        }
    }
}


impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self.shape {
            0 => " ", // nil tile
            1 => "⏺",
            2 => "✦",
            3 => "◆",
            4 => "■",
            5 => "𖤓",
            6 => "𖥔",
            _ => panic!("djdjd")
        };
        let cc = match self.color {
            0 => c.white(),
            1 => c.truecolor(255, 0, 0),
            2 => c.truecolor(255, 187, 51),
            3 => c.truecolor(255, 255, 0),
            4 => c.truecolor(0, 255, 0),
            5 => c.truecolor(0, 0, 255),
            6 => c.truecolor(191, 64, 191),
            _ => panic!("djdjd")
        };
        write!(f,"{cc}")
    }
}

impl<const N: usize> Display for State<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (low_x, low_y, high_x, high_y) = self.boundry();
        for x in (low_x - 3)..(high_x+4) {
            for y in (low_y - 3)..(high_y+4) {
                let t = self.table.get(&(x - low_x, y-low_y)).map(|x| *x).unwrap_or_default();
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
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::State;

    use super::Tile;

    #[test]
    fn test_tile_display() {
        for i in 1..7 {
            for j in 1..7 {
                let t = Tile {
                    shape: j, 
                    color: i
                };
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