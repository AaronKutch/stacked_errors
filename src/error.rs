use alloc::boxed::Box;
use core::panic::Location;

use thin_vec::{thin_vec, ThinVec};

use crate::ErrorKind;

/// An error struct that has an internal stack for different kinds of errors and
/// [Location](core::panic::Location)s. This is a replacement for the bad
/// information you get from backtraces within `async` tasks.
pub struct Error {
    // using a ThinVec has many advantages from taking as little space as possible, having single
    // indirection vs. other methods, and having the niche optimizations applied to `Result<(),
    // Error>` and others.
    pub stack: ThinVec<(ErrorKind, Option<&'static Location<'static>>)>,
}

/// Due to trait conflicts and not wanting users to accidentally embed
/// `crate::Error` in a `BoxedErr` of another `crate::Error`, `crate::Error`
/// itself does not actually implement `std::error::Error`. This does not pose a
/// problem in most cases, since `main` functions can return `Return<T, Error>`.
/// However, if a user absolutely needs an end result struct implementing
/// `std::error::Error`, they can use this wrapper.
#[derive(Debug, thiserror::Error)]
pub struct StackedError(pub Error);

/// Note: in most cases you can use `Error::from` or a call from `StackableErr`
/// instead of these functions.
impl Error {
    /// Returns an empty error stack
    pub fn empty() -> Self {
        Self {
            stack: ThinVec::new(),
        }
    }

    /// Returns an error stack with just a `UnitError` and location information
    #[track_caller]
    pub fn new() -> Self {
        Self::from_kind(ErrorKind::UnitError)
    }

    /// Returns an error stack with a `BoxedError` around `e`, and location
    /// info.
    #[track_caller]
    pub fn from_err<E: std::error::Error + Send + Sync + 'static>(e: E) -> Self {
        let l = Location::caller();
        Self {
            stack: thin_vec![(ErrorKind::BoxedError(Box::new(e)), Some(l))],
        }
    }

    /// Same as [Error::from_err] but without location.
    pub fn from_err_locationless<E: std::error::Error + Send + Sync + 'static>(e: E) -> Self {
        Self {
            stack: thin_vec![(ErrorKind::BoxedError(Box::new(e)), None)],
        }
    }

    /// Returns an error stack with just `kind`.
    #[track_caller]
    pub fn from_kind<K: Into<ErrorKind>>(kind: K) -> Self {
        let l = Location::caller();
        Self {
            stack: thin_vec![(kind.into(), Some(l))],
        }
    }

    /// Same as [Error::from_kind] but without location.
    pub fn from_kind_locationless<K: Into<ErrorKind>>(kind: K) -> Self {
        Self {
            stack: thin_vec![(kind.into(), None)],
        }
    }

    /// Returns an error stack with just a `BoxedErr`.
    #[track_caller]
    pub fn from_box(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::from_kind(ErrorKind::BoxedError(e))
    }

    /// Same as [Error::from_box] but without location.
    pub fn from_box_locationless(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::from_kind_locationless(ErrorKind::BoxedError(e))
    }

    /// Adds `kind` to the error stack alongside location information. Use
    /// `StackableErr` instead of this if anything expensive in creating the
    /// error is involved, because `stack_err` uses a closure analogous to
    /// `ok_or_else`.
    #[track_caller]
    pub fn add_err<K: Into<ErrorKind>>(mut self, kind: K) -> Self {
        self.stack.push((kind.into(), Some(Location::caller())));
        self
    }

    /// Same as [Error::add_err] but without location.
    pub fn add_err_locationless<K: Into<ErrorKind>>(mut self, kind: K) -> Self {
        self.stack.push((kind.into(), None));
        self
    }

    /// Only adds `track_caller` location to the stack
    #[track_caller]
    pub fn add_location(mut self) -> Self {
        self.stack
            .push((ErrorKind::UnitError, Some(Location::caller())));
        self
    }

    /// Moves the stack of `other` onto `self`
    pub fn chain_errors(mut self, mut other: Self) -> Self {
        self.stack.append(&mut other.stack);
        self
    }

    /// Returns a base `TimeoutError` error
    #[track_caller]
    pub fn timeout() -> Self {
        Self::from_kind(ErrorKind::TimeoutError)
    }

    /// Returns a base `ProbablyNotRootCauseError` error
    #[track_caller]
    pub fn probably_not_root_cause() -> Self {
        Self::from_kind(ErrorKind::ProbablyNotRootCauseError)
    }

    /// Returns if a `TimeoutError` is in the error stack
    pub fn is_timeout(&self) -> bool {
        for (error, _) in &self.stack {
            if matches!(error, ErrorKind::TimeoutError) {
                return true
            }
        }
        false
    }

    /// Returns if a `ProbablyNotRootCauseError` is in the error stack
    pub fn is_probably_not_root_cause(&self) -> bool {
        for (error, _) in &self.stack {
            if matches!(error, ErrorKind::ProbablyNotRootCauseError) {
                return true
            }
        }
        false
    }
}

impl Default for Error {
    #[track_caller]
    fn default() -> Self {
        Error::new()
    }
}
