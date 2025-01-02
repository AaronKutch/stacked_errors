use stacked_errors::{Error, Result, StackableErr};

#[test]
fn error_debug() {
    assert_eq!(
        format!("{:?}", ex("hello", false)),
        r#"Ok("hello")"#.to_owned()
    );
    assert_eq!(
        format!("{:?}", ex("hello\"", false)),
        r#"Ok("hello\"")"#.to_owned()
    );
    let tmp = ex("hello", true)
        .stack()
        .stack_locationless()
        .stack_err("test")
        .stack_err_locationless(ron::from_str::<bool>("invalid").unwrap_err())
        .stack_err_locationless(Box::new(ron::from_str::<bool>("invalid").unwrap_err()));
    println!("{tmp:?}");
    assert_eq!(
        format!("{:?}", tmp),
        r#"Err(Error { stack: [
1:1: Expected boolean
1:1: Expected boolean
Location { file: "tests/debug.rs", line: 16, col: 10 },
test
Location { file: "tests/debug.rs", line: 14, col: 10 },
Location { file: "tests/debug.rs", line: 39, col: 13 },
hello
] })"#
            .to_owned()
    );
}

fn ex(s: &str, error: bool) -> Result<String> {
    if error {
        // this line is the critical case that must work
        let _ = ron::from_str::<bool>("true").stack()?;
        Err(Error::from_err(s.to_owned()))
    } else {
        Ok(s.to_owned())
    }
}
