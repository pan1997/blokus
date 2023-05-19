use std::{
  cell::{Ref, RefCell, RefMut},
  collections::BTreeMap,
  rc::Rc,
};

use crate::search::{forest::ActionInfo, RunningAverage, TreeNode, TreeNodePtr};

pub struct Node<A, O> {
  visited: bool,
  actions: BTreeMap<A, ActionInfo>,
  // index to children
  children: BTreeMap<O, Rc<RefCell<Self>>>,
  value: RunningAverage,
  select_count: u32,
}

impl<A, O> TreeNode<A, O> for Node<A, O>
where
  A: Ord + 'static,
  O: Ord + 'static + Clone,
{
  type TreeNodePtr = Rc<RefCell<Self>>;
  fn first_visit(&mut self) -> bool {
    if !self.visited {
      self.visited = true;
      true
    } else {
      false
    }
  }
  fn add_action_sample(&mut self, action: &A, reward: f32) {
    self
      .actions
      .get_mut(action)
      .unwrap()
      .action_reward
      .add_sample(reward, 1)
  }
  
  fn get_child(&mut self, obs: &O) -> Self::TreeNodePtr {
    if !self.children.contains_key(obs) {
      self.children.insert(obs.clone(), Default::default());
    }
    self.children[obs].clone()
  }
  fn increment_select_count(&mut self, action: &A) {
    self.select_count += 1;
    //self.actions.get_mut(action).map(|a| a.select_count += 1);
    self.actions.get_mut(action).unwrap().select_count += 1;
  }
  fn select_count(&self) -> u32 {
    self.select_count
  }
  fn actions(&self) -> &BTreeMap<A, ActionInfo> {
    &self.actions
  }

  fn actions_mut(&mut self) -> &mut BTreeMap<A, ActionInfo> {
    &mut self.actions
  }

  fn children(&self) -> &BTreeMap<O, Self::TreeNodePtr> {
    &self.children
  }
  fn value(&self) -> &RunningAverage {
      &self.value
  }
  fn value_mut(&mut self) -> &mut RunningAverage {
      &mut self.value
  }
}

// TODO: relax this static
impl<A: 'static + Ord, O: 'static + Ord + Clone> TreeNodePtr<A, O> for Rc<RefCell<Node<A, O>>> {
  type TreeNode = Node<A, O>;
  type Guard<'a> = RefMut<'a, Node<A, O>>;
  fn lock<'a, 'b>(&'b self) -> Self::Guard<'a>
  where
    'b: 'a,
  {
    self.borrow_mut()
  }
}

impl<A, O> Default for Node<A, O> {
  fn default() -> Self {
    Node {
      visited: false,
      actions: BTreeMap::new(),
      children: BTreeMap::new(),
      value: RunningAverage::new(),
      select_count: 0,
    }
  }
}

impl<A, O> Node<A, O> {
  pub fn new() -> Rc<RefCell<Node<A, O>>> {
    Default::default()
  }
}
