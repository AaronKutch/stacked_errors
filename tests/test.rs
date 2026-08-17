use core::mem;

use stacked_errors::{
    bail, bail_locationless, eyre, Error, Result, StackableErr, StackedError, StackedErrorDowncast,
    UnitError,
};

#[allow(unused)]
trait VerifyCapable: Send + Sync {}
impl VerifyCapable for Error {}

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
    assert!(format!("{x:?}").contains("hello"));
    assert_eq!(format!("{x}"), "\n    hello");
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

    // the no argument case gets a standard message and still has a location
    let f = || -> Result<()> { bail!() };
    let tmp = f().unwrap_err();
    let x = tmp.iter().next().unwrap();
    assert_eq!(*x.downcast_ref::<&str>().unwrap(), "explicit bail");
    assert!(x.get_location().is_some());

    let f = || -> Result<()> { bail_locationless!() };
    let tmp = f().unwrap_err();
    let x = tmp.iter().next().unwrap();
    assert_eq!(*x.downcast_ref::<&str>().unwrap(), "explicit bail");
    assert!(x.get_location().is_none());
}

/// An [Error] that is passed to one of the `*_err*` functions has its stack
/// chained on, rather than being nested inside of a single entry
#[test]
fn nested_errors() {
    fn inner() -> Error {
        Error::from_err("root").add_err("mid")
    }

    // the two entries of `inner`, plus the location of the `from_err` call
    let e = Error::from_err(inner());
    assert_eq!(e.iter().len(), 3);
    let mut iter = e.iter();
    assert_eq!(
        *iter.next().unwrap().downcast_ref::<&str>().unwrap(),
        "root"
    );
    assert_eq!(*iter.next().unwrap().downcast_ref::<&str>().unwrap(), "mid");
    iter.next().unwrap().downcast_ref::<UnitError>().unwrap();

    // the locationless variants do not push the extra location
    let e = Error::from_err_locationless(inner());
    assert_eq!(e.iter().len(), 2);
    let e = Error::from_err("outer").add_err_locationless(inner());
    assert_eq!(e.iter().len(), 3);

    // chaining onto a foreign error type
    let e = ron::from_str::<bool>("invalid")
        .stack_err(inner())
        .unwrap_err();
    assert_eq!(e.iter().len(), 4);
    e.iter()
        .next()
        .unwrap()
        .downcast_ref::<ron::error::SpannedError>()
        .unwrap();

    // chaining when both sides are `Error`s
    let e = Err::<(), Error>(Error::from_err("outer"))
        .stack_err(inner())
        .unwrap_err();
    assert_eq!(e.iter().len(), 4);

    // the special errors are found through a chain, which is the main thing that
    // nesting used to silently break
    assert!(Error::from_err(Error::timeout()).is_timeout());
    assert!(Error::from_err(Error::probably_not_root_cause()).is_probably_not_root_cause());
    assert!(Error::empty().add_err(Error::timeout()).is_timeout());
    assert!(eyre!(Error::timeout()).is_timeout());
    fn bailing() -> Result<()> {
        bail!(Error::timeout())
    }
    assert!(bailing().unwrap_err().is_timeout());
}

#[test]
fn test_special() {
    let e = Error::from_err("hello")
        .chain_errors(StackedError::probably_not_root_cause())
        .add_err("world");
    assert_eq!(e.iter().len(), 3);
    assert!(e.is_probably_not_root_cause());
    let e = Error::from_err("hello")
        .chain_errors(StackedError::timeout())
        .add_err("world");
    assert_eq!(e.iter().len(), 3);
    assert!(e.is_timeout());
}
