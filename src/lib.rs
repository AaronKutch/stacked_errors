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

pub use error::Error;
pub use error_kind::ErrorKind;
pub use fmt::{DisplayShortLocation, DisplayStr};
pub use stackable_err::StackableErr;

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
