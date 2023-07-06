use crate::{Error, ErrorKind};

/// The intention of this trait is to convert `Option<T>`s, `Result<T, Error>`s,
/// and `Result<T, impl Into<ErrorKind>>`s into `Result<T, Error>`s with the
/// error having an `ErrorKind` and a location pushed onto its stacks
/// (`map_add_err` should have `#[track_caller]` on it and push on the
/// `Location::caller()`). You can also call `map_add_err` on plain `Error`s and
/// `impl Into<ErrorKind>`s to get a `Result<(), Error>`.
pub trait MapAddError {
    type Output;

    fn map_add_err<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output;
}

impl<T> MapAddError for core::result::Result<T, Error> {
    type Output = core::result::Result<T, Error>;

    #[track_caller]
    fn map_add_err<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output {
        match self {
            Ok(o) => Ok(o),
            Err(e) => Err(e.add_err(f())),
        }
    }
}

impl<T> MapAddError for Option<T> {
    type Output = core::result::Result<T, Error>;

    #[track_caller]
    fn map_add_err<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output {
        match self {
            Some(o) => Ok(o),
            None => Err(Error::from_kind(f())),
        }
    }
}

impl<T, K0: Into<ErrorKind>> MapAddError for core::result::Result<T, K0> {
    type Output = core::result::Result<T, Error>;

    /// Transforms `Result<T, K0>` into `Result<T, Error>` while adding location
    /// information and a second kind of error.
    #[track_caller]
    fn map_add_err<K1: Into<ErrorKind>, F: FnOnce() -> K1>(self, f: F) -> Self::Output {
        match self {
            Ok(o) => Ok(o),
            Err(kind) => Err(Error::from_kind(kind).add_err_no_location(f())),
        }
    }
}

impl MapAddError for Error {
    type Output = core::result::Result<(), Error>;

    #[track_caller]
    fn map_add_err<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output {
        Err(self.add_err(f()))
    }
}

impl<K0: Into<ErrorKind>> MapAddError for K0 {
    type Output = core::result::Result<(), Error>;

    #[track_caller]
    fn map_add_err<K1: Into<ErrorKind>, F: FnOnce() -> K1>(self, f: F) -> Self::Output {
        Err(Error::from_kind(self).add_err(f()))
    }
}
