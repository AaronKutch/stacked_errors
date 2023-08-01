use std::mem;

use stacked_errors::{Error, ErrorKind, Result, StackableErr};

fn ex(s: &str, error: bool) -> Result<String> {
    if error {
        //Err(Error::from(s.to_owned()))
        s.to_owned().stack_err(|| "hello")
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
    let tmp = ex("hello", true)
        .stack()
        .stack_locationless()
        .stack_err(|| "test")
        .stack_err_locationless(|| {
            ErrorKind::from_err(ron::from_str::<bool>("invalid").unwrap_err())
        })
        .stack_err_locationless(|| {
            ErrorKind::from_box(Box::new(ron::from_str::<bool>("invalid").unwrap_err()))
        });
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

#[test]
fn stacking() {
    use ron::error::SpannedError;
    let tmp: std::result::Result<bool, SpannedError> = ron::from_str("invalid");
    let tmp: Result<bool> = tmp.stack_err(|| "test");
    let mut tmp: Error = tmp.unwrap_err();
    assert_eq!(tmp.stack.len(), 2);
    let kind: ErrorKind = tmp.stack.pop().unwrap().0;
    assert!(matches!(kind, ErrorKind::StrError(_)));
    let kind: ErrorKind = tmp.stack.pop().unwrap().0;
    let _: SpannedError = kind.downcast().unwrap();
}
