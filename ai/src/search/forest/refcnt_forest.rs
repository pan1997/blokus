use std::{
  cell::{Ref, RefCell, RefMut},
  collections::BTreeMap,
  rc::Rc,
};

use crate::search::{forest::ActionInfo, RunningAverage, TreeNode, TreeNodePtr};

struct Node<A, O> {
  visited: bool,
  actions: BTreeMap<A, ActionInfo>,
  // index to children
  children: BTreeMap<O, Rc<RefCell<Node<A, O>>>>,
  value: RunningAverage,
  select_count: u32,
}

impl<A, O> TreeNode<A, O> for Node<A, O>
where
  A: Ord + 'static,
  O: Ord + 'static,
{
  type TreeNodePtr = Rc<RefCell<Node<A, O>>>;
  fn first_visit(&mut self) -> bool {
    if self.visited {
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
  fn expected_value(&self) -> f32 {
    self.value.value()
  }
  fn get_child(&mut self, obs: &O) -> Self::TreeNodePtr {
    self.children[obs].clone()
  }
  fn increment_select_count(&mut self, action: &A) {
    self.select_count += 1;
    self.actions.get_mut(action).map(|a| a.select_count += 1);
  }
  fn select_count(&self) -> u32 {
    self.select_count
  }
  fn actions(&self) -> &BTreeMap<A, ActionInfo> {
    &self.actions
  }
}

// TODO: relax this static
impl<A: 'static, O: 'static> TreeNodePtr<Node<A, O>> for Rc<RefCell<Node<A, O>>> {
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
