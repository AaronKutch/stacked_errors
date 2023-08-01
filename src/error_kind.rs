use alloc::boxed::Box;

/// In the future we plan on having almost every kind of error here under
/// different feature gates. Please file an issue if you would like to include
/// something.
///
/// The intention with `TimeoutError` is that if it is in the error stack, a
/// timeout occured. When other timeout structs are used, this should be added
/// on.
///
/// `ProbablyNotRootCauseError` is used to signal that the error is probably not
/// the "root cause". This is used by the `super_orchestrator` crate to reduce
/// noise when one error triggers a cascade of errors in a network.
///
/// Some things besides `BoxedError` are boxed to reduce the overall size of
/// `ErrorKind`
#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    // used for special cases where we need something
    #[error("UnitError")]
    UnitError,
    #[error("TimeoutError")]
    TimeoutError,
    #[error("ProbablyNotRootCauseError")]
    ProbablyNotRootCauseError,
    #[error("StrError")]
    StrError(&'static str),
    #[error("StringError")]
    StringError(alloc::string::String),
    #[error("BoxedError")]
    BoxedError(Box<dyn std::error::Error + Send + Sync>),
    #[error("TryFromIntError")]
    TryFromIntError(core::num::TryFromIntError),
    #[error("StdIoError")]
    StdIoError(std::io::Error),
    #[error("FromUtf8Error")]
    FromUtf8Error(alloc::string::FromUtf8Error),
    // this is more obscure but I think we should keep it because it deals with bad strings, and
    // we don't want that in our generic string errors.
    #[error("FromUtf16Error")]
    FromUtf16Error(alloc::string::FromUtf16Error),
    #[error("ParseIntError")]
    ParseIntError(core::num::ParseIntError),
    #[error("ParseFloatError")]
    ParseFloatError(core::num::ParseFloatError),
}

use ErrorKind::*;

impl ErrorKind {
    pub fn from_err<E: std::error::Error + Send + Sync + 'static>(e: E) -> Self {
        ErrorKind::BoxedError(Box::new(e))
    }

    pub fn from_box(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        ErrorKind::BoxedError(e)
    }

    /// Attempts to downcast from a `ErrorKind::BoxedError` to a concrete type.
    /// Returns the input in an `Err` if it was not an `ErrorKind::BoxedError`
    /// or the box would not downcast.
    pub fn downcast<E: std::error::Error + 'static>(self) -> Result<E, Self> {
        if let BoxedError(boxed_err) = self {
            match boxed_err.downcast() {
                Ok(err) => Ok(*err),
                Err(boxed_err) => Err(Self::BoxedError(boxed_err)),
            }
        } else {
            Err(self)
        }
    }
}

impl Default for ErrorKind {
    fn default() -> Self {
        UnitError
    }
}
