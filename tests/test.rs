use core::mem;

use stacked_errors::{
    bail, Error, Result, StackableErr, StackedError, StackedErrorDowncast, UnitError,
};

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
        .stack_err("test")
        .stack_err_locationless(ron::from_str::<bool>("invalid").unwrap_err())
        .stack_err_locationless(Box::new(ron::from_str::<bool>("invalid").unwrap_err()));
    println!("{tmp:?}");
    assert_eq!(
        format!("{:?}", tmp),
        r#"Err(Error { stack: [
1:1: Expected boolean
1:1: Expected boolean
Location { file: "tests/test.rs", line: 34, col: 10 },
test
Location { file: "tests/test.rs", line: 32, col: 10 },
Location { file: "tests/test.rs", line: 11, col: 13 },
hello
] })"#
            .to_owned()
    );
}

#[cfg(target_pointer_width = "64")]
#[test]
fn error_kind_size() {
    //use std::panic::Location;

    // FIXME
    //assert_eq!(mem::size_of::<ErrorItem>(), 40);
}

#[test]
fn error_size() {
    // thanks to thin-vec
    assert_eq!(mem::size_of::<Error>(), mem::size_of::<usize>());
    assert_eq!(mem::size_of::<Option<Error>>(), mem::size_of::<usize>());
    assert_eq!(mem::size_of::<Result<()>>(), mem::size_of::<usize>());
}

fn assert_stack(stack: Result<()>, unit: bool, has_location: bool) {
    let stack = stack.unwrap_err();
    let mut iter = stack.iter();
    let e = iter.next().unwrap();
    assert!(iter.next().is_none());
    if has_location {
        assert!(e.get_location().is_some());
    } else {
        assert!(e.get_location().is_none());
    }
    if unit {
        e.downcast_ref::<UnitError>().unwrap();
    } else {
        e.downcast_ref::<&str>().unwrap();
    }
}

#[test]
fn stacking() {
    use ron::error::SpannedError;
    let tmp: std::result::Result<bool, SpannedError> = ron::from_str("invalid");
    let tmp: Result<bool> = tmp.stack_err("test");
    let tmp: Error = tmp.unwrap_err();
    assert_eq!(tmp.iter().len(), 2);
    let mut iter = tmp.iter();
    iter.next().unwrap().downcast_ref::<SpannedError>().unwrap();
    iter.next().unwrap().downcast_ref::<&str>().unwrap();

    assert_stack(Err(Error::new()), true, true);
    assert_stack(Err(Error::from_err("s")), false, true);
    assert_stack(Err(Error::from_err_locationless("s")), false, false);
    assert_stack(Err(Error::empty().add_err("s")), false, true);
    assert_stack(Err(Error::empty().add_err_locationless("s")), false, false);
    assert_stack(Err(Error::empty().add()), true, true);

    assert_stack(Err(Error::empty()).stack_err("e"), false, true);
    assert_stack(
        Err(Error::empty()).stack_err_locationless("e"),
        false,
        false,
    );
    assert_stack(Err(Error::empty()).stack(), true, true);
    let tmp: core::result::Result<u8, Error> = Err(Error::empty());
    let tmp: core::result::Result<u8, Error> = tmp.stack_locationless();
    assert_eq!(tmp.unwrap_err().iter().len(), 0);

    assert_stack(None.stack_err("e"), false, true);
    assert_stack(None.stack_err_locationless("e"), false, false);
    assert_stack(None.stack(), true, true);
    let tmp: Option<u8> = None;
    let tmp: core::result::Result<u8, Error> = tmp.stack();
    assert_eq!(tmp.unwrap_err().iter().len(), 1);

    assert_stack(Error::empty().stack_err("e"), false, true);
    assert_stack(Error::empty().stack_err_locationless("e"), false, false);
    assert_stack(Error::empty().stack(), true, true);
    let tmp = Error::empty();
    let tmp: core::result::Result<(), Error> = tmp.stack_locationless();
    assert_eq!(tmp.unwrap_err().iter().len(), 0);
}

#[test]
fn debug_and_display() {
    let x = Error::from_err_locationless("hello");
    assert_eq!(format!("{x:?}"), "Error { stack: [\nhello\n] }");
    assert_eq!(format!("{x}"), "Error { stack: [\nhello\n] }");
    let x = StackedError(x);
    assert_eq!(
        format!("{x:?}"),
        "StackedError(Error { stack: [\nhello\n] })"
    );
    assert_eq!(format!("{x}"), "StackedError(Error { stack: [\nhello\n] })");
}

#[test]
fn test_bail() {
    let f = || -> Result<()> { bail!("test") };
    let tmp = f().unwrap_err();
    let x = tmp.iter().next().unwrap();
    assert_eq!(*x.downcast_ref::<&str>().unwrap(), "test");

    let f = || -> Result<()> {
        let x = 5u64;
        bail!("test {x}")
    };
    let tmp = f().unwrap_err();
    let x = tmp.iter().next().unwrap();
    assert_eq!(*x.downcast_ref::<String>().unwrap(), "test 5");

    let f = || -> Result<()> {
        let x = 5u64;
        bail!("test {}", x)
    };
    let tmp = f().unwrap_err();
    let x = tmp.iter().next().unwrap();
    assert_eq!(*x.downcast_ref::<String>().unwrap(), "test 5");
}
