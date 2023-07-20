use std::mem;

use stacked_errors::{Error, ErrorKind, MapAddError, Result};

fn ex(s: &str, error: bool) -> Result<String> {
    if error {
        Err(Error::from(s.to_owned())).map_add_err(|| "map_add_err")
    } else {
        Ok(s.to_owned())
    }
}

trait VerifyCapable: Send + Sync {}
impl VerifyCapable for Error {}

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
    let tmp = ex("hello", true);
    dbg!(&tmp);
    assert_eq!(
        format!("{:?}", tmp),
        r#"Err(Error { stack: [
Location { file: "tests/test.rs", line: 7, col: 40 },
map_add_err
Location { file: "tests/test.rs", line: 7, col: 13 },
hello
] })"#
            .to_owned()
    );
}

#[cfg(target_pointer_width = "64")]
#[test]
fn error_kind_size() {
    assert_eq!(mem::size_of::<ErrorKind>(), 40);
}

#[test]
fn error_size() {
    // thanks to thin-vec
    assert_eq!(mem::size_of::<Error>(), mem::size_of::<usize>());
    assert_eq!(mem::size_of::<Option<Error>>(), mem::size_of::<usize>());
    assert_eq!(mem::size_of::<Result<()>>(), mem::size_of::<usize>());
}
