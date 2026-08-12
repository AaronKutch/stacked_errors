use alloc::{fmt, fmt::Debug, string::String};
use core::fmt::{Display, Write};

use owo_colors::{CssColors, OwoColorize, Style};

use crate::{error::StackedErrorDowncast, Error, UnitError};

/// For implementing `Debug`, this wrapper makes strings use their `Display`
/// impl rather than `Debug` impl
pub struct DisplayStr<'a>(pub &'a str);
impl Debug for DisplayStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

/// Intended for shortening the file field of `Location`s.
///
/// If this finds "/.cargo/registry/src/", it truncates that and all previous
/// characters, and the following "/" group if it exists (it is alternately
/// configured to do this with "\\" on Windows). For example, "/home/admin/.
/// cargo/registry/src/index.crates.io-6f17d22bba15001f/ super_orchestrator-0.5.
/// 1/src/misc.rs" gets truncated to "super_orchestrator-0.5.1/src/misc.rs"
pub fn shorten_location(mut s: &str) -> &str {
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

/// Whether the [Debug] impl of [Error] applies terminal styling. Returns false
/// only if "supports-color" is enabled and the [supports_color] crate does not
/// detect a terminal that wants styling. Note that the [Display] impl is never
/// styled.
#[must_use]
pub fn styling_enabled() -> bool {
    #[cfg(feature = "supports-color")]
    {
        supports_color::on_cached(supports_color::Stream::Stderr).is_some()
    }
    #[cfg(not(feature = "supports-color"))]
    {
        true
    }
}

fn common_format(this: &Error, style: bool, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // in reverse order of a typical stack, I don't want to have to scroll up to see
    // the more specific errors

    // the message of the current entry needs to be rendered ahead of being written,
    // both to scan it for preexisting styling and to know its length
    let mut msg = String::new();
    for e in this.iter().rev() {
        let is_unit_err = e.downcast_ref::<UnitError>().is_some();
        let location = e.get_location();
        if is_unit_err && location.is_none() {
            continue;
        }
        // every entry is prefixed rather than suffixed by the newline, both because the
        // leading newline interacts better with `Error: ` etc since this is going to be
        // a list anyways (some other libraries do this as well), and because entries
        // can be skipped which would otherwise make a trailing newline possible
        writeln!(f)?;
        msg.clear();
        if !is_unit_err {
            write!(msg, "{}", e.get_err())?;
            // if there are vt100 styling characters already in the output, do not apply
            // styling
            if (!style) || msg.contains('\u{1b}') {
                write!(f, "    {}", msg)?;
            } else {
                let color = Style::new().color(CssColors::IndianRed);
                write!(f, "    {}", msg.style(color))?;
            }
        }
        if let Some(l) = location {
            if is_unit_err {
                // there is no message on this line to split away from
                write!(f, "  at ")?;
            } else if (msg.len() + l.file().len() + 8) > 80 {
                // if the message length plus the location length (the +8 is from the space,
                // colon, and 4 digits for line and 2 for column) is more than 80 then split
                // up
                write!(f, "\n  at ")?;
            } else {
                write!(f, " at ")?;
            }
            let dimmed = Style::new().dimmed();
            let bold = Style::new().bold();

            // TODO the `format_args` are repeated rather than bound to a variable, binding
            // one is only allowed when we bump MSRV
            if style {
                write!(
                    f,
                    "{} {}",
                    shorten_location(l.file()).style(dimmed),
                    format_args!("{}:{}", l.line(), l.column()).style(bold)
                )?;
            } else {
                write!(
                    f,
                    "{} {}",
                    shorten_location(l.file()),
                    format_args!("{}:{}", l.line(), l.column())
                )?;
            }
        }
    }
    Ok(())
}

impl Debug for Error {
    /// Has terminal styling if [styling_enabled]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        common_format(self, styling_enabled(), f)
    }
}

impl Display for Error {
    /// Same as `Debug` but always without terminal styling
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        common_format(self, false, f)
    }
}
