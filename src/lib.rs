//! ```
//! use stacked_errors::{Error, Result, StackableErr};
//!
//! // Note that `Error` uses `ThinVec` internally, which means that it often
//! // takes up only the stack space of a `usize` or the size of the `T` plus
//! // a byte.
//! fn innermost(s: &str) -> Result<u8> {
//!     if s == "return error" {
//!         // When creating the initial `Result<_, Error>` from something that
//!         // is directly representable in a `ErrorKind` (i.e. not needing
//!         // `BoxedErr`), use this `Err(Error::from(...))` format. This
//!         // format is cumbersome relative to the other features of this
//!         // crate, but it is the best solution because of technicalities
//!         // related to trait collisions at the design level, `Result` type
//!         // inference with the return type, wanting to keep the directly
//!         // representable strings outside of a box for performance, and
//!         // because of the `Display` impl which special cases them.
//!
//!         return Err(Error::from("bottom level `StrErr`"))
//!     }
//!     if s == "parse invalid" {
//!         // However, this is the common case where we have some external
//!         // crate function that returns a `Result<..., E: Error>`. We
//!         // usually call `StackableErr::stack_err` if we want to attach
//!         // some message to it right away (it is called with a closure
//!         // so that it doesn't have impact on the `Ok` cases). Otherwise, we
//!         // just call `StackableErr::stack` so that just the location is
//!         // pushed on the stack. We can then use `?` directly.
//!
//!         let _ = ron::from_str("invalid").stack_err(|| format!("parsing error with \"{s}\""))?;
//!     }
//!     Ok(42)
//! }
//!
//! fn inner(s: &str) -> Result<u16> {
//!     // Chainable with other combinators. Use `stack_err` with a message for
//!     // propogating up the stack when the error is something that should
//!     // have some mid layer information attached for it for quick diagnosis
//!     // by the user. Otherwise use just `stack` which will also do error
//!     // conversion if necessary, avoiding needing to wrangle with `map_err`.
//!
//!     let x = innermost(s)
//!         .map(|x| u16::from(x))
//!         .stack_err(|| format!("error from innermost(\"{s}\")"))?;
//!     Ok(x)
//! }
//!
//! fn outer(s: &str) -> Result<u64> {
//!     // ...
//!
//!     let x = inner(s).stack()?;
//!
//!     // ...
//!     Ok(u64::from(x))
//! }
//!
//! let res = format!("{:?}", outer("valid"));
//! assert_eq!(res, "Ok(42)");
//!
//! // The line numbers are slightly off because this is a doc test.
//! // In order from outer to the innermost call, it lists the location of the
//! // `stack` call from `outer`, the location of `stack_err` from `inner`,
//! // the associated error message, the location of either the `Error::from`
//! // or `stack_err` from `innermost`, and finally the root error message.
//!
//! let res = format!("{:?}", outer("return error"));
//! assert_eq!(
//!     res,
//!     r#"Err(Error { stack: [
//! Location { file: "src/lib.rs", line: 54, col: 22 },
//! Location { file: "src/lib.rs", line: 47, col: 10 },
//! error from innermost("return error")
//! Location { file: "src/lib.rs", line: 22, col: 20 },
//! bottom level `StrErr`
//! ] })"#
//! );
//!
//! let res = format!("{:?}", outer("parse invalid"));
//! assert_eq!(
//!     res,
//!     r#"Err(Error { stack: [
//! Location { file: "src/lib.rs", line: 54, col: 22 },
//! Location { file: "src/lib.rs", line: 47, col: 10 },
//! error from innermost("parse invalid")
//! parsing error with "parse invalid"
//! Location { file: "src/lib.rs", line: 33, col: 42 },
//! BoxedError(SpannedError { code: ExpectedUnit, position: Position { line: 1, col: 1 } }),
//! ] })"#
//! );
//! ```
//!
//! Some other partial examples of what using the crate properly looks like:
//!
//! ```text
//! f.map_err(|e| Error::from_box(Box::new(e)))?;
//! // replace the above with
//! f.stack()?;
//! ```
//! ```text
//! f.stack_err(|| ())?;
//! // replace the above with
//! f.stack()?;
//! ```
//! ```text
//! // if needing to push another arbitrary error onto the stack
//! f.stack_err(|| ErrorKind::from_err(arbitrary))?;
//! ```
//! ```text
//! let dir = self
//!     .path
//!     .parent()
//!     .stack_err(|| "FileOptions::preacquire() -> empty path")?
//!     .to_str()
//!     .stack_err(|| "bad OsStr conversion")?;
//! ```
//! ```text
//! option.take()
//!     .stack_err(|| "`Struct` has already had some termination method called")?
//!     .wait_with_output()
//!     .await
//!     .stack_err(|| {
//!         format!("{self:?}.outer_wait_with_output() -> failed when waiting")
//!     })?;
//! ```
//! ```text
//! // strings and some std errors can be created like this,
//! return Err(Error::from(format!(
//!     "failure of {x:?} to complete"
//! )))
//! // otherwise use this (also note that `Error::from*` includes
//! // `#[track_caller]` location, no need to add on a `stack` call)
//! return Err(Error::from_err(needs_boxing))
//! ```
//! ```text
//! // when the error type is already `crate::Error` you can do this if it is
//! // preferable over `map`
//! return match ... {
//!     Ok(ok) => {
//!         ...
//!     }
//!     Err(e) => Err(e.add_kind(format!("wait_for_ok_lookup_host(.., host: {host})"))),
//! }
//! ```
//! ```text
//! // in commonly used functions you may want `_locationless` to avoid adding
//! // on unnecessary information if the location is already being added on
//! return Err(e.add_err_locationless(ErrorKind::TimeoutError)).stack_err(|| {
//!     format!(
//!         "wait_for_ok(num_retries: {num_retries}, delay: {delay:?}) timeout, \
//!          last error stack was:"
//!     )
//! })
//! ```

