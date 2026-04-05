//! Tensor Implementation
//! 
//! N-dimensional array with automatic differentiation support.

use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;
use rand::Rng;
use crate::autograd::GradFn;

/// N-dimensional tensor with optional gradient tracking
#[derive(Clone)]
pub struct Tensor {
    /// Raw data storage
    data: Rc<RefCell<Vec<f32>>>,
    /// Shape of the tensor
    shape: Vec<usize>,
    /// Strides for indexing (reserved for strided views)
    #[allow(dead_code)]
    strides: Vec<usize>,
    /// Whether to track gradients
    requires_grad: bool,
    /// Accumulated gradient
    grad: Rc<RefCell<Option<Tensor>>>,
    /// Function that produced this tensor (for backprop)
    grad_fn: Rc<RefCell<Option<Box<dyn GradFn>>>>,
}

impl Tensor {
    // === Creation Methods ===
    
    /// Create a tensor filled with zeros
    pub fn zeros(shape: &[usize]) -> Self {
        let size = shape.iter().product();
        Self::from_data(vec![0.0; size], shape.to_vec())
    }
    
    /// Create a tensor filled with ones
    pub fn ones(shape: &[usize]) -> Self {
        let size = shape.iter().product();
        Self::from_data(vec![1.0; size], shape.to_vec())
    }
    
    /// Create a tensor with random normal values
    pub fn randn(shape: &[usize]) -> Self {
        let size: usize = shape.iter().product();
        let mut rng = rand::thread_rng();
        let data: Vec<f32> = (0..size)
            .map(|_| {
                // Box-Muller transform for normal distribution
                let u1: f32 = rng.gen();
                let u2: f32 = rng.gen();
                (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos()
            })
            .collect();
        Self::from_data(data, shape.to_vec())
    }
    
    /// Create a tensor from existing data
    pub fn from_data(data: Vec<f32>, shape: Vec<usize>) -> Self {
        let strides = Self::compute_strides(&shape);
        Self {
            data: Rc::new(RefCell::new(data)),
            shape,
            strides,
            requires_grad: false,
            grad: Rc::new(RefCell::new(None)),
            grad_fn: Rc::new(RefCell::new(None)),
        }
    }
    
    /// Create from a 2D array (list of lists)
    pub fn from_list(data: &[&[f32]]) -> Self {
        let rows = data.len();
        let cols = if rows > 0 { data[0].len() } else { 0 };
        let flat: Vec<f32> = data.iter().flat_map(|row| row.iter().copied()).collect();
        Self::from_data(flat, vec![rows, cols])
    }
    
    /// Create a scalar tensor
    pub fn scalar(value: f32) -> Self {
        Self::from_data(vec![value], vec![])
    }
    
    fn compute_strides(shape: &[usize]) -> Vec<usize> {
        let mut strides = vec![1; shape.len()];
        for i in (0..shape.len().saturating_sub(1)).rev() {
            strides[i] = strides[i + 1] * shape[i + 1];
        }
        strides
    }
    
    // === Properties ===
    
    pub fn shape(&self) -> &[usize] {
        &self.shape
    }
    
    pub fn size(&self) -> usize {
        self.shape.iter().product()
    }
    
    pub fn ndim(&self) -> usize {
        self.shape.len()
    }
    
    pub fn requires_grad(&self) -> bool {
        self.requires_grad
    }
    
    pub fn set_requires_grad(&mut self, val: bool) {
        self.requires_grad = val;
    }
    
    /// Get raw data as slice
    pub fn data(&self) -> std::cell::Ref<'_, Vec<f32>> {
        self.data.borrow()
    }
    
