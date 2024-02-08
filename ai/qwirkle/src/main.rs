mod lib;

use text_io::read;
use lib::Qwirkle;
use lib::Move;
use rustyai::MaPomdp;

fn main() {
  let game = Qwirkle::<2>;
  let agent = 0;
  let mut obs_seq = game.start(agent);
  let mut hidden_state = game.sample(&obs_seq, agent).state;
  let mut actions = vec![];
  loop {
    let cmd: String = read!();
    match cmd.as_str() {
      "exit" | "quit" => std::process::exit(0),
      "state" => println!("{}", hidden_state),
      "ob_seq" => println!("{:?}", obs_seq),
      "actions" => {
        actions = game.actions(&hidden_state, hidden_state.current_player());
        for ix in 0..actions.len() {
          println!("{ix} -> {:?}", actions[ix]);
        }
      },
      "move" => {
        let m: usize = read!();
        let mut joint_action = [Move::Pass, Move::Pass];
        joint_action[hidden_state.current_player()] = actions[m].clone();
        let tr = game.transition(&mut hidden_state, &joint_action);
        println!("rewards: {:?}", tr.rewards);
        println!("observations: {:?}", tr.observations);
      }
      _ => println!("Invalid command")
    }
  }
  println!("Hello, world!");
}
