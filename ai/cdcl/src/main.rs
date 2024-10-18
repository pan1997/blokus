use cdcl::{dpll, Clause, SatProblem};

fn main1() {
  let mut p = SatProblem::new();
  p.push(Clause::new([5, 3, -1]));
  p.push(Clause::new([6, -5, 4]));
  p.push(Clause::new([-3, 6, 2]));
  p.push(Clause::new([-5, 4, 6]));
  p.push(Clause::new([4, -1, -5]));
  p.push(Clause::new([1, -5, -4]));
  p.push(Clause::new([4, -2, 1]));
  p.push(Clause::new([1, -3, -5]));
  p.push(Clause::new([6, 4, 2]));
  p.push(Clause::new([6, 3, -2]));
  p.push(Clause::new([-5, 3, 2]));
  p.push(Clause::new([4, -2, 6]));

  let x = dpll(&mut p, 2);
  print!("{x:?}")
}
fn main() {
  let mut p = SatProblem::new();
  p.push(Clause::new([-3, 1, -4]));
  p.push(Clause::new([3, 4, 2]));
  p.push(Clause::new([-3, -2, -5]));
  p.push(Clause::new([5, 6, -1]));
  p.push(Clause::new([5, -1, -2]));
  p.push(Clause::new([5, 4, 6]));
  p.push(Clause::new([-2, 3, 1]));
  p.push(Clause::new([-1, 5, 2]));
  p.push(Clause::new([-3, 2, -5]));
  p.push(Clause::new([-3, -1, 6]));

  let x = dpll(&mut p, 2);
  print!("{x:?}")
}
