// TODO when https://github.com/rust-lang/rust/issues/103765 is stabilized
// we can make a large subset as no_std
//#![no_std]

extern crate alloc;
mod error;
/// This is an experimental errors crate.
/// Note: you should probably use `default-features = false` in your
/// `Cargo.toml`
mod error_kind;
mod map_add_err;
use alloc::boxed::Box;

pub use error::Error;
pub use error_kind::ErrorKind;
pub use map_add_err::MapAddError;

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
x_box!(RonError X8);
#[cfg(feature = "ctrlc_support")]
type X9 = ctrlc::Error;
#[cfg(feature = "ctrlc_support")]
x!(CtrlcError X9);
type X10 = core::num::ParseIntError;
x!(ParseIntError X10);
type X11 = core::num::ParseFloatError;
x!(ParseFloatError X11);
type X12 = core::num::TryFromIntError;
x!(TryFromIntError X12);
type X13 = Box<dyn std::error::Error + Send + Sync>;
x!(BoxedError X13);
#[cfg(feature = "toml_support")]
type X14 = toml::de::Error;
#[cfg(feature = "toml_support")]
x_box!(TomlDeError X14);
#[cfg(feature = "toml_support")]
type X15 = toml::ser::Error;
#[cfg(feature = "toml_support")]
x_box!(TomlSerError X15);
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
