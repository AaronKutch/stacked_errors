use alloc::{fmt, fmt::Debug};
use core::{fmt::Display, panic::Location};

use crate::{error::StackedError, Error, ErrorKind};

/// For implementing `Debug`, this wrapper makes strings use their `Display`
/// impl rather than `Debug` impl
pub struct DisplayStr<'a>(pub &'a str);
impl Debug for DisplayStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

/// This is a wrapper around a `Location` that shortens the `Debug` of the
/// `file` field
///
/// If this detects "/.cargo/registry/src/" in the `file` field, it truncates
/// that and all previous characters, and the following "/" group if it exists
/// (it is alternately configured to do this with "\\" on Windows). For example,
/// "/home/admin/.cargo/registry/src/index.crates.io-6f17d22bba15001f/
/// super_orchestrator-0.5.1/src/misc.rs"
/// gets truncated to "super_orchestrator-0.5.1/src/misc.rs"
pub struct DisplayShortLocation<'a>(pub &'a Location<'a>);
impl Debug for DisplayShortLocation<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = self.0.file();
        #[cfg(not(windows))]
        {
            let find = "/.cargo/registry/src/";
            if let Some(i) = s.find(find) {
                s = &s[(i + find.len())..];
                if let Some(i) = s.find('/') {
                    s = &s[(i + 1)..];
                }
            }
        }
        #[cfg(windows)]
        {
            let find = "\\.cargo\\registry\\src\\";
            if let Some(i) = s.find(find) {
                s = &s[(i + find.len())..];
                if let Some(i) = s.find('\\') {
                    s = &s[(i + 1)..];
                }
            }
        }
        f.write_fmt(format_args!(
            "Location {{ file: \"{}\", line: {}, col: {} }}",
            s,
            self.0.line(),
            self.0.column()
        ))
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // in reverse order of a typical stack, I don't want to have to scroll up to see
        // the more specific errors
        f.write_fmt(format_args!("Error {{ stack: [\n"))?;
        for (error, location) in self.stack.iter().rev() {
            if let Some(location) = location {
                let location = DisplayShortLocation(location);
                f.write_fmt(format_args!("{location:?},\n"))?;
            }
            match error {
                ErrorKind::UnitError => (),
                ErrorKind::StrError(s) => {
                    f.write_fmt(format_args!("{s}\n"))?;
                }
                ErrorKind::StringError(s) => {
                    f.write_fmt(format_args!("{s}\n"))?;
                }
                _ => {
                    f.write_fmt(format_args!("{error:?},\n"))?;
                }
            }
        }
        f.write_fmt(format_args!("] }}"))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

impl Display for StackedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}
