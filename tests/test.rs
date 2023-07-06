use std::mem;

use stacked_errors::{Error, ErrorKind};

trait VerifyCapable: Send + Sync {}
impl VerifyCapable for Error {}

#[cfg(target_pointer_width = "64")]
#[test]
fn error_kind_size() {
    assert_eq!(mem::size_of::<ErrorKind>(), 40);
}

#[test]
fn tests() {
    // thanks to thin-vec
    assert_eq!(mem::size_of::<Error>(), mem::size_of::<usize>());
    assert_eq!(mem::size_of::<Option<Error>>(), mem::size_of::<usize>());
    assert_eq!(mem::size_of::<Result<(), Error>>(), mem::size_of::<usize>());
}
