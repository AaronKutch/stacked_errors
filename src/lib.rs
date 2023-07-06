/// This is a very WIP experimental errors crate.
/// Note: you should probably use `default-features = false` in your
/// `Cargo.toml`
use std::{
    fmt::{self, Debug},
    panic::Location,
};

use thin_vec::{thin_vec, ThinVec};

/// In the future we plan on having almost every kind of error here under
/// different feature gates. Please file an issue if you would like to include
/// something.
///
/// The intention with `TimeoutError` is that if it is in the error stack, a
/// timeout occured. When other timeout structs are used, this should be added
/// on.
#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    // used for special cases where we need something
    #[error("UnitError")]
    UnitError,
    #[error("TimeoutError")]
    TimeoutError,
    #[error("StrError")]
    StrError(&'static str),
    #[error("StringError")]
    StringError(String),
    #[error("BoxedError")]
    BoxedError(Box<dyn std::error::Error + Send + Sync>),
    #[error("TryFromIntError")]
    TryFromIntError(std::num::TryFromIntError),
    #[error("StdIoError")]
    StdIoError(std::io::Error),
    #[error("FromUtf8Error")]
    FromUtf8Error(std::string::FromUtf8Error),
    // this is more obscure but I think we should keep it because it deals with bad strings, and
    // we don't want that in our generic string errors.
    #[error("FromUtf16Error")]
    FromUtf16Error(std::string::FromUtf16Error),
    #[error("ParseIntError")]
    ParseIntError(std::num::ParseIntError),
    #[error("ParseFloatError")]
    ParseFloatError(std::num::ParseFloatError),
    #[cfg(feature = "tokio_rt_support")]
    #[error("TokioJoinError")]
    TokioJoinError(tokio::task::JoinError),
    // Borsh effecively uses `std::io::Error`
    #[cfg(feature = "ron_support")]
    #[error("RonError")]
    RonError(ron::error::Error),
    #[cfg(feature = "serde_json_support")]
    #[error("SerdeJsonError")]
    SerdeJsonError(serde_json::Error),
    #[cfg(feature = "ctrlc_support")]
    #[error("CtrlcError")]
    CtrlcError(ctrlc::Error),
    #[cfg(feature = "toml_support")]
    #[error("TomlDeError")]
    TomlDeError(toml::de::Error),
    #[cfg(feature = "toml_support")]
    #[error("TomlSerError")]
    TomlSerError(toml::ser::Error),
    #[cfg(feature = "serde_yaml_support")]
    #[error("SerdeYamlError")]
    SerdeYamlError(serde_yaml::Error),
    #[cfg(feature = "reqwest_support")]
    #[error("ReqwestError")]
    ReqwestError(reqwest::Error),
    #[cfg(feature = "hyper_support")]
    #[error("HyperError")]
    HyperError(hyper::Error),
}

