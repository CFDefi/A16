//! Extended Standard Library Tests

// =========================================================================
// Math Tests
// =========================================================================

#[test]
fn test_math_constants() {
    assert!((crate::math::PI - 3.14159265).abs() < 0.0001);
    assert!((crate::math::E - 2.71828).abs() < 0.001);
    assert!((crate::math::TAU - 6.28318).abs() < 0.001);
}

#[test]
fn test_math_abs() {
    assert_eq!(crate::math::abs_int(-42), 42);
    assert_eq!(crate::math::abs_int(42), 42);
    assert_eq!(crate::math::abs_float(-3.14), 3.14);
}

#[test]
fn test_math_sqrt() {
    assert!((crate::math::sqrt(4.0) - 2.0).abs() < 1e-10);
    assert!((crate::math::sqrt(9.0) - 3.0).abs() < 1e-10);
}

#[test]
fn test_math_trig() {
    assert!((crate::math::sin(0.0)).abs() < 1e-10);
    assert!((crate::math::cos(0.0) - 1.0).abs() < 1e-10);
}

#[test]
fn test_math_rounding() {
    assert_eq!(crate::math::floor(3.7), 3.0);
    assert_eq!(crate::math::ceil(3.2), 4.0);
    assert_eq!(crate::math::round(3.5), 4.0);
    assert_eq!(crate::math::trunc(3.9), 3.0);
}

#[test]
fn test_math_clamp() {
    assert_eq!(crate::math::clamp_i64(5, 1, 10), 5);
    assert_eq!(crate::math::clamp_i64(-5, 1, 10), 1);
    assert_eq!(crate::math::clamp_i64(15, 1, 10), 10);
}

#[test]
fn test_math_gcd_lcm() {
    assert_eq!(crate::math::gcd(12, 8), 4);
    assert_eq!(crate::math::gcd(7, 13), 1);
    assert_eq!(crate::math::lcm(4, 6), 12);
}

#[test]
fn test_math_factorial() {
    assert_eq!(crate::math::factorial(0), 1);
    assert_eq!(crate::math::factorial(5), 120);
    assert_eq!(crate::math::factorial(10), 3628800);
}

#[test]
fn test_math_sign() {
    assert_eq!(crate::math::sign(5.0), 1.0);
    assert_eq!(crate::math::sign(-3.0), -1.0);
    assert_eq!(crate::math::sign(0.0), 0.0);
}

#[test]
fn test_math_log() {
    assert!((crate::math::log(crate::math::E) - 1.0).abs() < 1e-10);
    assert!((crate::math::log2(8.0) - 3.0).abs() < 1e-10);
    assert!((crate::math::log10(1000.0) - 3.0).abs() < 1e-10);
}

// =========================================================================
// String Tests
// =========================================================================

#[test]
fn test_string_case() {
    assert_eq!(crate::string::to_upper("hello"), "HELLO");
    assert_eq!(crate::string::to_lower("HELLO"), "hello");
    assert_eq!(crate::string::capitalize("hello"), "Hello");
    assert_eq!(crate::string::title_case("hello world"), "Hello World");
}

#[test]
fn test_string_trim() {
    assert_eq!(crate::string::trim("  hello  "), "hello");
    assert_eq!(crate::string::trim_start("  hello"), "hello");
    assert_eq!(crate::string::trim_end("hello  "), "hello");
}

#[test]
fn test_string_pad() {
    assert_eq!(crate::string::pad_start("42", 5, '0'), "00042");
    assert_eq!(crate::string::pad_end("hi", 5, '.'), "hi...");
}

#[test]
fn test_string_split_join() {
    let parts = crate::string::split("a,b,c", ",");
    assert_eq!(parts, vec!["a", "b", "c"]);
    assert_eq!(crate::string::join(&parts, "-"), "a-b-c");
}

#[test]
fn test_string_search() {
    assert!(crate::string::contains("hello world", "world"));
    assert_eq!(crate::string::index_of("hello", "ll"), 2);
    assert_eq!(crate::string::index_of("hello", "xyz"), -1);
    assert_eq!(crate::string::count("abcabc", "abc"), 2);
}

#[test]
fn test_string_replace() {
    assert_eq!(crate::string::replace("aaa", "a", "b"), "bbb");
    assert_eq!(crate::string::replace_first("aaa", "a", "b"), "baa");
}

#[test]
fn test_string_checks() {
    assert!(crate::string::is_digit("12345"));
    assert!(!crate::string::is_digit("123a5"));
    assert!(crate::string::is_alpha("hello"));
    assert!(crate::string::is_alnum("hello123"));
    assert!(crate::string::is_blank("   "));
    assert!(!crate::string::is_blank("  a  "));
}

#[test]
fn test_string_reverse() {
    assert_eq!(crate::string::reverse("hello"), "olleh");
}

#[test]
fn test_string_substring() {
    assert_eq!(crate::string::substring("hello world", 6, 11), "world");
    assert_eq!(crate::string::char_at("hello", 1), Some('e'));
    assert_eq!(crate::string::char_count("hello"), 5);
}

