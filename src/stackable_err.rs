use core::{fmt::Display, mem};

use crate::{Error, StackableErrorTrait};

/// Conversion to and addition to the stack of a
/// [stackable_error::Error](crate::Error).
///
/// See the main crate documentation and implementation for examples.
pub trait StackableErr {
    type Output;

    /// Pushes just location information to the error stack
    fn stack(self) -> Self::Output;

    /// Only converts to `Self::Output` and pushes it on the error stack
    fn stack_locationless(self) -> Self::Output;

    /// Pushes the result of `f` and location information to the error stack
    fn stack_err<E: Display + Send + Sync + 'static>(self, e: E) -> Self::Output;

    /// Pushes `e` and location information to the error stack
    fn stack_err_with<E: Display + Send + Sync + 'static, F: FnOnce() -> E>(
        self,
        f: F,
    ) -> Self::Output;

    /// Pushes `e` without location information to the error stack
    fn stack_err_locationless<E: Display + Send + Sync + 'static>(self, e: E) -> Self::Output;

    /// Pushes the result of `f` without location information to the error stack
    fn stack_err_with_locationless<E: Display + Send + Sync + 'static, F: FnOnce() -> E>(
        self,
        f: F,
    ) -> Self::Output;

    /// Alternate for [StackableErr::stack_err] which can be used for easier
    /// translation to and from the `eyre` crate
    fn wrap_err<D: Display + Send + Sync + 'static>(self, msg: D) -> Self::Output;

    /// Alternate for [StackableErr::stack_err_with] which can be used for
    /// easier translation to and from the `eyre` crate
    fn wrap_err_with<D: Display + Send + Sync + 'static, F: FnOnce() -> D>(
        self,
        msg: F,
    ) -> Self::Output;
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
// `impl<T, E: core::error::Error + Send + Sync + 'static> StackableErr for
// core::result::Result<T, E>` impl on any `Into<ErrorKind>` types

/*impl<T> StackableErr for core::result::Result<T, Error> {
    type Output = core::result::Result<T, Error>;

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

    #[track_caller]
    fn stack_err<E: Display + Send + Sync + 'static>(self, e: E) -> Self::Output {
        match self {
            Ok(o) => Ok(o),
            Err(e) => Err(e.add_kind(f())),
        }
    }

    fn stack_err_locationless<E: Display + Send + Sync + 'static>(self, e: E) -> Self::Output {
        match self {
            Ok(o) => Ok(o),
            Err(e) => Err(e.add_kind_locationless(f())),
        }
    }
}*/

#[track_caller]
fn stack<E: Display + Send + Sync + 'static>(mut err: E) -> Error {
    let tmp: &mut dyn StackableErrorTrait = &mut err;
    if let Some(tmp) = tmp._as_any_mut().downcast_mut::<Error>() {
        tmp.stack_inner();
        // TODO does the allocation here optimize away or can we do something about
        // this?
        mem::take(tmp)
    } else {
        Error::from_err(err)
    }
}

fn stack_locationless<E: Display + Send + Sync + 'static>(mut err: E) -> Error {
    let tmp: &mut dyn StackableErrorTrait = &mut err;
    if let Some(tmp) = tmp._as_any_mut().downcast_mut::<Error>() {
        tmp.stack_locationless_inner();
        mem::take(tmp)
    } else {
        Error::from_err_locationless(err)
    }
}

#[track_caller]
fn stack_err<E: Display + Send + Sync + 'static, E1: Display + Send + Sync + 'static>(
    mut err: E,
    e: E1,
) -> Error {
    let tmp: &mut dyn StackableErrorTrait = &mut err;
    if let Some(tmp) = tmp._as_any_mut().downcast_mut::<Error>() {
        tmp.stack_err_inner(e);
        mem::take(tmp)
    } else {
        Error::from_err(err)
    }
}

#[track_caller]
fn stack_err_locationless<
    E: Display + Send + Sync + 'static,
    E1: Display + Send + Sync + 'static,
