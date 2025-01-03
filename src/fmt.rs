use alloc::{fmt, fmt::Debug};
use core::fmt::Display;
use std::fmt::Write;

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

fn common_format(this: &Error, style: bool, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // in reverse order of a typical stack, I don't want to have to scroll up to see
    // the more specific errors
    let mut s = String::new();
    let mut tmp = String::new();
    let mut first = true;
    for (i, e) in this.iter().enumerate().rev() {
        s.clear();
        if first {
            // this we do to better interact with `Error: ` etc since this is going to be a
            // list anyways, some other libraries do this as well
            writeln!(s)?;
        }
        let is_unit_err = e.downcast_ref::<UnitError>().is_some();
        let is_last = i == 0;
        if is_unit_err {
            if e.get_location().is_none() {
                continue;
            }
        } else {
            // TODO can we get rid of the allocated temporaries?
            tmp.clear();
            write!(tmp, "{}", e.get_err())?;
            // if there are vt100 styling characters already in the output, do not apply
            // styling
            if (!style) || tmp.contains('\u{1b}') {
                write!(s, "    {}", tmp)?;
            } else {
                let color = Style::new().color(CssColors::IndianRed);
                write!(s, "    {}", tmp.style(color))?;
            }
        }
        if let Some(l) = e.get_location() {
            // if the current length plus the location length (the +8 is from the space,
            // colon, and 4 digits for line and 2 for column) is more than 80 then split up
            if (tmp.len() + l.file().len() + 8) > 80 {
                // split up
                write!(s, "\n  at ")?;
            } else if !is_unit_err {
                write!(s, " at ")?;
            } else {
                write!(s, "  at ")?;
            }
            let dimmed = Style::new().dimmed();
            let bold = Style::new().bold();

            tmp.clear();
            write!(tmp, "{}:{}", l.line(), l.column())?;

            if style {
                write!(
                    s,
                    "{} {}",
                    shorten_location(l.file()).style(dimmed),
                    tmp.style(bold)
                )?;
            } else {
                write!(s, "{} {}", shorten_location(l.file()), tmp)?;
            }
        }
        if !is_last {
            writeln!(s)?;
        }
        f.write_fmt(format_args!("{s}"))?;
        first = false;
    }
    Ok(())
}

impl Debug for Error {
    /// Has terminal styling
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        common_format(self, true, f)
    }
}

impl Display for Error {
    /// Same as `Debug` but without terminal styling
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        common_format(self, false, f)
    }
}
