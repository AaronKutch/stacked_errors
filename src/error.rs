use alloc::boxed::Box;
use core::panic::Location;

use thin_vec::{thin_vec, ThinVec};

use crate::ErrorKind;

/// An error struct intended for high level error propogation with programmable
/// backtraces
///
/// For lower level error propogation, you should still use ordinary [Option]
/// and [Result] with domain-specific enums, it is only when using OS-level
/// functions or when multiple domains converge that this is intended to be
/// used. This has an internal stack for different kinds of arbitrary lower
/// level errors and [Location](core::panic::Location)s. When used with the
/// [StackableErr](crate::StackableErr) trait, this enables easy conversion and
/// software defined backtraces for better `async` debugging. See the crate docs
/// for more.
///
/// Note that due to trait conflicts and not wanting users to accidentally
/// wastefully embed `stacked_errors::Error` in a `BoxedErr` of another
/// `stacked_errors::Error`, `stacked_errors::Error` itself does not actually
/// implement [core::error::Error]. This does not pose a problem in most cases
/// since it is intended to be the highest level of error that is directly
/// returned or panicked on. However, if a user needs the end result struct to
/// implement [core::error::Error], they can use the
/// [StackedError](crate::StackedError) wrapper.
pub struct Error {
    /// Using a ThinVec has advantages such as taking as little space as
    /// possible on the stack (since we are commiting to some indirection at
    /// this point), and having the niche optimizations applied to things like
    /// `Result<(), Error>`.
    pub stack: ThinVec<(ErrorKind, Option<&'static Location<'static>>)>,
}

/// Wraps around [stacked_errors::Error](crate::Error) to implement
/// [core::error::Error], since [stacked_errors::Error](crate::Error) itself
/// cannot implement the trait.
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

    /// Returns an error stack with just `kind`. The `impl From<_> for Error`
    /// implementations can usually be used in place of this.
    #[track_caller]
    pub fn from_kind<K: Into<ErrorKind>>(kind: K) -> Self {
        Self {
            stack: thin_vec![(kind.into(), Some(Location::caller()))],
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
    pub fn from_box(e: Box<dyn core::error::Error + Send + Sync>) -> Self {
        Self::from_kind(ErrorKind::BoxedError(e))
    }

    /// Same as [Error::from_box] but without location.
    pub fn from_box_locationless(e: Box<dyn core::error::Error + Send + Sync>) -> Self {
        Self::from_kind_locationless(ErrorKind::BoxedError(e))
    }

    /// Returns an error stack with a `BoxedError` around `e`, and location
    /// info. [Error::from_kind] or [Error::from] is more efficient and should
    /// be used instead if the type implements `Into<ErrorKind>`.
    #[track_caller]
    pub fn box_from<E: core::error::Error + Send + Sync + 'static>(e: E) -> Self {
        Self {
            stack: thin_vec![(ErrorKind::BoxedError(Box::new(e)), Some(Location::caller()))],
        }
    }

    /// Same as [Error::box_from] but without location.
    pub fn box_from_locationless<E: core::error::Error + Send + Sync + 'static>(e: E) -> Self {
        Self {
            stack: thin_vec![(ErrorKind::BoxedError(Box::new(e)), None)],
        }
    }

    /// Adds `kind` to the error stack alongside location information. Use
    /// `StackableErr` instead of this if anything expensive in creating the
    /// error is involved, because `stack_err` uses a closure analogous to
    /// `ok_or_else`.
    #[track_caller]
    pub fn add_kind<K: Into<ErrorKind>>(mut self, kind: K) -> Self {
        self.stack.push((kind.into(), Some(Location::caller())));
        self
    }

    /// Same as [Error::add_kind] but without location.
    pub fn add_kind_locationless<K: Into<ErrorKind>>(mut self, kind: K) -> Self {
        self.stack.push((kind.into(), None));
        self
    }

    /// Adds `e` to the error stack alongside location information. Use
    /// `StackableErr` instead of this if anything expensive in creating the
    /// error is involved, because `stack_err` uses a closure analogous to
    /// `ok_or_else`.
    #[track_caller]
    pub fn add_box(mut self, e: Box<dyn core::error::Error + Send + Sync>) -> Self {
        self.stack
            .push((ErrorKind::BoxedError(e), Some(Location::caller())));
        self
    }

    /// Same as [Error::add_box] but without location.
    pub fn add_box_locationless(mut self, e: Box<dyn core::error::Error + Send + Sync>) -> Self {
        self.stack.push((ErrorKind::BoxedError(e), None));
        self
    }

    /// Boxes a type implementing `core::error::Error + Send + Sync + 'static`
    /// and adds it with location data to the stack. [Error::add_kind] is
    /// preferred if possible, this is a shorthand for
    /// `stack.add_kind(ErrorKind::from_box(Box::new(e)))` where the error does
    /// not implement `Into<ErrorKind>` and needs to be boxed.
    #[track_caller]
    pub fn box_and_add<E: core::error::Error + Send + Sync + 'static>(mut self, e: E) -> Self {
        self.stack
            .push((ErrorKind::BoxedError(Box::new(e)), Some(Location::caller())));
        self
    }

    /// Same as [Error::box_and_add] but without location.
    pub fn box_and_add_locationless<E: core::error::Error + Send + Sync + 'static>(
        mut self,
        e: E,
    ) -> Self {
        self.stack.push((ErrorKind::BoxedError(Box::new(e)), None));
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
