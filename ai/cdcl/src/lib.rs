use std::{
  collections::{BTreeMap, BTreeSet},
  fmt::{Debug, Display},
};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Literal(i64);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Var(i64);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Clause(Vec<Literal>);

impl Literal {
  fn neg(&self) -> Self {
    Literal(-self.0)
  }

  fn var(&self) -> Var {
    Var(self.0.abs())
  }

  fn t() -> Self {
    Literal(0)
  }
}

impl Clause {
  fn remove(&mut self, literal: &Literal) {
    if let Some(index) = self.0.iter().position(|l| l == literal) {
      self.0.swap_remove(index);
    }
  }
  pub fn new<const N: usize>(literals: [i64; N]) -> Self {
    Clause(literals.map(|l| Literal(l)).to_vec())
  }
}

impl Debug for Clause {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self.0)
  }
}
impl Debug for Literal {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self.0)
  }
}

impl Debug for Var {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self.0)
  }
}

#[derive(Clone)]
pub struct SatProblem {
  vars: BTreeSet<Var>,
  // inverted index of the clauses
  map: BTreeMap<Literal, BTreeSet<usize>>,
  clauses: Vec<Clause>,
}

impl SatProblem {
  pub fn new() -> Self {
    Self {
      vars: BTreeSet::new(),
      map: BTreeMap::new(),
      clauses: vec![],
    }
  }

  pub fn push(&mut self, clause: Clause) -> usize {
    let index = self.clauses.len();
    for literal in clause.0.iter() {
      // update vars
      self.vars.insert(literal.var());

      // update map
      if !self.map.contains_key(&literal) {
        self.map.insert(*literal, BTreeSet::new());
      }
      self.map.get_mut(literal).unwrap().insert(index);
    }
    // push to clauses
    self.clauses.push(clause);
    index
  }

  fn remove_clause(&mut self, index: usize) {
    for literal in self.clauses[index].0.iter() {
      // update map (if literal is alive)
      if let Some(m) = self.map.get_mut(literal) {
        m.remove(&index);

        // remove m if empty
        if m.is_empty() {
          self.map.remove(literal);

          // if both literal and not literal have no clauses, update vars
          if !self.map.contains_key(&literal.neg()) {
            self.vars.remove(&literal.var());
          }
        }
      }
    }
  }
}

fn forward_checking_assign(
  p: &mut SatProblem,
  literal: &Literal,
) -> Result<(Vec<Literal>, Vec<Literal>), ()> {
  let mut units = vec![];
  let mut no_neg = vec![];

  if p.vars.contains(&literal.var()) {
    // drop all mentions of neg_literal
    let neg = literal.neg();
    if p.map.contains_key(&neg) {
      for clause_index in p.map[&neg].iter() {
        p.clauses[*clause_index].remove(&neg);
        match p.clauses[*clause_index].0.len() {
          0 => {
            // both l and not l need to be true
            return Err(());
          }
          1 => {
            // unit clause found
            units.push(p.clauses[*clause_index].0[0]);
          }
          _ => {
            // do nothing
          }
        }
      }
      p.map.remove(&neg);
    }

    // remove all clauses that are positive
    let positive_map = p.map[literal].clone();
    for clause_index in positive_map {
      p.remove_clause(clause_index);
      for pliteral in p.clauses[clause_index].0.iter() {
        if literal != pliteral && !p.map.contains_key(pliteral) {
          no_neg.push(pliteral.neg());
        }
      }
      p.clauses[clause_index].0.clear();
      p.clauses[clause_index].0.push(Literal::t());
    }

    Ok((units, no_neg))
  } else {
    Err(())
  }
}

pub fn dpll(problem: &mut SatProblem, d: usize) -> Option<Vec<Literal>> {
  if d == 0 {
    return None;
  }
  let mut assignments = vec![];

  let mut open_units = vec![];
  let mut open_no_neg = vec![];
  for clause_index in 0..problem.clauses.len() {
    match problem.clauses[clause_index].0.len() {
      1 => {
        if problem.clauses[clause_index].0[0].0 != 0 {
          open_units.push(problem.clauses[clause_index].0[0]);
        }
      }
      0 => {
        return None;
      }
      _ => {}
    }
  }
  for literal in problem.map.keys() {
    if !problem.map.contains_key(&literal.neg()) {
      open_no_neg.push(*literal);
    }
  }

  while !problem.vars.is_empty() {
    println!(
      "problem: {:?} (vars: {:?}, map: {:?})\nunits: {open_units:?}\nno_neg: {open_no_neg:?}",
      problem.clauses, problem.vars, problem.map
    );

    let l2s = if !open_units.is_empty() {
      let h = open_units[0];
      open_units.swap_remove(0);
      println!("select unit: {h:?}");
      Some(h)
    } else if !open_no_neg.is_empty() {
      let h = open_no_neg[0];
      open_no_neg.swap_remove(0);
      println!("select uno_neg: {h:?}");
      Some(h)
    } else {
      None
    };

    if l2s.is_some() {
      let literal = l2s.unwrap();
      if !problem.vars.contains(&literal.var()) {
        println!("no-op: {literal:?}");
        continue;
      }
      assignments.push(literal);
      let r = forward_checking_assign(problem, &literal);
      if r.is_err() {
        return None;
      }
      r.ok().map(|(mut ou, mut onn)| {
        open_units.append(&mut ou);
        open_no_neg.append(&mut onn);
      });
    } else {
      let l = problem.map.keys().next().unwrap();
      println!("Branching {l:?}");
      let mut cloned_p = problem.clone();
      let res = forward_checking_assign(&mut cloned_p, l);
      if res.is_ok() {
        let full = dpll(&mut cloned_p, d - 1);
        if full.is_some() {
          assignments.push(*l);
          assignments.append(&mut full.unwrap());
          return Some(assignments);
        }
      }
      println!("Backtracking to {:?}", l.neg());
      assignments.push(l.neg());
      let r = forward_checking_assign(problem, &l.neg());
      if r.is_err() {
        return None;
      }
      r.ok().map(|(mut ou, mut onn)| {
        open_units.append(&mut ou);
        open_no_neg.append(&mut onn);
      });
    }
  }
  Some(assignments)
}

#[cfg(test)]
mod test {
  use crate::{dpll, Clause, SatProblem};

  #[test]
  fn test_fc() {
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

    let x = dpll(&mut p, 4);
    print!("{x:?}")
  }
}
