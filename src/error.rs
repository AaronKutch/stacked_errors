use alloc::boxed::Box;
use core::panic::Location;

use thin_vec::{thin_vec, ThinVec};

use crate::ErrorKind;

/// An error struct that has an internal stack for different kinds of errors and
/// [Location](core::panic::Location)s. This is a replacement for the bad
/// information you get from backtraces within `async` tasks.
///
/// ```
/// use stacked_errors::{Error, Result, StackableErr};
///
/// // Note that `Error` uses `ThinVec` internally, which means that it often
/// // takes up only the space of a `usize` or the size of the `T` plus a byte.
/// fn innermost(s: &str) -> Result<u8> {
///     if s == "return error" {
///         // When creating the initial `Result<_, Error>` from something that
///         // is directly representable in a `ErrorKind` (i.e. not needing
///         // `BoxedErr`), use this `Err(Error::from(...))` format. This
///         // format is cumbersome relative to the other features of this
///         // crate, but it is the best solution because of technicalities
///         // related to trait collisions at the design level, type
///         // inference with the return type, wanting to keep the directly
///         // representable strings outside of a box for performance, and so
///         // that the `Display` impl can make use of them.
///
///         return Err(Error::from("bottom level `StrErr`"))
///     }
///     if s == "eval invalid" {
///         // However, this is the common case where we have some external
///         // crate function that returns a `Result<..., E: Error>`. We
///         // usually call `StackableErr::stack_err` if we want to attach
///         // some message to it right away (it is called with a closure
///         // so that it doesn't have impact on the `Ok` cases). Otherwise, we
///         // just call `StackableErr::stack` so that the location is pushed
///         // on the stack. We can then use `?` directly.
///
///         let _ = ron::from_str("invalid").stack_err(|| format!("parsing error with \"{s}\""))?;
///     }
///
///     Ok(42)
/// }
/// ```
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