    /// Get mutable data
    pub fn data_mut(&self) -> std::cell::RefMut<'_, Vec<f32>> {
        self.data.borrow_mut()
    }
    
    /// Get scalar value (panics if not scalar)
    pub fn item(&self) -> f32 {
        assert!(self.size() == 1, "item() requires scalar tensor");
        self.data.borrow()[0]
    }
    
    /// Get gradient
    pub fn grad(&self) -> Option<Tensor> {
        self.grad.borrow().clone()
    }
    
    // === Operations ===
    
    /// Element-wise addition
    pub fn add(&self, other: &Tensor) -> Tensor {
        self.binary_op(other, |a, b| a + b, "add")
    }
    
    /// Element-wise subtraction
    pub fn sub(&self, other: &Tensor) -> Tensor {
        self.binary_op(other, |a, b| a - b, "sub")
    }
    
    /// Element-wise multiplication
    pub fn mul(&self, other: &Tensor) -> Tensor {
        self.binary_op(other, |a, b| a * b, "mul")
    }
    
    /// Element-wise division
    pub fn div(&self, other: &Tensor) -> Tensor {
        self.binary_op(other, |a, b| a / b, "div")
    }
    
    /// Element-wise power
    pub fn pow(&self, exp: f32) -> Tensor {
        let data: Vec<f32> = self.data.borrow().iter().map(|x| x.powf(exp)).collect();
        let mut result = Tensor::from_data(data, self.shape.clone());
        
        if self.requires_grad {
            result.requires_grad = true;
            let self_clone = self.clone();
            result.grad_fn = Rc::new(RefCell::new(Some(Box::new(PowBackward { 
                input: self_clone, 
                exp 
            }))));
        }
        
        result
    }
    
    /// Matrix multiplication
    pub fn matmul(&self, other: &Tensor) -> Tensor {
        assert!(self.ndim() >= 1 && other.ndim() >= 1, "matmul requires at least 1D tensors");
        
        // Handle different dimensions
        let (m, k1) = if self.ndim() == 1 {
            (1, self.shape[0])
        } else {
            (self.shape[self.ndim() - 2], self.shape[self.ndim() - 1])
        };
        
        let (k2, n) = if other.ndim() == 1 {
            (other.shape[0], 1)
        } else {
            (other.shape[other.ndim() - 2], other.shape[other.ndim() - 1])
        };
        
        assert_eq!(k1, k2, "matmul dimension mismatch: {} vs {}", k1, k2);
        
        // Simple 2D matmul
        let mut result_data = vec![0.0; m * n];
        let a = self.data.borrow();
        let b = other.data.borrow();
        
        for i in 0..m {
            for j in 0..n {
                let mut sum = 0.0;
                for k in 0..k1 {
                    sum += a[i * k1 + k] * b[k * n + j];
                }
                result_data[i * n + j] = sum;
            }
        }
        
        let result_shape = if self.ndim() == 1 && other.ndim() == 1 {
            vec![]  // scalar
        } else if self.ndim() == 1 {
            vec![n]
        } else if other.ndim() == 1 {
            vec![m]
        } else {
            vec![m, n]
        };
        
        let mut result = Tensor::from_data(result_data, result_shape);
        
        if self.requires_grad || other.requires_grad {
            result.requires_grad = true;
            let self_clone = self.clone();
            let other_clone = other.clone();
            result.grad_fn = Rc::new(RefCell::new(Some(Box::new(MatMulBackward {
                left: self_clone,
                right: other_clone,
            }))));
        }
        
        result
    }
    
    /// Transpose (swap last two dimensions)
    pub fn transpose(&self) -> Tensor {
        if self.ndim() < 2 {
            return self.clone();
        }
        
        let m = self.shape[self.ndim() - 2];
        let n = self.shape[self.ndim() - 1];
        
        let mut result_data = vec![0.0; m * n];
        let data = self.data.borrow();
        
        for i in 0..m {
            for j in 0..n {
                result_data[j * m + i] = data[i * n + j];
            }
        }
        
        Tensor::from_data(result_data, vec![n, m])
    }
    
    /// ReLU activation
    pub fn relu(&self) -> Tensor {
        let data: Vec<f32> = self.data.borrow().iter().map(|x| x.max(0.0)).collect();
        let mut result = Tensor::from_data(data, self.shape.clone());
        
        if self.requires_grad {
            result.requires_grad = true;
            let self_clone = self.clone();
            result.grad_fn = Rc::new(RefCell::new(Some(Box::new(ReluBackward { 
                input: self_clone 
            }))));
        }
        
        result
    }
    
    /// Sigmoid activation
    pub fn sigmoid(&self) -> Tensor {
        let data: Vec<f32> = self.data.borrow()
            .iter()
            .map(|x| 1.0 / (1.0 + (-x).exp()))
            .collect();
        let mut result = Tensor::from_data(data, self.shape.clone());
        
        if self.requires_grad {
            result.requires_grad = true;
            let output_clone = result.clone();
            result.grad_fn = Rc::new(RefCell::new(Some(Box::new(SigmoidBackward { 
                output: output_clone 
            }))));
        }
        
        result
    }
    
    /// Tanh activation
    pub fn tanh(&self) -> Tensor {
        let data: Vec<f32> = self.data.borrow().iter().map(|x| x.tanh()).collect();
        let mut result = Tensor::from_data(data, self.shape.clone());
        
        if self.requires_grad {
            result.requires_grad = true;
            let output_clone = result.clone();
            result.grad_fn = Rc::new(RefCell::new(Some(Box::new(TanhBackward { 
                output: output_clone 
            }))));
        }
        
        result
    }
    
    /// Mean of all elements
    pub fn mean(&self) -> Tensor {
        let sum: f32 = self.data.borrow().iter().sum();
        let mean = sum / self.size() as f32;
        let mut result = Tensor::scalar(mean);
        
        if self.requires_grad {
            result.requires_grad = true;
            let input = self.clone();
            result.grad_fn = Rc::new(RefCell::new(Some(Box::new(MeanBackward { 
                input,
            }))));
        }
        
        result
    }
    
    /// Sum of all elements
    pub fn sum(&self) -> Tensor {
        let sum: f32 = self.data.borrow().iter().sum();
        let mut result = Tensor::scalar(sum);
        
        if self.requires_grad {
            result.requires_grad = true;
            let input = self.clone();
            result.grad_fn = Rc::new(RefCell::new(Some(Box::new(SumBackward { input }))));
        }
        
        result
    }
    
    // === Autograd ===
    
    /// Compute gradients via backpropagation
    pub fn backward(&self) {
        // Start with gradient of 1 for the output
        let grad = Tensor::ones(&self.shape);
        self.backward_with_grad(&grad);
    }
    
    fn backward_with_grad(&self, grad: &Tensor) {
        // Accumulate gradient
        {
            let mut self_grad = self.grad.borrow_mut();
            if let Some(ref existing) = *self_grad {
                *self_grad = Some(existing.add(grad));
            } else {
                *self_grad = Some(grad.clone());
            }
        }
        
        // Propagate to inputs
        if let Some(ref grad_fn) = *self.grad_fn.borrow() {
            let input_grads = grad_fn.backward(grad);
            for (input, input_grad) in grad_fn.inputs().iter().zip(input_grads.iter()) {
                input.backward_with_grad(input_grad);
            }
        }
    }
    
    /// Zero out gradients
    pub fn zero_grad(&self) {
        *self.grad.borrow_mut() = None;
    }
    
    // === Helpers ===
    
    fn binary_op<F>(&self, other: &Tensor, op: F, name: &str) -> Tensor 
    where F: Fn(f32, f32) -> f32
    {
        // Simple element-wise (assumes same shape or broadcastable)
        let a = self.data.borrow();
        let b = other.data.borrow();
        
        let result_data: Vec<f32> = if a.len() == b.len() {
            a.iter().zip(b.iter()).map(|(&x, &y)| op(x, y)).collect()
        } else if b.len() == 1 {
            // Scalar broadcast
            a.iter().map(|&x| op(x, b[0])).collect()
        } else if a.len() == 1 {
            b.iter().map(|&y| op(a[0], y)).collect()
        } else if self.ndim() == 2 && other.ndim() == 1 && self.shape[1] == other.shape[0] {
            // Row-wise broadcast: [M, N] op [N] -> [M, N]
            let n = other.shape[0];
            a.iter().enumerate().map(|(i, &x)| op(x, b[i % n])).collect()
        } else if self.ndim() == 1 && other.ndim() == 2 && self.shape[0] == other.shape[1] {
            // Row-wise broadcast (reversed): [N] op [M, N] -> [M, N]
            let n = self.shape[0];
            b.iter().enumerate().map(|(i, &y)| op(a[i % n], y)).collect()
        } else {
            panic!("Shape mismatch in {}: {:?} vs {:?}", name, self.shape, other.shape);
        };
        
        let result_shape = if self.shape.len() >= other.shape.len() {
            self.shape.clone()
        } else {
            other.shape.clone()
        };
        
        let mut result = Tensor::from_data(result_data, result_shape);
        
        if self.requires_grad || other.requires_grad {
            result.requires_grad = true;
            // Set up grad_fn based on operation
            let self_clone = self.clone();
            let other_clone = other.clone();
            let grad_fn: Box<dyn GradFn> = match name {
                "add" => Box::new(AddBackward { left: self_clone, right: other_clone }),
                "sub" => Box::new(SubBackward { left: self_clone, right: other_clone }),
                "mul" => Box::new(MulBackward { left: self_clone, right: other_clone }),
                "div" => Box::new(DivBackward { left: self_clone, right: other_clone }),
                _ => panic!("Unknown op: {}", name),
            };
            result.grad_fn = Rc::new(RefCell::new(Some(grad_fn)));
        }
        
        result
    }
}

