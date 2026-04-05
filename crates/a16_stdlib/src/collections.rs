//! Collections Standard Library
//!
//! Advanced collection operations for lists, dicts, and sets.

/// Sum of a list of integers
pub fn sum_int(items: &[i64]) -> i64 { items.iter().sum() }

/// Sum of a list of floats
pub fn sum_float(items: &[f64]) -> f64 { items.iter().sum() }

/// Product of a list of integers
pub fn product_int(items: &[i64]) -> i64 { items.iter().product() }

/// Product of a list of floats
pub fn product_float(items: &[f64]) -> f64 { items.iter().product() }

/// Average of a list of floats
pub fn average(items: &[f64]) -> f64 {
    if items.is_empty() { return 0.0; }
    items.iter().sum::<f64>() / items.len() as f64
}

/// Median of a list of floats
pub fn median(items: &mut [f64]) -> f64 {
    if items.is_empty() { return 0.0; }
    items.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = items.len() / 2;
    if items.len() % 2 == 0 {
        (items[mid - 1] + items[mid]) / 2.0
    } else {
        items[mid]
    }
}

/// Min of a list of floats
pub fn min_of(items: &[f64]) -> Option<f64> {
    items.iter().copied().reduce(f64::min)
}

/// Max of a list of floats
pub fn max_of(items: &[f64]) -> Option<f64> {
    items.iter().copied().reduce(f64::max)
}

/// Flatten a 2D list
pub fn flatten<T: Clone>(nested: &[Vec<T>]) -> Vec<T> {
    nested.iter().flat_map(|v| v.iter().cloned()).collect()
}

/// Get unique elements (preserving order)
pub fn unique<T: Clone + PartialEq>(items: &[T]) -> Vec<T> {
    let mut result = Vec::new();
    for item in items {
        if !result.contains(item) {
            result.push(item.clone());
        }
    }
    result
}

/// Chunk a list into groups of size n
pub fn chunk<T: Clone>(items: &[T], size: usize) -> Vec<Vec<T>> {
    items.chunks(size).map(|c| c.to_vec()).collect()
}

/// Zip two lists together
pub fn zip_lists<A: Clone, B: Clone>(a: &[A], b: &[B]) -> Vec<(A, B)> {
    a.iter().cloned().zip(b.iter().cloned()).collect()
}

/// Enumerate a list (index, value)
pub fn enumerate_list<T: Clone>(items: &[T]) -> Vec<(usize, T)> {
    items.iter().cloned().enumerate().collect()
}

/// Take first n elements
pub fn take<T: Clone>(items: &[T], n: usize) -> Vec<T> {
    items.iter().take(n).cloned().collect()
}

/// Skip first n elements
pub fn skip<T: Clone>(items: &[T], n: usize) -> Vec<T> {
    items.iter().skip(n).cloned().collect()
}

/// Check if any element satisfies predicate
pub fn any_true(items: &[bool]) -> bool { items.iter().any(|&b| b) }

/// Check if all elements satisfy predicate
pub fn all_true(items: &[bool]) -> bool { items.iter().all(|&b| b) }

/// Count elements that are true
pub fn count_true(items: &[bool]) -> usize { items.iter().filter(|&&b| b).count() }

/// Range (inclusive start, exclusive end)
pub fn range_int(start: i64, end: i64) -> Vec<i64> {
    (start..end).collect()
}

/// Range with step
pub fn range_step(start: i64, end: i64, step: i64) -> Vec<i64> {
    if step == 0 { return vec![]; }
    let mut result = Vec::new();
    let mut i = start;
    if step > 0 {
        while i < end {
            result.push(i);
            i += step;
        }
    } else {
        while i > end {
            result.push(i);
            i += step;
        }
    }
    result
}