// TODO when https://github.com/rust-lang/rust/issues/103765 is stabilized
// we can make a large subset as no_std
//#![no_std]

extern crate alloc;
mod error;
/// This is an experimental errors crate.
/// Note: you should probably use `default-features = false` in your
/// `Cargo.toml`
mod error_kind;
mod stackable_err;
use alloc::boxed::Box;
mod fmt;

pub use error::{Error, StackedError};
pub use error_kind::ErrorKind;
pub use fmt::{DisplayShortLocation, DisplayStr};
pub use stackable_err::StackableErr;

/// A shorthand for [core::result::Result<T, crate::Error>]
pub type Result<T> = core::result::Result<T, Error>;

macro_rules! unit_x {
    ($kind:ident $x:ty) => {
        impl From<$x> for ErrorKind {
            fn from(_e: $x) -> Self {
                Self::$kind
            }
        }

        impl From<$x> for Error {
            #[track_caller]
            fn from(e: $x) -> Self {
                Self::from_kind(e)
            }
        }
    };
}

macro_rules! x {
    ($kind:ident $x:ty) => {
        impl From<$x> for ErrorKind {
            fn from(e: $x) -> Self {
                Self::$kind(e)
            }
        }

        impl From<$x> for Error {
            #[track_caller]
            fn from(e: $x) -> Self {
                Self::from_kind(e)
            }
        }
    };
}

#[allow(unused_macros)]
macro_rules! x_box {
    ($kind:ident $x:ty) => {
        impl From<$x> for ErrorKind {
            fn from(e: $x) -> Self {
                Self::$kind(Box::new(e))
            }
        }

        impl From<$x> for Error {
            #[track_caller]
            fn from(e: $x) -> Self {
                Self::from_kind(e)
            }
        }
    };
}

type X0 = ();
unit_x!(UnitError X0);
type X1 = &'static str;
x!(StrError X1);
type X2 = alloc::string::String;
x!(StringError X2);
type X3 = std::io::Error;
x!(StdIoError X3);
type X4 = alloc::string::FromUtf8Error;
x!(FromUtf8Error X4);
type X5 = alloc::string::FromUtf16Error;
x!(FromUtf16Error X5);
type X10 = core::num::ParseIntError;
x!(ParseIntError X10);
type X11 = core::num::ParseFloatError;
x!(ParseFloatError X11);
type X12 = core::num::TryFromIntError;
x!(TryFromIntError X12);
type X13 = Box<dyn std::error::Error + Send + Sync>;
x!(BoxedError X13);

/*
type X = ;
x!(Error X);
*/
