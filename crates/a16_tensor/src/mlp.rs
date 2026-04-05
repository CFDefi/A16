//! Multi-Layer Perceptron (MLP)
//!
//! Built-in neural network architecture for A16's native AI runtime.
//! Supports forward inference, backpropagation, and training via SGD.

use crate::tensor::Tensor;
use crate::optim::{SGD, Optimizer};

/// A single fully-connected (dense) layer: y = x @ W^T + b
#[derive(Clone, Debug)]
pub struct LinearLayer {
    /// Weight matrix [out_features, in_features]
    pub weight: Tensor,
    /// Bias vector [out_features]
    pub bias: Tensor,
}

impl LinearLayer {
    /// Create a new linear layer with Xavier/Glorot initialization
    pub fn new(in_features: usize, out_features: usize) -> Self {
        // Xavier initialization: scale = sqrt(2 / (fan_in + fan_out))
        let scale = (2.0 / (in_features + out_features) as f32).sqrt();
        let mut weight = Tensor::randn(&[out_features, in_features]);
        {
            let mut data = weight.data_mut();
            for v in data.iter_mut() {
                *v *= scale;
            }
        }
        weight.set_requires_grad(true);

        let mut bias = Tensor::zeros(&[out_features]);
        bias.set_requires_grad(true);

        Self { weight, bias }
    }

    /// Forward pass: output = input @ W^T + b
    /// input shape: [batch, in_features]
    /// output shape: [batch, out_features]
    pub fn forward(&self, input: &Tensor) -> Tensor {
        let wt = self.weight.transpose();
        let out = input.matmul(&wt);
        out.add(&self.bias)
    }

    /// Get all trainable parameters
    pub fn parameters(&self) -> Vec<Tensor> {
        vec![self.weight.clone(), self.bias.clone()]
    }
}

/// Activation function variants
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Activation {
    ReLU,
    Sigmoid,
    Tanh,
    None,
}

/// Multi-Layer Perceptron
///
/// A feedforward neural network with configurable hidden layers and activations.
///
/// # Example (conceptual)
/// ```ignore
/// let mlp = MLP::new(&[2, 16, 8, 1], Activation::ReLU);
/// let input = Tensor::randn(&[4, 2]);  // batch of 4, 2 features
/// let output = mlp.forward(&input);    // [4, 1]
/// ```
#[derive(Clone, Debug)]
pub struct MLP {
    layers: Vec<LinearLayer>,
    activation: Activation,
}

impl MLP {
    /// Create a new MLP from layer sizes.
    ///
    /// `sizes` defines the width of each layer including input and output.
    /// For example, `&[2, 16, 1]` creates a network with:
    ///   - Input: 2 features
    ///   - Hidden: 16 neurons (with activation)
    ///   - Output: 1 neuron (no activation on final layer)
    pub fn new(sizes: &[usize], activation: Activation) -> Self {
        assert!(sizes.len() >= 2, "MLP requires at least input and output sizes");

        let mut layers = Vec::with_capacity(sizes.len() - 1);
        for i in 0..sizes.len() - 1 {
            layers.push(LinearLayer::new(sizes[i], sizes[i + 1]));
        }

        Self { layers, activation }
    }

    /// Forward pass through all layers.
    /// Applies activation after every hidden layer (not the output layer).
    pub fn forward(&self, input: &Tensor) -> Tensor {
        let mut x = input.clone();

        for (i, layer) in self.layers.iter().enumerate() {
            x = layer.forward(&x);

            // Apply activation to all layers except the last
            if i < self.layers.len() - 1 {
                x = match self.activation {
                    Activation::ReLU => x.relu(),
                    Activation::Sigmoid => x.sigmoid(),
                    Activation::Tanh => x.tanh(),
                    Activation::None => x,
                };
            }
        }

        x
    }

    /// Get all trainable parameters across all layers.
    pub fn parameters(&self) -> Vec<Tensor> {
        self.layers.iter()
            .flat_map(|l| l.parameters())
            .collect()
    }

