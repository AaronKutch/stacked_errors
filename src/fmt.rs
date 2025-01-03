use alloc::{fmt, fmt::Debug};
use core::{fmt::Display, panic::Location};
use std::fmt::Write;

use owo_colors::{OwoColorize, Style};

use crate::{error::StackedErrorDowncast, Error, UnitError};

/// For implementing `Debug`, this wrapper makes strings use their `Display`
/// impl rather than `Debug` impl
pub struct DisplayStr<'a>(pub &'a str);
impl Debug for DisplayStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

fn shorten_location(mut s: &str) -> &str {
    #[cfg(not(windows))]
    {
        let find = "/.cargo/registry/src/";
        if let Some(i) = s.find(find) {
            s = &s[(i + find.len())..];
            if let Some(i) = s.find('/') {
                s = &s[(i + 1)..];
            }
        }
        s
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
        s
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
    /// Has terminal styling in alternate mode
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            let underline = Style::new().underline();
            let bold = Style::new().bold();

            f.write_fmt(format_args!(
                "{} {}:{}",
                shorten_location(self.0.file()).style(underline),
                self.0.line().style(bold),
                self.0.column().style(bold),
            ))
        } else {
            f.write_fmt(format_args!(
                "{} {}:{}",
                shorten_location(self.0.file()),
                self.0.line(),
                self.0.column(),
            ))
        }
    }
}
impl Display for DisplayShortLocation<'_> {
    /// Same as `Debug`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl Debug for Error {
    /// Has terminal styling
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // in reverse order of a typical stack, I don't want to have to scroll up to see
        // the more specific errors
        let mut s = String::new();
        for (i, e) in self.iter().enumerate().rev() {
            s.clear();
            let is_unit_err = e.downcast_ref::<UnitError>().is_some();
            let is_last = i == 0;
            if is_unit_err {
                if e.get_location().is_none() {
                    continue;
                }
            } else {
                write!(s, "{}", e.get_err())?;
            }
            if let Some(l) = e.get_location() {
                // if the current length plus the location length (the +8 is from the space,
                // colon, and 4 digits for line and 2 for column) is more than 80 then split up
                if (s.len() + l.file().len() + 8) > 80 {
                    // split up
                    write!(s, "\n")?;
                } else if !is_unit_err {
                    write!(s, " ")?;
                }
                let underline = Style::new().underline();
                let bold = Style::new().bold();

                write!(
                    s,
                    "at {} {}:{}",
                    shorten_location(l.file()).style(underline),
                    l.line().style(bold),
                    l.column().style(bold),
                )?;
            }
            if !is_last {
                write!(s, ",\n")?;
            }
            f.write_fmt(format_args!("{s}"))?
        }
        Ok(())
    }
}

impl Display for Error {
    /// Same as `Debug` but without terminal styling
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // in reverse order of a typical stack, I don't want to have to scroll up to see
        // the more specific errors
        let mut s = String::new();
        for (i, e) in self.iter().enumerate().rev() {
            s.clear();
            let is_unit_err = e.downcast_ref::<UnitError>().is_some();
            let is_last = i == 0;
            if is_unit_err {
                if e.get_location().is_none() {
                    continue;
                }
            } else {
                write!(s, "{}", e.get_err())?;
            }
            if let Some(l) = e.get_location() {
                // if the current length plus the location length (the +8 is from the space,
                // colon, and 4 digits for line and 2 for column) is more than 80 then split up
                if (s.len() + l.file().len() + 8) > 80 {
                    // split up
                    write!(s, "\n")?;
                } else if !is_unit_err {
                    write!(s, " ")?;
                }

                write!(
                    s,
                    "at {} {}:{}",
                    shorten_location(l.file()),
                    l.line(),
                    l.column(),
                )?;
            }
            if !is_last {
                write!(s, ",\n")?;
            }
            f.write_fmt(format_args!("{s}"))?
        }
        Ok(())
    }
}