// =========================================================================
// Collections Tests
// =========================================================================

#[test]
fn test_collections_sum() {
    assert_eq!(crate::collections::sum_int(&[1, 2, 3, 4, 5]), 15);
    assert!((crate::collections::sum_float(&[1.0, 2.5, 3.5]) - 7.0).abs() < 1e-10);
}

#[test]
fn test_collections_product() {
    assert_eq!(crate::collections::product_int(&[1, 2, 3, 4]), 24);
}

#[test]
fn test_collections_average_median() {
    assert!((crate::collections::average(&[1.0, 2.0, 3.0]) - 2.0).abs() < 1e-10);
    assert!((crate::collections::median(&mut [3.0, 1.0, 2.0]) - 2.0).abs() < 1e-10);
    assert!((crate::collections::median(&mut [1.0, 2.0, 3.0, 4.0]) - 2.5).abs() < 1e-10);
}

#[test]
fn test_collections_min_max() {
    assert_eq!(crate::collections::min_of(&[3.0, 1.0, 2.0]), Some(1.0));
    assert_eq!(crate::collections::max_of(&[3.0, 1.0, 2.0]), Some(3.0));
}

#[test]
fn test_collections_flatten() {
    let nested = vec![vec![1, 2], vec![3, 4]];
    assert_eq!(crate::collections::flatten(&nested), vec![1, 2, 3, 4]);
}

#[test]
fn test_collections_unique() {
    assert_eq!(crate::collections::unique(&[1, 2, 2, 3, 1]), vec![1, 2, 3]);
}

#[test]
fn test_collections_chunk() {
    let chunks = crate::collections::chunk(&[1, 2, 3, 4, 5], 2);
    assert_eq!(chunks, vec![vec![1, 2], vec![3, 4], vec![5]]);
}

#[test]
fn test_collections_take_skip() {
    assert_eq!(crate::collections::take(&[1, 2, 3, 4, 5], 3), vec![1, 2, 3]);
    assert_eq!(crate::collections::skip(&[1, 2, 3, 4, 5], 3), vec![4, 5]);
}

#[test]
fn test_collections_range() {
    assert_eq!(crate::collections::range_int(0, 5), vec![0, 1, 2, 3, 4]);
    assert_eq!(crate::collections::range_step(0, 10, 3), vec![0, 3, 6, 9]);
}

#[test]
fn test_collections_any_all() {
    assert!(crate::collections::any_true(&[false, true, false]));
    assert!(!crate::collections::all_true(&[true, false, true]));
    assert!(crate::collections::all_true(&[true, true, true]));
    assert_eq!(crate::collections::count_true(&[true, false, true]), 2);
}

// =========================================================================
// JSON Tests
// =========================================================================

#[test]
fn test_json_parse() {
    let val = crate::json::parse(r#"{"name": "A16", "version": 1}"#).unwrap();
    assert!(matches!(val, crate::json::A16Json::Object(_)));
}

#[test]
fn test_json_stringify() {
    let val = crate::json::A16Json::Object(vec![
        ("name".to_string(), crate::json::A16Json::String("A16".to_string())),
        ("version".to_string(), crate::json::A16Json::Int(1)),
    ]);
    let json = crate::json::stringify(&val);
    assert!(json.contains("A16"));
    assert!(json.contains("1"));
}

#[test]
fn test_json_field_access() {
    let val = crate::json::parse(r#"{"x": 42}"#).unwrap();
    let field = crate::json::get_field(&val, "x").unwrap();
    assert_eq!(crate::json::as_int(field), Some(42));
}

#[test]
fn test_json_array() {
    let val = crate::json::parse("[1, 2, 3]").unwrap();
    let elem = crate::json::get_index(&val, 1).unwrap();
    assert_eq!(crate::json::as_int(elem), Some(2));
}

#[test]
fn test_json_null() {
    let val = crate::json::parse("null").unwrap();
    assert!(crate::json::is_null(&val));
}

// =========================================================================
// I/O Tests
// =========================================================================

#[test]
fn test_io_path_utils() {
    assert_eq!(crate::io::file_extension("test.a16"), Some("a16".to_string()));
    assert_eq!(crate::io::file_stem("test.a16"), Some("test".to_string()));
}

#[test]
fn test_io_join_path() {
    let joined = crate::io::join_path("src", "main.rs");
    assert!(joined.contains("main.rs"));
}

// =========================================================================
// OS Tests
// =========================================================================

#[test]
fn test_os_name() {
    let name = crate::os::os_name();
    assert!(!name.is_empty());
}

#[test]
fn test_os_arch() {
    let arch = crate::os::arch();
    assert!(!arch.is_empty());
}

#[test]
fn test_os_timestamp() {
    let ts = crate::os::timestamp();
    assert!(ts > 1_700_000_000);  // After 2023
}

#[test]
fn test_os_pid() {
    let pid = crate::os::pid();
    assert!(pid > 0);
}

#[test]
fn test_os_env() {
    let val = crate::os::env_get_or("NONEXISTENT_VAR_XYZ", "default");
    assert_eq!(val, "default");
}
