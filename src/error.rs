use core::{fmt, fmt::Debug, panic::Location};

use thin_vec::{thin_vec, ThinVec};

use crate::ErrorKind;

/// An experimental error struct that has an internal stack for different kinds
/// of errors and a stack for locations. This is a replacement for the bad
/// information you get from backtraces within `async` tasks.
///
/// # Note
///
/// Import the `MapAddError` trait and use `.map_add_err` instead of `map_err`
/// or other such functions.
///
/// Use at least `.map_add_err(|| ())` before every time an error is propogated
/// up the stack to make sure the location stack is filled.
pub struct Error {
    // using a ThinVec has many advantages from taking as little space as possible, having single
    // indirection vs. other methods, and having the niche optimizations applied to `Result<(),
    // Error>` and others.
    pub stack: ThinVec<(ErrorKind, Option<&'static Location<'static>>)>,
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // in reverse order of a typical stack, I don't want to have to scroll up to see
        // the more specific errors
        f.write_fmt(format_args!("Error {{ stack: [\n"))?;
        for (i, (error, location)) in self.stack.iter().enumerate().rev() {
            match error {
                ErrorKind::UnitError => (),
                ErrorKind::StrError(s) => {
                    if i == 0 {
                        f.write_fmt(format_args!("{s}\n"))?;
                    } else {
                        f.write_fmt(format_args!("{s} ->\n"))?;
                    }
                }
                ErrorKind::StringError(s) => {
                    if i == 0 {
                        f.write_fmt(format_args!("{s}\n"))?;
                    } else {
                        f.write_fmt(format_args!("{s} ->\n"))?;
                    }
                }
                _ => {
                    f.write_fmt(format_args!("{error:?},\n"))?;
                }
            }
            if let Some(location) = location {
                f.write_fmt(format_args!("{location:?},\n"))?;
            }
        }
        f.write_fmt(format_args!("] }}"))
    }
}

impl Error {
    /// Returns an empty error stack
    pub fn empty() -> Self {
        Self {
            stack: ThinVec::new(),
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

    /// Returns a base timeout error
    #[track_caller]
    pub fn timeout() -> Self {
        Self::from_kind(ErrorKind::TimeoutError)
    }

    /// Can handle anything implementing `std::error::Error`. Most often called
    /// like `Err(Error::boxed(Box::new(e)))` or `.map_err(|e|
    /// Error::boxed(Box::new(e) as Box<dyn std::error::Error>)).map_add_err(||
    /// "more info and a location")?`.
    #[track_caller]
    pub fn boxed(e: alloc::boxed::Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::from_kind(ErrorKind::BoxedError(e))
    }

    /// The same as [Error::add_err] but without pushing location to stack
    pub fn add_err_no_location<K: Into<ErrorKind>>(mut self, kind: K) -> Self {
        self.stack.push((kind.into(), None));
        self
    }

    /// Use `MapAddErr` instead of this if anything expensive in creating the
    /// error is involved, because `map_add_err` uses a closure analogous to
    /// `ok_or_else`.
    #[track_caller]
    pub fn add_err<K: Into<ErrorKind>>(mut self, kind: K) -> Self {
        self.stack.push((kind.into(), Some(Location::caller())));
        self
    }

    /// Only adds `track_caller` location to the stack
    #[track_caller]
    pub fn add_location(mut self) -> Self {
        self.stack
            .push((ErrorKind::UnitError, Some(Location::caller())));
        self
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

    /// Chains the stacks of `other` onto `self`
    pub fn chain_errors(mut self, mut other: Self) -> Self {
        self.stack.append(&mut other.stack);
        self
    }
}
