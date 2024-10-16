use std::marker::PhantomData;

use num_traits::Float;

use crate::{
  node::{DualType, NodeDeref, NodeLink, Step, Trajectory},
  MarkovDecisionProcess, Pomdp,
};

trait TrajectorySampling<Pr, S, E, P, R, ND> {
  fn sample_trajctory(&self, problem: &Pr, state: S, root: P, deref: &mut ND) -> Trajectory<E, P, R>;
}

struct PomdpSampler<N> {
  phantom: PhantomData<N>
}

impl<
    NL: NodeLink,
    P: Pomdp,
    ND: NodeDeref<N, DualType<P::Action, P::Observation>, NL, P::R>,
    N,
  > TrajectorySampling<P, P::ObservationSeq, DualType<P::Action, P::Observation>, NL, P::R, ND>
  for PomdpSampler<N>
where
  P::Action: Ord + Clone,
  P::Observation: Ord,
  NL: Clone,
  P::R: Float,
{
  fn sample_trajctory(
    &self,
    problem: &P,
    mut obs_seq: P::ObservationSeq,
    mut root: NL,
    deref: &mut ND,
  ) -> Trajectory<DualType<P::Action, P::Observation>, NL, P::R> {
    // TODO: handle node creartion
    let mut trajectory = Trajectory::new();

    let mut state = problem.sample_hidden_state(&obs_seq);
    while !problem.is_terminal(&state) && !root.is_nil() {
      let (next_node_link, ei, selected_action) = {
        let node = deref.deref_mut(&root);
        if node.children.is_empty() {
          break;
        }

        let actions = problem.outgoing_edges(&state);
        // todo: select action
        let selected_action = actions[0].clone();

        let ei = DualType::A(selected_action.clone());
        let next_edge = node.children.get_mut(&ei).unwrap();
        next_edge.select_count += 1;
        (next_edge.link.clone(), ei, selected_action)
      };
      trajectory.steps.push(Step {
        node: next_node_link.clone(),
        edge: ei,
        reward: P::R::neg_zero(),
      });

      let (next_node_link, ei, r) = {
        let next_node = deref.deref_mut(&next_node_link);


        let (obs, r) = problem.transition(&mut state, &selected_action);
        let ei = DualType::B(obs);
        let next_edge = next_node.children.get_mut(&ei).unwrap();
        next_edge.select_count += 1;
        (next_edge.link.clone(), ei, r)
      };
      trajectory.steps.push(Step {
        node: next_node_link.clone(),
        edge: ei,
        reward: r,
      });
      root = next_node_link;
    }
    trajectory
  }
}


struct MdpSampler<N> {
  phantom: PhantomData<N>
}

impl<
    NL: NodeLink,
    P: MarkovDecisionProcess,
    ND: NodeDeref<N, DualType<P::Action, P::State>, NL, P::R>,
    N
  > TrajectorySampling<P, P::State, DualType<P::Action, P::State>, NL, P::R, ND>
  for MdpSampler<N>
where
  P::Action: Ord + Clone,
  P::State: Ord + Clone,
  NL: Clone,
  P::R: Float,
{
  fn sample_trajctory(
    &self,
    problem: &P,
    mut state: P::State,
    mut root: NL,
    deref: &mut ND,
  ) -> Trajectory<DualType<P::Action, P::State>, NL, P::R> {
    let mut trajectory = Trajectory::new();
    while !problem.is_terminal(&state) {
      let (next_node_link, ei, selected_action) = {
        let node = deref.deref_mut(&root);
        if node.children.is_empty() {
          break;
        }

        let actions = problem.outgoing_edges(&state);
        // todo: select action
        let selected_action = actions[0].clone();

        let ei = DualType::A(selected_action.clone());
        let next_edge = node.children.get_mut(&ei).unwrap();
        next_edge.select_count += 1;
        (next_edge.link.clone(), ei, selected_action)
      };
      trajectory.steps.push(Step {
        node: next_node_link.clone(),
        edge: ei,
        reward: P::R::neg_zero(),
      });

      let (next_node_link, ei, r) = {
        let next_node = deref.deref_mut(&next_node_link);

        let r = problem.transition(&mut state, &selected_action);
        let ei = DualType::B(state.clone());
        let next_edge = next_node.children.get_mut(&ei).unwrap();
        next_edge.select_count += 1;
        (next_edge.link.clone(), ei, r)
      };
      trajectory.steps.push(Step {
        node: next_node_link.clone(),
        edge: ei,
        reward: r,
      });
      root = next_node_link;
    }
    trajectory
  }
}




#[cfg(test)]
mod test {
  trait T<B> {
    fn f(&self, arg: B);
  }

  struct my_struct<A> {
    data: A
  }

  impl<A, B> T<B> for my_struct<A> {
    fn f(&self, arg: B) {
    }
  }

  #[test] fn t1(){

  }
}