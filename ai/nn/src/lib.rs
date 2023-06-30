use std::{marker::PhantomData, sync::Arc};

use ai::{
  search::eval::{BaseEval, EvaluationResult},
  MaMdp,
};
use ndarray::ArrayD;
use ort::{Environment, Session, tensor::{InputTensor, FromArray, OrtOwnedTensor}};

trait AlphaZeroProblem<State, Action, Observation, const N: usize>:
  MaMdp<State, Action, Observation, N>
{
  fn map_to_array(&self, state: &State) -> ArrayD<f32>;
}

struct AlphaZeroEval<Observation> {
  environent: Arc<Environment>,
  session: Session,
  phantom: PhantomData<Observation>,
}

impl<M, State, Action, Observation, const N: usize> BaseEval<M, State, Action, N>
  for AlphaZeroEval<Observation>
where
  M: AlphaZeroProblem<State, Action, Observation, N>,
{
  fn evaluate<'a>(&self, problem: &M, state: &'a mut State) -> EvaluationResult<'a, Action, N> {
    let input = problem.map_to_array(state);
    let input_tensor = InputTensor::from_array(input);

    let res = self.session.run([input_tensor]).unwrap();
    let values_tensor: OrtOwnedTensor<f32, _> = res[0].try_extract().unwrap();
    let values_vec: Vec<_> = values_tensor.view().as_slice().unwrap().iter().map(|x| *x).collect();
    //let policy = ?

    EvaluationResult {
      values: values_vec.try_into().unwrap(),
      policies: [(); N].map(|_| vec![]),
    }
  }
}