impl fmt::Debug for Tensor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tensor(shape={:?}, requires_grad={})", self.shape, self.requires_grad)
    }
}

// === Backward Functions ===

struct AddBackward {
    left: Tensor,
    right: Tensor,
}

impl GradFn for AddBackward {
    fn backward(&self, grad: &Tensor) -> Vec<Tensor> {
        vec![grad.clone(), grad.clone()]
    }
    
    fn inputs(&self) -> Vec<Tensor> {
        vec![self.left.clone(), self.right.clone()]
    }
}

struct SubBackward {
    left: Tensor,
    right: Tensor,
}

impl GradFn for SubBackward {
    fn backward(&self, grad: &Tensor) -> Vec<Tensor> {
        let neg_grad = Tensor::from_data(
            grad.data().iter().map(|x| -x).collect(),
            grad.shape().to_vec()
        );
        vec![grad.clone(), neg_grad]
    }
    
    fn inputs(&self) -> Vec<Tensor> {
        vec![self.left.clone(), self.right.clone()]
    }
}

struct MulBackward {
    left: Tensor,
    right: Tensor,
}

impl GradFn for MulBackward {
    fn backward(&self, grad: &Tensor) -> Vec<Tensor> {
        // d/dx(x*y) = y, d/dy(x*y) = x
        let grad_left = grad.mul(&self.right);
        let grad_right = grad.mul(&self.left);
        vec![grad_left, grad_right]
    }
    
