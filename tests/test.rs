use std::mem;

use stacked_errors::{Error, ErrorKind};

trait VerifyCapable: Send + Sync {}
impl VerifyCapable for Error {}

#[test]
fn tests() {
    // thanks to thin-vec
    assert_eq!(mem::size_of::<Error>(), mem::size_of::<usize>());
    assert_eq!(mem::size_of::<Option<Error>>(), mem::size_of::<usize>());
    assert_eq!(mem::size_of::<Result<(), Error>>(), mem::size_of::<usize>());

    assert_eq!(mem::size_of::<ErrorKind>(), 96);
}