>(
    mut err: E,
    e: E1,
) -> Error {
    let tmp: &mut dyn StackableErrorTrait = &mut err;
    if let Some(tmp) = tmp._as_any_mut().downcast_mut::<Error>() {
        tmp.stack_err_locationless_inner(e);
        mem::take(tmp)
    } else {
        Error::from_err_locationless(err)
    }
}

impl<T, E: Display + Send + Sync + 'static> StackableErr for core::result::Result<T, E> {
    type Output = core::result::Result<T, Error>;

    #[track_caller]
    fn stack(self) -> Self::Output {
        match self {
            Ok(o) => Ok(o),
            Err(err) => Err(stack(err)),
        }
    }

    fn stack_locationless(self) -> Self::Output {
        match self {
            Ok(o) => Ok(o),
            Err(err) => Err(stack_locationless(err)),
        }
    }

    #[track_caller]
    fn stack_err<E1: Display + Send + Sync + 'static>(self, e: E1) -> Self::Output {
        match self {
            Ok(o) => Ok(o),
            Err(err) => Err(stack_err(err, e)),
        }
    }

    #[track_caller]
    fn stack_err_with<E1: Display + Send + Sync + 'static, F: FnOnce() -> E1>(
        self,
        f: F,
    ) -> Self::Output {
        match self {
            Ok(o) => Ok(o),
            Err(err) => Err(stack_err(err, f())),
        }
    }

    fn stack_err_locationless<E1: Display + Send + Sync + 'static>(self, e: E1) -> Self::Output {
        match self {
            Ok(o) => Ok(o),
            Err(err) => Err(stack_err_locationless(err, e)),
        }
    }

    fn stack_err_with_locationless<E1: Display + Send + Sync + 'static, F: FnOnce() -> E1>(
        self,
        f: F,
    ) -> Self::Output {
        match self {
            Ok(o) => Ok(o),
            Err(err) => Err(stack_err_locationless(err, f())),
        }
    }

    #[track_caller]
    fn wrap_err<D: Display + Send + Sync + 'static>(self, msg: D) -> Self::Output {
        self.stack_err(msg)
    }

    fn wrap_err_with<D: Display + Send + Sync + 'static, F: FnOnce() -> D>(
        self,
        msg: F,
    ) -> Self::Output {
        self.stack_err_with(msg)
    }
}

impl<T> StackableErr for Option<T> {
    type Output = core::result::Result<T, Error>;

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

    #[track_caller]
    fn stack_err<E1: Display + Send + Sync + 'static>(self, e: E1) -> Self::Output {
        match self {
            Some(o) => Ok(o),
            None => Err(Error::from_err(e)),
        }
    }

    #[track_caller]
    fn stack_err_with<E1: Display + Send + Sync + 'static, F: FnOnce() -> E1>(
        self,
        f: F,
    ) -> Self::Output {
        match self {
            Some(o) => Ok(o),
            None => Err(Error::from_err(f())),
        }
    }

    fn stack_err_locationless<E1: Display + Send + Sync + 'static>(self, e: E1) -> Self::Output {
        match self {
            Some(o) => Ok(o),
            None => Err(Error::from_err_locationless(e)),
        }
    }

    fn stack_err_with_locationless<E1: Display + Send + Sync + 'static, F: FnOnce() -> E1>(
        self,
        f: F,
    ) -> Self::Output {
        match self {
            Some(o) => Ok(o),
            None => Err(Error::from_err_locationless(f())),
        }
    }

    #[track_caller]
    fn wrap_err<D: Display + Send + Sync + 'static>(self, msg: D) -> Self::Output {
        self.stack_err(msg)
    }

    fn wrap_err_with<D: Display + Send + Sync + 'static, F: FnOnce() -> D>(
        self,
        msg: F,
    ) -> Self::Output {
        self.stack_err_with(msg)
    }
}
