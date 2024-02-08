mod lib;


use lib::Qwirkle;
use rustyai::MaPomdp;

fn main() {
  let game = Qwirkle::<2>;
  let agent = 0;
  let mut obs_seq = game.start(agent);
  let mut hidden_state = game.sample(&obs_seq, agent);

  println!("Hello, world!");
}
