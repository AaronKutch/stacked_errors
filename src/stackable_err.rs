use crate::{Error, ErrorKind};

/// The intention of this trait is to convert `Option<T>`s, `Result<T, Error>`s,
/// and `Result<T, impl Into<ErrorKind>>`s into `Result<T, Error>`s with the
/// error having an `ErrorKind` and a location pushed onto its stacks
/// (`map_add_err` should have `#[track_caller]` on it and push on the
/// `Location::caller()`). You can also call `map_add_err` on plain `Error`s and
/// `impl Into<ErrorKind>`s to get a `Result<(), Error>`.
pub trait StackableErr {
    type Output;

    /// Pushes the result of `f` and location information to the stack
    fn stack_err<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output;

    /// Pushes the result of `f` without location information
    fn stack_err_locationless<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output;

    /// Pushes just location information to the stack
    fn stack(self) -> Self::Output;

    /// Only converts to `Self::Output`
    fn stack_locationless(self) -> Self::Output;
}

// NOTE: trait conflicts prevent us from implementing some desirable cases.
// However, if specialization allows us to one day implement more, we have to be
// careful that internal behavior similar to
//
// `Err(Error::from(string)).stack_err(|| "...")`
//
// is enforced, i.e. we do not want strings boxed if we were able to write
//
// `Err(string).stack()`
// or `string.stack()`
//
// the current state of affairs is cumbersome when starting from a
// `Into<ErrorKind>` wrapped with nothing, but we do not want to invoke the
// `impl<T, E: std::error::Error + Send + Sync + 'static> StackableErr for
// core::result::Result<T, E>` impl on any `Into<ErrorKind>` types

impl<T> StackableErr for core::result::Result<T, Error> {
    type Output = core::result::Result<T, Error>;

    #[track_caller]
    fn stack_err<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output {
        match self {
            Ok(o) => Ok(o),
            Err(e) => Err(e.add_kind(f())),
        }
    }

    fn stack_err_locationless<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output {
        match self {
            Ok(o) => Ok(o),
            Err(e) => Err(e.add_kind_locationless(f())),
        }
    }

    #[track_caller]
    fn stack(self) -> Self::Output {
        match self {
            Ok(o) => Ok(o),
            Err(e) => Err(e.add_location()),
        }
    }

    fn stack_locationless(self) -> Self::Output {
        self
    }
}

impl<T, E: std::error::Error + Send + Sync + 'static> StackableErr for core::result::Result<T, E> {
    type Output = core::result::Result<T, Error>;

    #[track_caller]
    fn stack_err<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output {
        match self {
            Ok(o) => Ok(o),
            // location added by boxing
            Err(err) => Err(Error::from_box(Box::new(err)).add_kind_locationless(f())),
        }
    }

    fn stack_err_locationless<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output {
        match self {
            Ok(o) => Ok(o),
            Err(err) => Err(Error::from_box_locationless(Box::new(err)).add_kind_locationless(f())),
        }
    }

    #[track_caller]
    fn stack(self) -> Self::Output {
        match self {
            Ok(o) => Ok(o),
            Err(err) => Err(Error::from_box(Box::new(err))),
        }
    }

    fn stack_locationless(self) -> Self::Output {
        match self {
            Ok(o) => Ok(o),
            Err(err) => Err(Error::from_box_locationless(Box::new(err))),
        }
    }
}

impl<T> StackableErr for Option<T> {
    type Output = core::result::Result<T, Error>;

    #[track_caller]
    fn stack_err<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output {
        match self {
            Some(o) => Ok(o),
            None => Err(Error::from_kind(f())),
        }
    }

    fn stack_err_locationless<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output {
        match self {
            Some(o) => Ok(o),
            None => Err(Error::from_kind_locationless(f())),
        }
    }

    #[track_caller]
    fn stack(self) -> Self::Output {
        match self {
            Some(o) => Ok(o),
            None => Err(Error::new()),
        }
    }

    fn stack_locationless(self) -> Self::Output {
        match self {
            Some(o) => Ok(o),
            None => Err(Error::empty()),
        }
    }
}

impl StackableErr for Error {
    type Output = core::result::Result<(), Error>;

    #[track_caller]
    fn stack_err<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output {
        Err(self.add_kind(f()))
    }

    fn stack_err_locationless<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output {
        Err(self.add_kind_locationless(f()))
    }

    #[track_caller]
    fn stack(self) -> Self::Output {
        Err(self.add_location())
    }

    fn stack_locationless(self) -> Self::Output {
        Err(self)
    }
}

//impl<E: std::error::Error + Send + Sync + 'static> StackableErr for E

// this causes refactor issues when `T` is changed, and we can't fix this due to
// conflicts, and this might not be good in the sense that `K0` is not wrapped
// by anything indicating potential failure or is representing failure itself
/*
impl<K0: Into<ErrorKind>> StackableErr for K0 {
    type Output = core::result::Result<(), Error>;

    #[track_caller]
    fn stack_err<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output {
        // avoid adding redundant locations
        Err(Error::from_kind(self).add_err_locationless(f()))
    }

    fn stack_err_locationless<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output {
        Err(Error::from_kind_locationless(self).add_err_locationless(f()))
    }

    #[track_caller]
    fn stack(self) -> Self::Output {
        Err(Error::from_kind(self))
    }

    fn stack_locationless(self) -> Self::Output {
        Err(Error::from_kind_locationless(self))
    }
}
*/
