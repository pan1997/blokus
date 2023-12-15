use std::{fmt::Display, collections::BTreeMap, cmp::min, cmp::max};
use colored::Colorize;
use rand::{Rng, seq::IteratorRandom};


#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Tile {
    // both shape and color start from 1,
    // and 0, 0 is an invalid tile
    shape: u8,
    color: u8
}

struct State<const N: usize> {
    hands: [[Tile; 6]; N],
    table: BTreeMap<(i16, i16), Tile>,
    bag: BTreeMap<Tile, u8>,
    bag_size: usize,
}




impl<const N: usize> State<N> {
    fn new() -> Self {
        let mut result = State {
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

    fn initialize_hands(&mut self) {
        for player in 0..N {
            let tiles = self.tiles_from_bag(6);
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
        let mut state = State::<4>::new();
        state.initialize_hands();
        println!("{state}");
    }
}