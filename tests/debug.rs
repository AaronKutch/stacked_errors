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
    // the long part tests that it puts location on another line
    let tmp = ex("hello", true)
        .stack()
        .stack_locationless()
        .stack_err("test long ___________________________________________________________________")
        .stack_err_locationless(ron::from_str::<bool>("invalid").unwrap_err())
        .stack_err_locationless(Box::new(ron::from_str::<bool>("invalid").unwrap_err()));
    println!("{tmp:?}");
    assert_eq!(
        format!("{}", tmp.unwrap_err()),
        r#"1:1: Expected boolean,
1:1: Expected boolean,
test long ___________________________________________________________________
at tests/debug.rs 17:10,
at tests/debug.rs 15:10,
hello at tests/debug.rs 37:13"#
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
