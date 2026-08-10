use stacked_errors::{Error, Result, StackableErr, UnitError};

fn ex(s: &str, error: bool) -> Result<String> {
    if error {
        // this line is the critical case that must work
        let _ = ron::from_str::<bool>("true").stack()?;
        Err(Error::from_err(s.to_owned()))
    } else {
        Ok(s.to_owned())
    }
}

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
    if cfg!(windows) {
        assert_eq!(
            format!("{}", tmp.unwrap_err()),
            r#"
    1:1: Expected boolean
    1:1: Expected boolean
    test long ___________________________________________________________________
  at tests\debug.rs 27:10
  at tests\debug.rs 25:10
    hello at tests\debug.rs 7:13"#
                .to_owned()
        );
    } else {
        assert_eq!(
            format!("{}", tmp.unwrap_err()),
            convert(
                r#"
    1:1: Expected boolean
    1:1: Expected boolean
    test long ___________________________________________________________________
  at tests/debug.rs 27:10
  at tests/debug.rs 25:10
    hello at tests/debug.rs 7:13"#
            )
        );
    }
}

/// `UnitError`s only contribute their location to the formatting, which has
/// several edge cases
#[test]
fn unit_error_debug() {
    // a locationless `UnitError` is skipped entirely, and must not leave behind a
    // trailing newline from the entry after it
    let tmp = Error::from_err_locationless(UnitError {}).add_err("hello");
    println!("{tmp:?}");
    assert_eq!(
        format!("{tmp}"),
        convert(
            r#"
    hello at tests/debug.rs 65:58"#
        )
    );

    // the line splitting of a `UnitError` location must not be influenced by the
    // length of the message of the entry before it
    let long = "_".repeat(75);
    let tmp = Error::new().add_err_locationless(long.clone());
    println!("{tmp:?}");
    assert_eq!(
        format!("{tmp}"),
        convert(&format!(
            r#"
    {long}
  at tests/debug.rs 78:15"#
        ))
    );

    // an empty stack formats to nothing
    assert_eq!(format!("{}", Error::empty()), "");

    // and a stack of only skipped entries also formats to nothing
    let tmp = Error::from_err_locationless(UnitError {}).add_err_locationless(UnitError {});
    assert_eq!(format!("{tmp}"), "");
}

pub fn convert(s: &str) -> String {
    let s = s.to_owned();
    if cfg!(windows) {
        s.replace("/", "\\")
    } else {
        s
    }
}