    fn inputs(&self) -> Vec<Tensor> {
        vec![self.left.clone(), self.right.clone()]
    }
}

struct DivBackward {
    left: Tensor,
    right: Tensor,
}

impl GradFn for DivBackward {
    fn backward(&self, grad: &Tensor) -> Vec<Tensor> {
        // d/dx(x/y) = 1/y
        // d/dy(x/y) = -x/y^2
        let ones = Tensor::ones(self.right.shape());
        let grad_left = grad.mul(&ones.div(&self.right));
        
        let right_sq = self.right.mul(&self.right);
        let neg_left = Tensor::from_data(
            self.left.data().iter().map(|x| -x).collect(),
            self.left.shape().to_vec()
        );
        let grad_right = grad.mul(&neg_left.div(&right_sq));
        
        vec![grad_left, grad_right]
    }
    
    fn inputs(&self) -> Vec<Tensor> {
        vec![self.left.clone(), self.right.clone()]
    }
}

struct MatMulBackward {
    left: Tensor,
    right: Tensor,
}

impl GradFn for MatMulBackward {
    fn backward(&self, grad: &Tensor) -> Vec<Tensor> {
        // d/dA(A @ B) = grad @ B^T
        // d/dB(A @ B) = A^T @ grad
        let grad_left = grad.matmul(&self.right.transpose());
        let grad_right = self.left.transpose().matmul(grad);
        vec![grad_left, grad_right]
    }
    
