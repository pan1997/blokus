use std::{marker::PhantomData, ops::AddAssign};

use num_traits::Float;

use crate::{
  node::{DualType, Edge, NodeLink, NodeStore, Step, Trajectory},
  util::RunningAverage,
  MarkovDecisionProcess, Pomdp,
};

pub trait TrajectorySampling<Pr, S, E, P, R, ND, HS> {
  fn sample_trajctory(
    &self,
    problem: &Pr,
    state: S,
    root: P,
    node_store: &mut ND,
  ) -> (Trajectory<E, P, R>, HS);
}

pub struct PomdpSampler;

impl<
    NL: NodeLink,
    P: Pomdp,
    ND: NodeStore<(), DualType<P::Action, P::Observation>, NL, P::R, P::ObservationSeq>,
  >
  TrajectorySampling<
    P,
    P::ObservationSeq,
    DualType<P::Action, P::Observation>,
    NL,
    P::R,
    ND,
    P::HiddenState,
  > for PomdpSampler
where
  P::Action: Ord + Clone,
  P::Observation: Ord + Clone,
  P::ObservationSeq: Clone,
  NL: Clone,
  P::R: Float + AddAssign,
{
  fn sample_trajctory(
    &self,
    problem: &P,
    mut obs_seq: P::ObservationSeq,
    mut root: NL,
    node_store: &mut ND,
  ) -> (
    Trajectory<DualType<P::Action, P::Observation>, NL, P::R>,
    P::HiddenState,
  ) {
    assert!(!root.is_nil(), "Cannot start with a nil tree");
    let mut trajectory = Trajectory::new();

    let mut state = problem.sample_hidden_state(&obs_seq);
    //while !problem.is_terminal(&state) && !root.is_nil() {
    loop {
      let actions = problem.outgoing_edges(&state);
      // todo: select action
      let selected_action = actions[0].clone();
      let edge_index = DualType::A(selected_action.clone());
      let (next_node_link, _) = descend(node_store, &root, &edge_index, None, true);
      trajectory.steps.push(Step {
        node: root.clone(),
        edge: edge_index,
        reward: P::R::neg_zero(),
      });

      let (obs, r) = problem.transition(&mut state, &selected_action);
      problem.update_observation_seq(&mut obs_seq, &obs);
      let edge_index = DualType::B(obs);
      let (next_root_link, is_new) = descend(node_store, &next_node_link, &edge_index, Some(&obs_seq), true);
      if is_new {
        trajectory.last_node = next_root_link;
        return (trajectory, state);
      }
      trajectory.steps.push(Step {
        node: next_node_link.clone(),
        edge: edge_index,
        reward: r,
      });
      root = next_root_link;
    }
  }
}

pub struct MdpSampler {}

impl<
    NL: NodeLink,
    P: MarkovDecisionProcess,
    ND: NodeStore<(), DualType<P::Action, P::State>, NL, P::R, P::State>,
  > TrajectorySampling<P, P::State, DualType<P::Action, P::State>, NL, P::R, ND, P::State>
  for MdpSampler
where
  P::Action: Ord + Clone,
  P::State: Ord + Clone,
  NL: Clone,
  P::R: Float + AddAssign,
{
  fn sample_trajctory(
    &self,
    problem: &P,
    mut state: P::State,
    mut root: NL,
    node_store: &mut ND,
  ) -> (
    Trajectory<DualType<P::Action, P::State>, NL, P::R>,
    P::State,
  ) {
    assert!(!root.is_nil(), "Cannot start with a nil tree");
    let mut trajectory = Trajectory::new();
    //while !problem.is_terminal(&state) && !root.is_nil() {
    loop {

      let actions = problem.outgoing_edges(&state);
      // todo: select action
      let selected_action = actions[0].clone();
      let edge_index = DualType::A(selected_action.clone());
      let (next_node_link, _) = descend(node_store, &root, &edge_index, None, true);
      trajectory.steps.push(Step {
        node: root.clone(),
        edge: edge_index,
        reward: P::R::neg_zero(),
      });

      let r = problem.transition(&mut state, &selected_action);
      let edge_index = DualType::B(state.clone());
      let (next_root_link, is_new) = descend(node_store, &next_node_link, &edge_index, Some(&state), true);
      if is_new {
        trajectory.last_node = next_root_link;
        return (trajectory, state);
      }
      trajectory.steps.push(Step {
        node: next_node_link.clone(),
        edge: edge_index,
        reward: r,
      });
      root = next_root_link;
    }
  }
}

fn descend<
  NS: NodeStore<(), E, P, R, K>,
  E: Ord + Clone,
  P: NodeLink + Clone,
  R: Float + AddAssign,
  K
>(
  node_store: &mut NS,
  node_link: &P,
  child_key: &E,
  node_key: Option<&K>,
  create_edge_if_missing: bool,
) -> (P, bool) {
  
  if node_link.is_nil() {
    // return nil if node itself is nil
    return (P::nil(), false);
  }

  let node = node_store.deref_mut(&node_link);
  node.select_count += 1;
  if !node.children.contains_key(child_key) {
    if !create_edge_if_missing {
      return (P::nil(), false);
    }
    // insert edge if not exist
    node.children.insert(
      child_key.clone(),
      Edge {
        select_count: 0,
        value: RunningAverage::new(),
        link: P::nil(),
      },
    );
  }
  let edge = node.children.get_mut(child_key).unwrap();
  edge.select_count += 1;
  if !edge.link.is_nil() {
    return (edge.link.clone(), false);
  }
  let new_node = node_store.new_node((), node_key);
  let node = node_store.deref_mut(&node_link);
  let edge = node.children.get_mut(child_key).unwrap();
  edge.link = new_node;
  (edge.link.clone(), true)
}
