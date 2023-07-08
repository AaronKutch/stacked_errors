use alloc::boxed::Box;

/// In the future we plan on having almost every kind of error here under
/// different feature gates. Please file an issue if you would like to include
/// something.
///
/// The intention with `TimeoutError` is that if it is in the error stack, a
/// timeout occured. When other timeout structs are used, this should be added
/// on.
///
/// `ProbablyNotRootCauseError` is used to signal that the error is probably not
/// the "root cause". This is used by the `super_orchestrator` crate to reduce
/// noise when one error triggers a cascade of errors in a network.
///
/// Some things besides `BoxedError` are boxed to reduce the overall size of
/// `ErrorKind`
#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    // used for special cases where we need something
    #[error("UnitError")]
    UnitError,
    #[error("TimeoutError")]
    TimeoutError,
    #[error("ProbablyNotRootCauseError")]
    ProbablyNotRootCauseError,
    #[error("StrError")]
    StrError(&'static str),
    #[error("StringError")]
    StringError(alloc::string::String),
    #[error("BoxedError")]
    BoxedError(Box<dyn std::error::Error + Send + Sync>),
    #[error("TryFromIntError")]
    TryFromIntError(core::num::TryFromIntError),
    #[error("StdIoError")]
    StdIoError(std::io::Error),
    #[error("FromUtf8Error")]
    FromUtf8Error(alloc::string::FromUtf8Error),
    // this is more obscure but I think we should keep it because it deals with bad strings, and
    // we don't want that in our generic string errors.
    #[error("FromUtf16Error")]
    FromUtf16Error(alloc::string::FromUtf16Error),
    #[error("ParseIntError")]
    ParseIntError(core::num::ParseIntError),
    #[error("ParseFloatError")]
    ParseFloatError(core::num::ParseFloatError),
    #[cfg(feature = "tokio_rt_support")]
    #[error("TokioJoinError")]
    TokioJoinError(tokio::task::JoinError),
    // Borsh effecively uses `std::io::Error`
    #[cfg(feature = "ron_support")]
    #[error("RonError")]
    RonError(Box<ron::error::Error>), // box
    #[cfg(feature = "serde_json_support")]
    #[error("SerdeJsonError")]
    SerdeJsonError(serde_json::Error),
    #[cfg(feature = "ctrlc_support")]
    #[error("CtrlcError")]
    CtrlcError(ctrlc::Error),
    #[cfg(feature = "toml_support")]
    #[error("TomlDeError")]
    TomlDeError(Box<toml::de::Error>), // box
    #[cfg(feature = "toml_support")]
    #[error("TomlSerError")]
    TomlSerError(Box<toml::ser::Error>), // box
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

impl Default for ErrorKind {
    fn default() -> Self {
        Self::UnitError
    }
}
