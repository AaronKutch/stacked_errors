//! A crate for high level error propogation with software controlled backtraces
//! that are entirely independent of the `RUST_BACKTRACE` system.
//!
//! In Rust development, major crates will often have their own error enums that
//! work well in their own specialized domain, but when orchestrating many
//! domains together we run into issues. `map_err` is very annoying to work
//! with. In `async` call stacks or cases where we can't easily control
//! `RUST_BACKTRACE`, we run into an especially annoying problem
//! where the same kind of error can be returned from multiple places, and we
//! are sometimes forced into `println` debugging to find out where it is
//! actually from. This crate introduces the `StackableErr` trait and a
//! "stackable" error type that allows for both software-defined error
//! backtraces and easily converting errors into the stackable error type.
//!
//! This crate is similar to `eyre`, but has a more efficient internal layout
//! with a `ThinVec` array of `SmallBox`es, works with `no_std`, implements
//! `core::error::Error`, and more.
//!
//! Some partial examples of what using the crate looks like:
//!
//! ```text
//! f.map_err(|e| Error::from_err(e))?;
//! // replace the above with
//! f.stack()?; // uses `#[track_caller]` when an error is being propagated
//! ```
//! ```text
//! let dir = self
//!     .path
//!     .parent()
//!     .stack_err("FileOptions::preacquire() -> empty path")?
//!     .to_str()
//!     .stack_err("bad OsStr conversion")?;
//! ```
//! ```text
//! // arbitrary things implementing `Display + Send + Sync + 'static` can be stacked
//! f.stack_err(arbitrary)?;
//! // readily swappable with `anyhow` and `eyre` due to extra method, trait, and
//! // struct aliases
//! f.wrap_err(arbitrary)?;
//! f.context(arbitrary)?;
//! ```
//! ```text
//! // The trait is implemented for options so that we don't need `OptionExt` like
//! // `eyre` does
//! option.take()
//!     .stack_err("`Struct` has already been taken")?
//!     .wait_with_output()
//!     .await
//!     .stack_err_with(|| {
//!         format!("{self:?}.xyz() -> failed when waiting")
//!     })?;
//! ```
//! ```text
//! return Err(Error::from_err(format!(
//!     "failure of {x:?} to complete"
//! )))
//! // replace the above with
//! bail!("failure of {x:?} to complete")
//! ```
//! ```text
//! // when the error type is already `stacked_errors::Error` you can do this if it is
//! // preferable over `map`
//! return match ... {
//!     Ok(ok) => {
//!         ...
//!     }
//!     Err(e) => Err(e.add_err(format!("myfunction(.., host: {host})"))),
//! }
//! ```
//!
//! ```
//! use stacked_errors::{bail, Error, Result, StackableErr};
//!
//! // Note that `Error` uses `ThinVec` internally, which means that it often
//! // takes up only the stack space of a `usize` or the size of the `T` plus
//! // a byte.
//! fn innermost(s: &str) -> Result<u8> {
//!     if s == "return error" {
//!         bail!("bottom level `StrErr`")
//!     }
//!     if s == "parse invalid" {
//!         // This is the common case where we have some external
//!         // crate function that returns a `Result<..., E: Error>`. We
//!         // usually call `StackableErr::stack_err` if we want to attach
//!         // some message to it right away (it is called with a closure
//!         // so that it doesn't have impact on the `Ok` cases). Otherwise, we
//!         // just call `StackableErr::stack` so that just the location is
//!         // pushed on the stack. We can then use `?` directly.
//!
//!         let _: () = ron::from_str("invalid")
//!             .stack_err_with(|| format!("parsing error with \"{s}\""))?;
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
//!         .stack_err_with(|| format!("error from innermost(\"{s}\")"))?;
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
//! // note that debug mode (used when returning errors from the main function)
//! // includes terminal styling
//! println!("{:?}", outer("return error"));
//! let res = format!("{}", outer("return error").unwrap_err());
//! assert_eq!(
//!     res,
//!     r#"
//!   at src/lib.rs 45:22
//!     error from innermost("return error") at src/lib.rs 38:10
//!     bottom level `StrErr` at src/lib.rs 12:9"#
//! );
//!
//! println!("{:?}", outer("parse invalid"));
//! let res = format!("{}", outer("parse invalid").unwrap_err());
//! assert_eq!(
//!     res,
//!     r#"
//!   at src/lib.rs 45:22
//!     error from innermost("parse invalid") at src/lib.rs 38:10
//!     parsing error with "parse invalid" at src/lib.rs 24:14
//!     1:1: Expected unit"#
//! );
//! ```
//!
//! ```text
//! // in commonly used functions you may want `_locationless` to avoid adding
//! // on unnecessary information if the location is already being added on
//! return Err(e.add_err_locationless(ErrorKind::TimeoutError)).stack_err_with(|| {
//!     format!(
//!         "wait_for_ok(num_retries: {num_retries}, delay: {delay:?}) timeout, \
//!          last error stack was:"
//!     )
//! })
//! ```

#![no_std]

extern crate alloc;
mod error;
mod fmt;
mod macros;
mod special;
mod stackable_err;

pub use error::{Error, StackableErrorTrait, StackedError, StackedErrorDowncast};
pub use fmt::{shorten_location, DisplayStr};
pub use special::*;
pub use stackable_err::StackableErr;

/// A shorthand for [core::result::Result<T, stacked_errors::Error>]
pub type Result<T> = core::result::Result<T, Error>;

/// used by the macros
#[doc(hidden)]
pub mod __private {
    pub use alloc::format;
    pub use core::{concat, format_args, stringify};

    #[track_caller]
    pub fn format_err(args: core::fmt::Arguments<'_>) -> crate::Error {
        let fmt_arguments_as_str = args.as_str();

        if let Some(message) = fmt_arguments_as_str {
            // &'static str
            crate::Error::from_err(message)
        } else {
            // interpolation
            crate::Error::from_err(alloc::fmt::format(args))
        }
    }

    pub fn format_err_locationless(args: core::fmt::Arguments<'_>) -> crate::Error {
        let fmt_arguments_as_str = args.as_str();

        if let Some(message) = fmt_arguments_as_str {
            // &'static str
            crate::Error::from_err_locationless(message)
        } else {
            // interpolation
            crate::Error::from_err_locationless(alloc::fmt::format(args))
        }
    }
}