    fn inputs(&self) -> Vec<Tensor> {
        vec![self.left.clone(), self.right.clone()]
    }
}

struct ReluBackward {
    input: Tensor,
}

impl GradFn for ReluBackward {
    fn backward(&self, grad: &Tensor) -> Vec<Tensor> {
        // d/dx(relu(x)) = 1 if x > 0 else 0
        let mask: Vec<f32> = self.input.data()
            .iter()
            .map(|x| if *x > 0.0 { 1.0 } else { 0.0 })
            .collect();
        let mask_tensor = Tensor::from_data(mask, self.input.shape().to_vec());
        vec![grad.mul(&mask_tensor)]
    }
    
    fn inputs(&self) -> Vec<Tensor> {
        vec![self.input.clone()]
    }
}

struct SigmoidBackward {
    output: Tensor,
}

impl GradFn for SigmoidBackward {
    fn backward(&self, grad: &Tensor) -> Vec<Tensor> {
        // d/dx(sigmoid(x)) = sigmoid(x) * (1 - sigmoid(x))
        let ones = Tensor::ones(self.output.shape());
        let one_minus = ones.sub(&self.output);
        let local_grad = self.output.mul(&one_minus);
        vec![grad.mul(&local_grad)]
    }
    
    fn inputs(&self) -> Vec<Tensor> {
        vec![self.output.clone()]
    }
}

struct TanhBackward {
    output: Tensor,
}

impl GradFn for TanhBackward {
    fn backward(&self, grad: &Tensor) -> Vec<Tensor> {
        // d/dx(tanh(x)) = 1 - tanh(x)^2
        let sq = self.output.mul(&self.output);
        let ones = Tensor::ones(self.output.shape());
        let local_grad = ones.sub(&sq);
        vec![grad.mul(&local_grad)]
    }
    
    fn inputs(&self) -> Vec<Tensor> {
        vec![self.output.clone()]
    }
}

struct MeanBackward {
    input: Tensor,
}

impl GradFn for MeanBackward {
    fn backward(&self, grad: &Tensor) -> Vec<Tensor> {
        // d/dx(mean(x)) = 1/n for all elements
        let size = self.input.size();
        let scale = 1.0 / size as f32;
        let grad_val = grad.item() * scale;
        vec![Tensor::from_data(vec![grad_val; size], self.input.shape().to_vec())]
    }
    
    fn inputs(&self) -> Vec<Tensor> {
        vec![self.input.clone()]
    }
}

struct SumBackward {
    input: Tensor,
}

impl GradFn for SumBackward {
    fn backward(&self, grad: &Tensor) -> Vec<Tensor> {
        // d/dx(sum(x)) = 1 for all elements
        let size = self.input.size();
        let grad_val = grad.item();
        vec![Tensor::from_data(vec![grad_val; size], self.input.shape().to_vec())]
    }
    
    fn inputs(&self) -> Vec<Tensor> {
        vec![self.input.clone()]
    }
}

struct PowBackward {
    input: Tensor,
    exp: f32,
}

impl GradFn for PowBackward {
    fn backward(&self, grad: &Tensor) -> Vec<Tensor> {
        // d/dx(x^n) = n * x^(n-1)
        let local_grad: Vec<f32> = self.input.data()
            .iter()
            .map(|x| self.exp * x.powf(self.exp - 1.0))
            .collect();
        let local_tensor = Tensor::from_data(local_grad, self.input.shape().to_vec());
        vec![grad.mul(&local_tensor)]
    }
    
    fn inputs(&self) -> Vec<Tensor> {
        vec![self.input.clone()]
    }
}
