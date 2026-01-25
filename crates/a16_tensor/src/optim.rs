//! Optimizers
//! 
//! Provides SGD and other optimizers for training.

use crate::tensor::Tensor;

/// Trait for optimizers
pub trait Optimizer {
    /// Perform one optimization step
    fn step(&mut self);
    
    /// Zero out all gradients
    fn zero_grad(&mut self);
}

/// Stochastic Gradient Descent optimizer
pub struct SGD {
    params: Vec<Tensor>,
    lr: f32,
}

impl SGD {
    /// Create new SGD optimizer
    pub fn new(params: Vec<Tensor>, lr: f32) -> Self {
        Self { params, lr }
    }
    
    /// Get learning rate
    pub fn lr(&self) -> f32 {
        self.lr
    }
    
    /// Set learning rate
    pub fn set_lr(&mut self, lr: f32) {
        self.lr = lr;
    }
}

impl Optimizer for SGD {
    fn step(&mut self) {
        for param in &self.params {
            if let Some(grad) = param.grad() {
                // param.data = param.data - lr * grad
                let grad_data = grad.data();
                let mut param_data = param.data_mut();
                for i in 0..param_data.len() {
                    param_data[i] -= self.lr * grad_data[i];
                }
            }
        }
    }
    
    fn zero_grad(&mut self) {
        for param in &self.params {
            param.zero_grad();
        }
    }
}
