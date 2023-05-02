/// Very naive implementation of FizzBuzz
pub fn fizz_buzz(i: u32) -> String {
    if i % 15 == 0 {
        "FizzBuzz".to_owned()
    } else if i % 3 == 0 {
        "Fizz".to_owned()
    } else if i % 5 == 0 {
        "Buzz".to_owned()
    } else {
        i.to_string()
    }
}

// TODO Write a unit test, using the contents of `fizzbuzz.out` file
// to compare.
// You can use the `include_str!()` macro to include file
// contents as `&str` in your artifact.
#[test]
fn test_fizz_buzz() {
    let out = include_str!("../fizzbuzz.out");
    for (idx, line) in out.lines().enumerate() {
        assert_eq!(fizz_buzz((idx + 1) as u32), line);
    }
}
