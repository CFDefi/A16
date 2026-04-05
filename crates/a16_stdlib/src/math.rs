//! Math Standard Library
//!
//! Mathematical constants and functions.

use std::f64::consts;

/// Mathematical constants
pub const PI: f64 = consts::PI;
pub const E: f64 = consts::E;
pub const TAU: f64 = consts::TAU;
pub const INF: f64 = f64::INFINITY;
pub const NEG_INF: f64 = f64::NEG_INFINITY;
pub const NAN: f64 = f64::NAN;

/// Absolute value
pub fn abs_int(x: i64) -> i64 { x.abs() }
pub fn abs_float(x: f64) -> f64 { x.abs() }

/// Power
pub fn pow(base: f64, exp: f64) -> f64 { base.powf(exp) }
pub fn ipow(base: i64, exp: u32) -> i64 { base.pow(exp) }

/// Square root
pub fn sqrt(x: f64) -> f64 { x.sqrt() }

/// Cube root
pub fn cbrt(x: f64) -> f64 { x.cbrt() }

/// Logarithms
pub fn log(x: f64) -> f64 { x.ln() }
pub fn log2(x: f64) -> f64 { x.log2() }
pub fn log10(x: f64) -> f64 { x.log10() }

/// Trigonometric functions
pub fn sin(x: f64) -> f64 { x.sin() }
pub fn cos(x: f64) -> f64 { x.cos() }
pub fn tan(x: f64) -> f64 { x.tan() }
pub fn asin(x: f64) -> f64 { x.asin() }
pub fn acos(x: f64) -> f64 { x.acos() }
pub fn atan(x: f64) -> f64 { x.atan() }
pub fn atan2(y: f64, x: f64) -> f64 { y.atan2(x) }

/// Hyperbolic functions
pub fn sinh(x: f64) -> f64 { x.sinh() }
pub fn cosh(x: f64) -> f64 { x.cosh() }
pub fn tanh(x: f64) -> f64 { x.tanh() }

/// Rounding
pub fn floor(x: f64) -> f64 { x.floor() }
pub fn ceil(x: f64) -> f64 { x.ceil() }
pub fn round(x: f64) -> f64 { x.round() }
pub fn trunc(x: f64) -> f64 { x.trunc() }

/// Min / Max
pub fn min_f64(a: f64, b: f64) -> f64 { a.min(b) }
pub fn max_f64(a: f64, b: f64) -> f64 { a.max(b) }
pub fn min_i64(a: i64, b: i64) -> i64 { a.min(b) }
pub fn max_i64(a: i64, b: i64) -> i64 { a.max(b) }

/// Clamp
pub fn clamp_f64(x: f64, lo: f64, hi: f64) -> f64 { x.clamp(lo, hi) }
pub fn clamp_i64(x: i64, lo: i64, hi: i64) -> i64 { x.clamp(lo, hi) }

/// Sign
pub fn sign(x: f64) -> f64 {
    if x > 0.0 { 1.0 }
    else if x < 0.0 { -1.0 }
    else { 0.0 }
}

/// Is NaN / Is Infinite
pub fn is_nan(x: f64) -> bool { x.is_nan() }
pub fn is_infinite(x: f64) -> bool { x.is_infinite() }
pub fn is_finite(x: f64) -> bool { x.is_finite() }

/// Degrees / Radians
pub fn to_degrees(x: f64) -> f64 { x.to_degrees() }
pub fn to_radians(x: f64) -> f64 { x.to_radians() }

/// GCD (Greatest Common Divisor)
pub fn gcd(mut a: i64, mut b: i64) -> i64 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// LCM (Least Common Multiple)
pub fn lcm(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 { return 0; }
    (a / gcd(a, b)) * b
}

/// Factorial (iterative, for small n)
pub fn factorial(n: u64) -> u64 {
    (1..=n).product()
}
