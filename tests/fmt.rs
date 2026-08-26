//! Formatting tests that do not depend on the lines they are written on, see
//! also `debug.rs`

use stacked_errors::{Error, UnitError};

/// The `file` of a [core::panic::Location] from this file
const FILE: &str = "tests/fmt.rs";

/// Replaces the ` line:column` following every location file name with
/// ` L:C`, so that the tests do not need to be updated when lines shift
fn scrub(s: &str) -> String {
    let file = convert(FILE);
    let mut res = String::new();
    let mut rest = s;
    while let Some(i) = rest.find(&file) {
        let (before, after) = rest.split_at(i + file.len());
        res.push_str(before);
        // consume everything the location could have left behind
        let end = after
            .find(|c: char| !(c == ' ' || c == ':' || c.is_ascii_digit()))
            .unwrap_or(after.len());
        res.push_str(" L:C");
        rest = &after[end..];
    }
    res.push_str(rest);
    res
}

fn convert(s: &str) -> String {
    let s = s.to_owned();
    if cfg!(windows) {
        s.replace("/", "\\")
    } else {
        s
    }
}

/// Every entry is separated by exactly one newline, and there is a leading but
/// no trailing newline
#[test]
fn separators() {
    let e = Error::from_err("root")
        .add_err("middle")
        .add_err_locationless("outer");
    assert_eq!(
        scrub(&format!("{e}")),
        convert("\n    outer\n    middle at tests/fmt.rs L:C\n    root at tests/fmt.rs L:C")
    );
}

/// A [UnitError] contributes only its location, and a locationless one
/// contributes nothing at all
#[test]
fn unit_errors() {
    // the entry that is printed last is skipped entirely, which must not leave
    // behind the newline separating it from the entry before it
    let e = Error::from_err_locationless(UnitError {}).add_err("hello");
    assert_eq!(
        scrub(&format!("{e}")),
        convert("\n    hello at tests/fmt.rs L:C")
    );

    // the same for the entry that is printed first
    let e = Error::from_err("hello").add_err_locationless(UnitError {});
    assert_eq!(
        scrub(&format!("{e}")),
        convert("\n    hello at tests/fmt.rs L:C")
    );

    // a `UnitError` with a location gets a line of its own
    let e = Error::new().add_err("hello");
    assert_eq!(
        scrub(&format!("{e}")),
        convert("\n    hello at tests/fmt.rs L:C\n  at tests/fmt.rs L:C")
    );

    // an empty stack, and a stack of nothing but skipped entries, format to
    // nothing
    assert_eq!(format!("{}", Error::empty()), "");
    let e = Error::from_err_locationless(UnitError {}).add_err_locationless(UnitError {});
    assert_eq!(format!("{e}"), "");
}

/// A location is split onto its own line when it would not fit after the
/// message
#[test]
fn location_splitting() {
    let long = "_".repeat(75);
    let e = Error::from_err(long.clone());
    assert_eq!(
        scrub(&format!("{e}")),
        convert(&format!("\n    {long}\n  at tests/fmt.rs L:C"))
    );

    // the length of the message of a previous entry must not influence this,
    // this entry is short enough that its location stays on the same line
    let e = Error::from_err("short").add_err_locationless(long.clone());
    assert_eq!(
        scrub(&format!("{e}")),
        convert(&format!("\n    {long}\n    short at tests/fmt.rs L:C"))
    );

    // and neither must it influence the line of a `UnitError`, which never has
    // a message to be split away from
    let e = Error::new().add_err_locationless(long.clone());
    assert_eq!(
        scrub(&format!("{e}")),
        convert(&format!("\n    {long}\n  at tests/fmt.rs L:C"))
    );
}

/// A chained on [Error] keeps the order of its entries, with the location of
/// the call that chained it on top
#[test]
fn chained_errors() {
    let inner = Error::from_err("inner root").add_err("inner mid");
    let e = Error::from_err("outer root").add_err(inner);
    assert_eq!(
        scrub(&format!("{e}")),
        convert(
            "\n  at tests/fmt.rs L:C\n    inner mid at tests/fmt.rs L:C\n    inner root at \
             tests/fmt.rs L:C\n    outer root at tests/fmt.rs L:C"
        )
    );
}

/// The `Debug` impl is the same except for the terminal styling
#[test]
fn styling() {
    let e = Error::from_err("hello").add_err_locationless(UnitError {});
    let debug = format!("{e:?}");
    // the styling is only unconditional without the "supports-color" feature, with
    // it the answer depends on whether stderr happens to be a terminal
    assert_eq!(debug.contains('\u{1b}'), stacked_errors::styling_enabled());
    // removing the styling should recover the `Display` output
    let mut plain = String::new();
    let mut in_escape = false;
    for c in debug.chars() {
        if in_escape {
            in_escape = c != 'm';
        } else if c == '\u{1b}' {
            in_escape = true;
        } else {
            plain.push(c);
        }
    }
    assert_eq!(plain, format!("{e}"));
}
