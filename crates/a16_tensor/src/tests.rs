//! Tests for a16_tensor

use crate::tensor::Tensor;

#[test]
fn test_zeros() {
    let t = Tensor::zeros(&[2, 3]);
    assert_eq!(t.shape(), &[2, 3]);
    assert_eq!(t.size(), 6);
    for &v in t.data().iter() {
        assert_eq!(v, 0.0);
    }
}

#[test]
fn test_ones() {
    let t = Tensor::ones(&[3, 2]);
    assert_eq!(t.shape(), &[3, 2]);
    for &v in t.data().iter() {
        assert_eq!(v, 1.0);
    }
}

#[test]
fn test_randn() {
    let t = Tensor::randn(&[10, 10]);
    assert_eq!(t.shape(), &[10, 10]);
    assert_eq!(t.size(), 100);
    // Check that values are not all zero (very unlikely for randn)
    let sum: f32 = t.data().iter().map(|x| x.abs()).sum();
    assert!(sum > 0.1);
}

#[test]
fn test_add() {
    let a = Tensor::from_data(vec![1.0, 2.0, 3.0], vec![3]);
    let b = Tensor::from_data(vec![4.0, 5.0, 6.0], vec![3]);
    let c = a.add(&b);
    let data = c.data();
    assert_eq!(data[0], 5.0);
    assert_eq!(data[1], 7.0);
    assert_eq!(data[2], 9.0);
}

#[test]
fn test_mul() {
    let a = Tensor::from_data(vec![2.0, 3.0], vec![2]);
    let b = Tensor::from_data(vec![4.0, 5.0], vec![2]);
    let c = a.mul(&b);
    let data = c.data();
    assert_eq!(data[0], 8.0);
    assert_eq!(data[1], 15.0);
}

#[test]
fn test_matmul() {
    // [1, 2] @ [[1], [2]] = [5]
    let a = Tensor::from_data(vec![1.0, 2.0], vec![1, 2]);
    let b = Tensor::from_data(vec![1.0, 2.0], vec![2, 1]);
    let c = a.matmul(&b);
    assert_eq!(c.shape(), &[1, 1]);
    assert_eq!(c.data()[0], 5.0);
}

#[test]
fn test_matmul_2d() {
    // [[1, 2], [3, 4]] @ [[1, 0], [0, 1]] = [[1, 2], [3, 4]]
    let a = Tensor::from_data(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]);
    let b = Tensor::from_data(vec![1.0, 0.0, 0.0, 1.0], vec![2, 2]);
    let c = a.matmul(&b);
    assert_eq!(c.shape(), &[2, 2]);
    let data = c.data();
    assert_eq!(data[0], 1.0);
    assert_eq!(data[1], 2.0);
    assert_eq!(data[2], 3.0);
    assert_eq!(data[3], 4.0);
}

#[test]
fn test_relu() {
    let a = Tensor::from_data(vec![-1.0, 0.0, 1.0, 2.0], vec![4]);
    let b = a.relu();
    let data = b.data();
    assert_eq!(data[0], 0.0);
    assert_eq!(data[1], 0.0);
    assert_eq!(data[2], 1.0);
    assert_eq!(data[3], 2.0);
}

#[test]
fn test_sigmoid() {
    let a = Tensor::from_data(vec![0.0], vec![1]);
    let b = a.sigmoid();
    assert!((b.data()[0] - 0.5).abs() < 1e-6);
}

#[test]
fn test_mean() {
    let a = Tensor::from_data(vec![1.0, 2.0, 3.0, 4.0], vec![4]);
    let m = a.mean();
    assert_eq!(m.item(), 2.5);
}

#[test]
fn test_sum() {
    let a = Tensor::from_data(vec![1.0, 2.0, 3.0], vec![3]);
    let s = a.sum();
    assert_eq!(s.item(), 6.0);
}

#[test]
fn test_backward_simple() {
    // y = x^2 + 2x, dy/dx = 2x + 2
    // At x = 3: dy/dx = 8
    let mut x = Tensor::from_data(vec![3.0], vec![1]);
    x.set_requires_grad(true);
    
    let two = Tensor::from_data(vec![2.0], vec![1]);
    let x_sq = x.mul(&x);
    let two_x = two.mul(&x);
    let y = x_sq.add(&two_x);
    
    y.backward();
    
    let grad = x.grad().unwrap();
    // dy/dx = 2*3 + 2 = 8
    assert!((grad.data()[0] - 8.0).abs() < 1e-5);
}

#[test]
fn test_manual_gradient_descent() {
    // Manual gradient descent without optimizer
    // y = 2*x, learn w such that w*x ≈ y
    
    let x_data = vec![1.0, 2.0, 3.0, 4.0];
    let y_data = vec![2.0, 4.0, 6.0, 8.0];
    
    let x = Tensor::from_data(x_data, vec![4]);
    let y = Tensor::from_data(y_data, vec![4]);
    
    let mut w = Tensor::from_data(vec![0.5], vec![1]);
    w.set_requires_grad(true);
    let lr = 0.05;  // Higher learning rate
    
    for _ in 0..200 {  // More iterations
        let pred = x.mul(&w);
        let diff = pred.sub(&y);
        let loss = diff.mul(&diff).mean();
        
        loss.backward();
        
        // Manual update
        if let Some(grad) = w.grad() {
            let grad_data = grad.data();
            let mut w_data = w.data_mut();
            for i in 0..w_data.len() {
                w_data[i] -= lr * grad_data[i];
            }
        }
        w.zero_grad();
    }
    
    let final_w = w.data()[0];
    assert!((final_w - 2.0).abs() < 0.2, "Expected w ≈ 2.0, got {}", final_w);
}

#[test]
fn test_transpose() {
    let a = Tensor::from_data(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]);
    let b = a.transpose();
    assert_eq!(b.shape(), &[3, 2]);
    let data = b.data();
    // Original: [[1,2,3],[4,5,6]]
    // Transposed: [[1,4],[2,5],[3,6]]
    assert_eq!(data[0], 1.0);
    assert_eq!(data[1], 4.0);
    assert_eq!(data[2], 2.0);
    assert_eq!(data[3], 5.0);
}

#[test]
fn test_backward_matmul() {
    // Simple matmul backward test
    let mut a = Tensor::from_data(vec![1.0, 2.0], vec![1, 2]);
    a.set_requires_grad(true);
    let b = Tensor::from_data(vec![3.0, 4.0], vec![2, 1]);
    // a @ b = [[1*3 + 2*4]] = [[11]]
    let c = a.matmul(&b);
    c.backward();
    
    // d(a@b)/da = b.T = [[3, 4]]
    let grad = a.grad().unwrap();
    assert_eq!(grad.shape(), &[1, 2]);
    assert!((grad.data()[0] - 3.0).abs() < 1e-5);
    assert!((grad.data()[1] - 4.0).abs() < 1e-5);
}
