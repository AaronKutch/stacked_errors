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

impl<T> StackableErr for core::result::Result<T, Error> {
    type Output = core::result::Result<T, Error>;

    #[track_caller]
    fn stack_err<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output {
        match self {
            Ok(o) => Ok(o),
            Err(e) => Err(e.add_err(f())),
        }
    }

    fn stack_err_locationless<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output {
        match self {
            Ok(o) => Ok(o),
            Err(e) => Err(e.add_err_locationless(f())),
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
            Err(err) => Err(Error::from_box(Box::new(err)).add_err_locationless(f())),
        }
    }

    fn stack_err_locationless<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output {
        match self {
            Ok(o) => Ok(o),
            Err(err) => Err(Error::from_box_locationless(Box::new(err)).add_err_locationless(f())),
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
            None => Err(Error::new_locationless()),
        }
    }
}

impl StackableErr for Error {
    type Output = core::result::Result<(), Error>;

    #[track_caller]
    fn stack_err<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output {
        Err(self.add_err(f()))
    }

    fn stack_err_locationless<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output {
        Err(self.add_err_locationless(f()))
    }

    #[track_caller]
    fn stack(self) -> Self::Output {
        Err(self.add_location())
    }

    fn stack_locationless(self) -> Self::Output {
        Err(self)
    }
}

impl<K0: Into<ErrorKind>> StackableErr for K0 {
    type Output = core::result::Result<(), Error>;

    #[track_caller]
    fn stack_err<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output {
        // avoid adding redundant locations
        Err(Error::from_kind_locationless(self).add_err(f()))
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