/// This is boxed inside an `Error` to make sure that all function signatures
/// involving it aren't inflated. See `Error` for helper functions.
pub struct ErrorInner {
    pub stack: ThinVec<(ErrorKind, Option<&'static Location<'static>>)>,
}

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
pub struct Error(pub ErrorInner);

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // in reverse order of a typical stack, I don't want to have to scroll up to see
        // the more specific errors
        f.write_fmt(format_args!("Error {{ stack: [\n"))?;
        for (i, (error, location)) in self.0.stack.iter().enumerate().rev() {
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
    /// Use `MapAddErr` instead of this
    #[track_caller]
    pub fn from_kind<K: Into<ErrorKind>>(kind: K) -> Self {
        let l = Location::caller();
        Self(ErrorInner {
            stack: thin_vec![(kind.into(), Some(l))],
        })
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
    pub fn boxed(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::from_kind(ErrorKind::BoxedError(e))
    }

    /// The same as [Error::add_err] but without pushing location to stack
    pub fn add_err_no_location<K: Into<ErrorKind>>(mut self, kind: K) -> Self {
        self.0.stack.push((kind.into(), None));
        self
    }

    /// Use `MapAddErr` instead of this if anything expensive in creating the
    /// error is involved, because `map_add_err` uses a closure analogous to
    /// `ok_or_else`.
    #[track_caller]
    pub fn add_err<K: Into<ErrorKind>>(mut self, kind: K) -> Self {
        self.0.stack.push((kind.into(), Some(Location::caller())));
        self
    }

    /// Only adds `track_caller` location to the stack
    #[track_caller]
    pub fn add_location(mut self) -> Self {
        self.0
            .stack
            .push((ErrorKind::UnitError, Some(Location::caller())));
        self
    }

    /// Returns if a `TimeoutError` is in the error stack
    pub fn is_timeout(&self) -> bool {
        for (error, _) in &self.0.stack {
            if matches!(error, ErrorKind::TimeoutError) {
                return true
            }
        }
        false
    }

    /// Chains the stacks of `other` onto `self`
    pub fn chain_errors(mut self, mut other: Self) -> Self {
        self.0.stack.append(&mut other.0.stack);
        self
    }
}

/// The intention of this trait is to convert `Option<T>`s, `Result<T, Error>`s,
/// and `Result<T, impl Into<ErrorKind>>`s into `Result<T, Error>`s with the
/// error having an `ErrorKind` and a location pushed onto its stacks
/// (`map_add_err` should have `#[track_caller]` on it and push on the
/// `Location::caller()`). You can also call `map_add_err` on plain `Error`s and
/// `impl Into<ErrorKind>`s to get a `Result<(), Error>`.
pub trait MapAddError {
    type Output;

    fn map_add_err<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output;
}

impl<T> MapAddError for core::result::Result<T, Error> {
    type Output = core::result::Result<T, Error>;

    #[track_caller]
    fn map_add_err<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output {
        match self {
            Ok(o) => Ok(o),
            Err(e) => Err(e.add_err(f())),
        }
    }
}

impl<T> MapAddError for Option<T> {
    type Output = core::result::Result<T, Error>;

    #[track_caller]
    fn map_add_err<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output {
        match self {
            Some(o) => Ok(o),
            None => Err(Error::from_kind(f())),
        }
    }
}

impl<T, K0: Into<ErrorKind>> MapAddError for core::result::Result<T, K0> {
    type Output = core::result::Result<T, Error>;

    /// Transforms `Result<T, K0>` into `Result<T, Error>` while adding location
    /// information and a second kind of error.
    #[track_caller]
    fn map_add_err<K1: Into<ErrorKind>, F: FnOnce() -> K1>(self, f: F) -> Self::Output {
        match self {
            Ok(o) => Ok(o),
            Err(kind) => Err(Error::from_kind(kind).add_err_no_location(f())),
        }
    }
}

impl MapAddError for Error {
    type Output = core::result::Result<(), Error>;

    #[track_caller]
    fn map_add_err<K: Into<ErrorKind>, F: FnOnce() -> K>(self, f: F) -> Self::Output {
        Err(self.add_err(f()))
    }
}

impl<K0: Into<ErrorKind>> MapAddError for K0 {
    type Output = core::result::Result<(), Error>;

    #[track_caller]
    fn map_add_err<K1: Into<ErrorKind>, F: FnOnce() -> K1>(self, f: F) -> Self::Output {
        Err(Error::from_kind(self).add_err(f()))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

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

type X0 = ();
unit_x!(UnitError X0);
type X1 = &'static str;
x!(StrError X1);
type X2 = String;
x!(StringError X2);
type X3 = std::io::Error;
x!(StdIoError X3);
type X4 = std::string::FromUtf8Error;
x!(FromUtf8Error X4);
type X5 = std::string::FromUtf16Error;
x!(FromUtf16Error X5);
#[cfg(feature = "tokio_rt_support")]
type X6 = tokio::task::JoinError;
#[cfg(feature = "tokio_rt_support")]
x!(TokioJoinError X6);
#[cfg(feature = "serde_json_support")]
type X7 = serde_json::Error;
#[cfg(feature = "serde_json_support")]
x!(SerdeJsonError X7);
#[cfg(feature = "ron_support")]
type X8 = ron::error::Error;
#[cfg(feature = "ron_support")]
x!(RonError X8);
#[cfg(feature = "ctrlc_support")]
type X9 = ctrlc::Error;
#[cfg(feature = "ctrlc_support")]
x!(CtrlcError X9);
type X10 = std::num::ParseIntError;
x!(ParseIntError X10);
type X11 = std::num::ParseFloatError;
x!(ParseFloatError X11);
type X12 = std::num::TryFromIntError;
x!(TryFromIntError X12);
type X13 = Box<dyn std::error::Error + Send + Sync>;
x!(BoxedError X13);
#[cfg(feature = "toml_support")]
type X14 = toml::de::Error;
#[cfg(feature = "toml_support")]
x!(TomlDeError X14);
#[cfg(feature = "toml_support")]
type X15 = toml::ser::Error;
#[cfg(feature = "toml_support")]
x!(TomlSerError X15);
#[cfg(feature = "serde_yaml_support")]
type X16 = serde_yaml::Error;
#[cfg(feature = "serde_yaml_support")]
x!(SerdeYamlError X16);
#[cfg(feature = "reqwest_support")]
type X17 = reqwest::Error;
#[cfg(feature = "reqwest_support")]
x!(ReqwestError X17);
#[cfg(feature = "hyper_support")]
type X18 = hyper::Error;
#[cfg(feature = "hyper_support")]
x!(HyperError X18);

/*
type X = ;
x!(Error X);
*/
