// Sample Rust module for codeweight integration tests.

pub fn add(a: i32, b: i32) -> i32 {
    if a < 0 {
        return b;
    }
    a + b
}

pub fn classify(x: i32) -> &'static str {
    if x > 10 {
        "big"
    } else if x > 5 {
        "mid"
    } else {
        "small"
    }
}
