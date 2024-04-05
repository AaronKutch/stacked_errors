use std::mem;

use stacked_errors::{Error, ErrorKind, Result, StackableErr, StackedError};

fn ex(s: &str, error: bool) -> Result<String> {
    if error {
        // this line is the critical case that must work
        let _ = ron::from_str::<bool>("true").stack()?;
        Err(Error::from(s.to_owned()))
    } else {
        Ok(s.to_owned())
    }
}

#[allow(unused)]
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
    println!("{tmp:?}");
    assert_eq!(
        format!("{:?}", tmp),
        r#"Err(Error { stack: [
BoxedError(SpannedError { code: ExpectedBoolean, position: Position { line: 1, col: 1 } }),
BoxedError(SpannedError { code: ExpectedBoolean, position: Position { line: 1, col: 1 } }),
Location { file: "tests/test.rs", line: 31, col: 10 },
test
Location { file: "tests/test.rs", line: 29, col: 10 },
Location { file: "tests/test.rs", line: 9, col: 13 },
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

fn assert_stack(stack: Result<()>, unit: bool, boxed: bool, has_location: bool) {
    dbg!(&stack);
    let mut stack = stack.unwrap_err();
    assert_eq!(stack.stack.len(), 1);
    let e = stack.stack.pop().unwrap();
    if has_location {
        assert!(e.1.is_some());
    } else {
        assert!(e.1.is_none());
    }
    if boxed {
        assert!(matches!(e.0, ErrorKind::BoxedError(_)));
    } else if unit {
        assert!(matches!(e.0, ErrorKind::UnitError));
    } else {
        assert!(matches!(e.0, ErrorKind::StrError(_)));
    }
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

    // I'm just using `ErrorKind` here for something that implements `Error`
    let x = || Box::new(ErrorKind::StrError("box")) as Box<dyn std::error::Error + Sync + Send>;
    let y = || ErrorKind::StrError("err");

    assert_stack(Err(Error::new()), true, false, true);
    assert_stack(Err(Error::from_box(x())), false, true, true);
    assert_stack(Err(Error::from_box_locationless(x())), false, true, false);
    assert_stack(Err(Error::from_err(y())), false, true, true);
    assert_stack(Err(Error::from_err_locationless(y())), false, true, false);
    assert_stack(Err(Error::from_kind("s")), false, false, true);
    assert_stack(Err(Error::from_kind_locationless("s")), false, false, false);
    assert_stack(Err(Error::empty().add_kind("s")), false, false, true);
    assert_stack(
        Err(Error::empty().add_kind_locationless("s")),
        false,
        false,
        false,
    );
    assert_stack(Err(Error::empty().add_location()), true, false, true);

    assert_stack(Err(Error::empty()).stack_err(|| "e"), false, false, true);
    assert_stack(
        Err(Error::empty()).stack_err_locationless(|| "e"),
        false,
        false,
        false,
    );
    assert_stack(Err(Error::empty()).stack(), true, false, true);
    let tmp: core::result::Result<u8, Error> = Err(Error::empty());
    let tmp: core::result::Result<u8, Error> = tmp.stack_locationless();
    assert!(tmp.unwrap_err().stack.is_empty());

    assert_stack(None.stack_err(|| "e"), false, false, true);
    assert_stack(None.stack_err_locationless(|| "e"), false, false, false);
    assert_stack(None.stack(), true, false, true);
    let tmp: Option<u8> = None;
    let tmp: core::result::Result<u8, Error> = tmp.stack_locationless();
    assert!(tmp.unwrap_err().stack.is_empty());

    assert_stack(Error::empty().stack_err(|| "e"), false, false, true);
    assert_stack(
        Error::empty().stack_err_locationless(|| "e"),
        false,
        false,
        false,
    );
    assert_stack(Error::empty().stack(), true, false, true);
    let tmp = Error::empty();
    let tmp: core::result::Result<(), Error> = tmp.stack_locationless();
    assert!(tmp.unwrap_err().stack.is_empty());
}

#[test]
fn debug_and_display() {
    //let x = format!("{}", ron::from_str::<bool>("lkj").unwrap_err());
    let x = ErrorKind::StrError("hello");
    assert_eq!(format!("{x:?}"), "StrError(\"hello\")");
    assert_eq!(format!("{x}"), "hello");
    let x = ErrorKind::StringError("hello".to_owned());
    assert_eq!(format!("{x}"), "hello");
    let x = Error::from_kind_locationless("hello");
    assert_eq!(format!("{x:?}"), "Error { stack: [\nhello\n] }");
    assert_eq!(format!("{x}"), "Error { stack: [\nhello\n] }");
    let x = StackedError(x);
    assert_eq!(
        format!("{x:?}"),
        "StackedError(Error { stack: [\nhello\n] })"
    );
    assert_eq!(format!("{x}"), "StackedError(Error { stack: [\nhello\n] })");
}
