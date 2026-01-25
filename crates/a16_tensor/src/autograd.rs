//! Autograd - Automatic Differentiation
//! 
//! Provides the GradFn trait for backward pass computation.

use crate::tensor::Tensor;

/// Trait for gradient computation functions
pub trait GradFn {
    /// Compute gradients with respect to inputs given output gradient
    fn backward(&self, grad_output: &Tensor) -> Vec<Tensor>;
    
    /// Get the input tensors that this function depends on
    fn inputs(&self) -> Vec<Tensor>;
}