    /// Number of layers (not counting input)
    pub fn num_layers(&self) -> usize {
        self.layers.len()
    }

    /// Get layer sizes as a description
    pub fn layer_sizes(&self) -> Vec<(usize, usize)> {
        self.layers.iter()
            .map(|l| {
                let shape = l.weight.shape();
                (shape[1], shape[0]) // (in_features, out_features)
            })
            .collect()
    }
}

/// Mean Squared Error loss: MSE = mean((pred - target)^2)
pub fn mse_loss(predictions: &Tensor, targets: &Tensor) -> Tensor {
    let diff = predictions.sub(targets);
    let sq = diff.mul(&diff);
    sq.mean()
}

/// Perform a single training step.
/// Returns the loss value as f32.
pub fn train_step(
    model: &MLP,
    input: &Tensor,
    target: &Tensor,
    optimizer: &mut SGD,
) -> f32 {
    // Zero gradients
    optimizer.zero_grad();

    // Forward pass
    let output = model.forward(input);

    // Compute loss
    let loss = mse_loss(&output, target);
    let loss_val = loss.item();

    // Backward pass
    loss.backward();

    // Update weights
    optimizer.step();

    loss_val
}

#[cfg(test)]
mod mlp_tests {
    use super::*;

    #[test]
    fn test_linear_layer_forward() {
        let layer = LinearLayer::new(3, 2);
        let input = Tensor::ones(&[1, 3]);
        let output = layer.forward(&input);
        assert_eq!(output.shape(), &[1, 2]);
    }

    #[test]
    fn test_mlp_forward_shape() {
        let mlp = MLP::new(&[4, 8, 3], Activation::ReLU);
        let input = Tensor::randn(&[2, 4]); // batch of 2
        let output = mlp.forward(&input);
        assert_eq!(output.shape(), &[2, 3]);
    }

    #[test]
    fn test_mlp_parameters_count() {
        let mlp = MLP::new(&[2, 4, 1], Activation::ReLU);
        let params = mlp.parameters();
        // 2 params per layer (weight + bias), 2 layers = 4
        assert_eq!(params.len(), 4);
    }

    #[test]
    fn test_mse_loss() {
        let pred = Tensor::from_data(vec![1.0, 2.0, 3.0], vec![3]);
        let target = Tensor::from_data(vec![1.0, 2.0, 3.0], vec![3]);
        let loss = mse_loss(&pred, &target);
        assert!((loss.item() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_mse_loss_nonzero() {
        let pred = Tensor::from_data(vec![1.0, 2.0], vec![2]);
        let target = Tensor::from_data(vec![3.0, 4.0], vec![2]);
        let loss = mse_loss(&pred, &target);
        // (1-3)^2 + (2-4)^2 = 4 + 4 = 8, mean = 4.0
        assert!((loss.item() - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_training_runs() {
        // Verify that the full training pipeline executes without panicking
        // and produces finite loss values
        let mlp = MLP::new(&[2, 4, 1], Activation::Sigmoid);
        let mut optimizer = SGD::new(mlp.parameters(), 0.01);

        let inputs = Tensor::from_data(
            vec![0.0, 0.0,  0.5, 0.5,  1.0, 1.0,  0.5, 0.0],
            vec![4, 2],
        );
        let targets = Tensor::from_data(
            vec![0.0, 0.5, 1.0, 0.25],
            vec![4, 1],
        );

        // Run 10 training steps and verify no NaN/Inf
        for _ in 0..10 {
            let loss = train_step(&mlp, &inputs, &targets, &mut optimizer);
            assert!(loss.is_finite(), "Loss must be finite, got {}", loss);
            assert!(loss >= 0.0, "MSE loss must be non-negative, got {}", loss);
        }

        // Forward pass should produce correct shape
        let output = mlp.forward(&inputs);
        assert_eq!(output.shape(), &[4, 1]);
    }
}

