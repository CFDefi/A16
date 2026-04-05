//! A16 Tensor Engine with Autograd
//! 
//! Provides tensor operations and automatic differentiation for neural network training.

mod tensor;
mod autograd;
mod optim;
pub mod mlp;

#[cfg(test)]
mod tests;

pub use tensor::Tensor;
pub use autograd::GradFn;
pub use optim::{Optimizer, SGD};
pub use mlp::{MLP, LinearLayer, Activation, mse_loss, train_step};
