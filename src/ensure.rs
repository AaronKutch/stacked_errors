/// Asserts that a boolean expression is `true` at runtime, returning a
/// stackable error otherwise.
///
/// Has `return Err(...)` with a [stacked_errors::Error](crate::Error) and
/// attached location if the expression is false. An custom message can be
/// attached that is used as a [StackableErr](crate::StackableErr) argument.
///
/// ```
/// use stacked_errors::{ensure, Result, StackableErr};
///
/// fn ex(val0: bool, val1: bool) -> Result<()> {
///     ensure!(true);
///
///     ensure!(val0);
///
///     ensure!(val1, format!("val1 was {}", val1));
///
///     Ok(())
/// }
///
/// ex(true, true).unwrap();
///
/// assert_eq!(
///     format!("{}", ex(false, true).unwrap_err()),
///     r#"Error { stack: [
/// Location { file: "src/ensure.rs", line: 10, col: 5 },
/// ensure(val0) -> assertion failed
/// ] }"#
/// );
///
/// assert_eq!(
///     format!("{}", ex(true, false).unwrap_err()),
///     r#"Error { stack: [
/// Location { file: "src/ensure.rs", line: 12, col: 5 },
/// val1 was false
/// ] }"#
/// );
/// ```
#[macro_export]
macro_rules! ensure {
    ($expr:expr) => {
        if !$expr {
            return Err($crate::Error::from_kind($crate::__private::concat!(
                "ensure(",
                $crate::__private::stringify!($expr),
                ") -> assertion failed"
            )))
        }
    };
    ($expr:expr, $msg:expr) => {
        if !$expr {
            return Err($crate::Error::from_kind($msg))
        }
    };
}

/// Asserts that two expressions are equal to each other (with [PartialEq]),
/// returning a stackable error if they are equal. [Debug] is also required if
/// there is no custom message.
///
/// Has `return Err(...)` with a [stacked_errors::Error](crate::Error) and
/// attached location if the expressions are unequal. A custom message can be
/// attached that is used as an [Error::from_kind](crate::Error::from_kind)
/// argument.
///
/// ```
/// use stacked_errors::{ensure_eq, Result, StackableErr};
///
/// fn ex(val0: u8, val1: &str) -> Result<()> {
///     ensure_eq!(42, 42);
///
///     ensure_eq!(8, val0);
///
///     ensure_eq!("test", val1, format!("val1 was \"{}\"", val1));
///
///     Ok(())
/// }
///
/// ex(8, "test").unwrap();
///
/// assert_eq!(
///     format!("{}", ex(0, "test").unwrap_err()),
///     r#"Error { stack: [
/// Location { file: "src/ensure.rs", line: 10, col: 5 },
/// ensure_eq(
///  lhs: 8
///  rhs: 0
/// ) -> equality assertion failed
/// ] }"#
/// );
///
/// assert_eq!(
///     format!("{}", ex(8, "other").unwrap_err()),
///     r#"Error { stack: [
/// Location { file: "src/ensure.rs", line: 12, col: 5 },
/// val1 was "other"
/// ] }"#
/// );
/// ```
#[macro_export]
macro_rules! ensure_eq {
    ($lhs:expr, $rhs:expr) => {
        // use the strategy that the core library does for `assert_eq`
        match (&$lhs, &$rhs) {
            (lhs, rhs) => {
                // use the double inversion because we are relying on `PartialEq`
                if !(*lhs == *rhs) {
                    return Err($crate::Error::from_err($crate::__private::format!(
                        "ensure_eq(\n lhs: {:?}\n rhs: {:?}\n) -> equality assertion failed",
                        lhs,
                        rhs,
                    )))
                }
            }
        }
    };
    ($lhs:expr, $rhs:expr, $msg:expr) => {
        match (&$lhs, &$rhs) {
            (lhs, rhs) => {
                if !(*lhs == *rhs) {
                    return Err($crate::Error::from_err($msg))
                }
            }
        }
    };
}

/// Asserts that two expressions are not equal to each other (with [PartialEq]),
/// returning a stackable error if they are equal. [Debug] is also required if
/// there is no custom message.
///
/// Has `return Err(...)` with a [stacked_errors::Error](crate::Error) and
/// attached location if the expressions are equal. A custom message can be
/// attached that is used as an [Error::from_kind](crate::Error::from_kind)
/// argument.
///
/// ```
/// use stacked_errors::{ensure_ne, Result, StackableErr};
///
/// fn ex(val0: u8, val1: &str) -> Result<()> {
///     ensure_ne!(42, 8);
///
///     ensure_ne!(8, val0);
///
///     ensure_ne!("test", val1, format!("val1 was \"{}\"", val1));
///
///     Ok(())
/// }
///
/// ex(0, "other").unwrap();
///
/// assert_eq!(
///     format!("{}", ex(8, "other").unwrap_err()),
///     r#"Error { stack: [
/// Location { file: "src/ensure.rs", line: 10, col: 5 },
/// ensure_ne(
///  lhs: 8
///  rhs: 8
/// ) -> inequality assertion failed
/// ] }"#
/// );
///
/// assert_eq!(
///     format!("{}", ex(0, "test").unwrap_err()),
///     r#"Error { stack: [
/// Location { file: "src/ensure.rs", line: 12, col: 5 },
/// val1 was "test"
/// ] }"#
/// );
/// ```
#[macro_export]
macro_rules! ensure_ne {
    ($lhs:expr, $rhs:expr) => {
        // use the strategy that the core library does for `assert_ne`
        match (&$lhs, &$rhs) {
            (lhs, rhs) => {
                // use the double inversion because we are relying on `PartialEq`
                if !(*lhs != *rhs) {
                    return Err($crate::Error::from_err($crate::__private::format!(
                        "ensure_ne(\n lhs: {:?}\n rhs: {:?}\n) -> inequality assertion failed",
                        lhs,
                        rhs,
                    )))
                }
            }
        }
    };
    ($lhs:expr, $rhs:expr, $msg:expr) => {
        match (&$lhs, &$rhs) {
            (lhs, rhs) => {
                if !(*lhs != *rhs) {
                    return Err($crate::Error::from_err($msg))
                }
            }
        }
    };
}
