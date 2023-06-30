use std::sync::Arc;

use ndarray::Array1;
use ort::{
  tensor::{FromArray, InputTensor, OrtOwnedTensor},
  Environment, ExecutionProvider, SessionBuilder,
};

mod lib;

fn main() {
  tracing_subscriber::fmt::init();
  let environment = Arc::new(
    Environment::builder()
      .with_execution_providers([ExecutionProvider::coreml()])
      .build()
      .unwrap(),
  );

  let session = SessionBuilder::new(&environment)
    .unwrap()
    .with_optimization_level(ort::GraphOptimizationLevel::Level1)
    .unwrap()
    .with_intra_threads(1)
    .unwrap()
    .with_model_from_file("/Users/pankaj/Learning/blokus/ai/nn/py/test.onnx")
    .unwrap();

  let array = Array1::from_iter((1..7).map(|i| i as f32))
    .into_shape((1, 2, 3))
    .unwrap();

  let res = session
    .run([InputTensor::from_array(array.into_dyn())])
    .unwrap();
  let output: OrtOwnedTensor<f32, _> = res[0].try_extract().unwrap();
  let rr = output.view();
  let r = rr.as_slice().unwrap();
  println!("{:?}", r);
  println!("Hello, world!");
}
