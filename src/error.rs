use alloc::boxed::Box;
use core::panic::Location;

use thin_vec::{thin_vec, ThinVec};

use crate::ErrorKind;

/// An experimental error struct that has an internal stack for different kinds
/// of errors and [Location](core::panic::Location)s. This is a replacement for
/// the bad information you get from backtraces within `async` tasks.
///
/// # Note
///
/// Import the `StackableErr` trait and use `.stack_err` or the other functions.
///
/// Use at least `.stack()` before every time an error is propogated
/// up the stack to make sure the location stack is filled.
pub struct Error {
    // using a ThinVec has many advantages from taking as little space as possible, having single
    // indirection vs. other methods, and having the niche optimizations applied to `Result<(),
    // Error>` and others.
    pub stack: ThinVec<(ErrorKind, Option<&'static Location<'static>>)>,
}

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

    pub fn new_locationless() -> Self {
        Self::from_kind_locationless(ErrorKind::UnitError)
    }

    #[track_caller]
    pub fn from_err<E: std::error::Error + Send + Sync + 'static>(e: E) -> Self {
        let l = Location::caller();
        Self {
            stack: thin_vec![(ErrorKind::BoxedError(Box::new(e)), Some(l))],
        }
    }

    pub fn from_err_locationless<E: std::error::Error + Send + Sync + 'static>(e: E) -> Self {
        Self {
            stack: thin_vec![(ErrorKind::BoxedError(Box::new(e)), None)],
        }
    }

    /// Use `MapAddErr` instead of this
    #[track_caller]
    pub fn from_kind<K: Into<ErrorKind>>(kind: K) -> Self {
        let l = Location::caller();
        Self {
            stack: thin_vec![(kind.into(), Some(l))],
        }
    }

    /// Use `MapAddErr` instead of this
    pub fn from_kind_locationless<K: Into<ErrorKind>>(kind: K) -> Self {
        Self {
            stack: thin_vec![(kind.into(), None)],
        }
    }

    #[track_caller]
    pub fn from_box(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::from_kind(ErrorKind::BoxedError(e))
    }

    pub fn from_box_locationless(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::from_kind_locationless(ErrorKind::BoxedError(e))
    }

    /// Use `StackableErr` instead of this if anything expensive in creating the
    /// error is involved, because `stack_err` uses a closure analogous to
    /// `ok_or_else`.
    #[track_caller]
    pub fn add_err<K: Into<ErrorKind>>(mut self, kind: K) -> Self {
        self.stack.push((kind.into(), Some(Location::caller())));
        self
    }

    /// The same as [Error::add_err] but without pushing location to stack
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

    /// Chains the stacks of `other` onto `self`
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
