/// Equivalent to `return Err(Error::from_err(format_args!(...)))` if a string
/// literal, `return Err(Error::from_err(expr))` if a single expression, or
/// `return Err(Error::from_err(format!(...)))` otherwise.
#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return Err($crate::__private::format_err($crate::__private::format_args!($msg)));
    };
    ($err:expr $(,)?) => {
        return Err($crate::Error::from_err($err));
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err($crate::Error::from_err($crate::__private::format!($fmt, $($arg)*)));
    };
}

/// The `bail` macro but with `_locationless` variations
#[macro_export]
macro_rules! bail_locationless {
    ($msg:literal $(,)?) => {
        return Err($crate::__private::format_err_locationless(
            $crate::__private::format_args!($msg)
        ));
    };
    ($err:expr $(,)?) => {
        return Err($crate::Error::from_err_locationless($err));
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err($crate::Error::from_err_locationless(
            $crate::__private::format!($fmt, $($arg)*)
        ));
    };
}

/// For ease of translating from the `eyre` crate, but also the recommended
/// macro to use if you use this kind of macro
#[macro_export]
macro_rules! eyre {
    ($msg:literal $(,)?) => {
        $crate::__private::format_err($crate::__private::format_args!($msg));
    };
    ($err:expr $(,)?) => {
        $crate::Error::from_err($err);
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::Error::from_err($crate::__private::format!($fmt, $($arg)*));
    };
}

/// For ease of translating from the `anyhow` crate
#[macro_export]
macro_rules! anyhow {
    ($msg:literal $(,)?) => {
        $crate::__private::format_err($crate::__private::format_args!($msg));
    };
    ($err:expr $(,)?) => {
        $crate::Error::from_err($err);
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::Error::from_err($crate::__private::format!($fmt, $($arg)*));
    };
}

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
///     r#"ensure(val0) -> assertion failed at src/macros.rs 10:5"#
/// );
///
/// assert_eq!(
///     format!("{}", ex(true, false).unwrap_err()),
///     r#"val1 was false at src/macros.rs 12:5"#
/// );
/// ```
#[macro_export]
macro_rules! ensure {
    ($expr:expr) => {
        if !$expr {
            return Err($crate::Error::from_err($crate::__private::concat!(
                "ensure(",
                $crate::__private::stringify!($expr),
                ") -> assertion failed"
            )))
        }
    };
    ($expr:expr, $msg:expr) => {
        if !$expr {
            return Err($crate::Error::from_err($msg))
        }
    };
}

/// Asserts that two expressions are equal to each other (with [PartialEq]),
/// returning a stackable error if they are equal. [Debug] is also required if
/// there is no custom message.
///
/// Has `return Err(...)` with a [stacked_errors::Error](crate::Error) and
/// attached location if the expressions are unequal. A custom message can be
/// attached that is used as an [Error::from_err](crate::Error::from_err)
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
///     r#"ensure_eq(
///  lhs: 8
///  rhs: 0
/// ) -> equality assertion failed at src/macros.rs 10:5"#
/// );
///
/// assert_eq!(
///     format!("{}", ex(8, "other").unwrap_err()),
///     r#"val1 was "other" at src/macros.rs 12:5"#
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
/// attached that is used as an [Error::from_err](crate::Error::from_err)
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
///     r#"ensure_ne(
///  lhs: 8
///  rhs: 8
/// ) -> inequality assertion failed at src/macros.rs 10:5"#
/// );
///
/// assert_eq!(
///     format!("{}", ex(0, "test").unwrap_err()),
///     r#"val1 was "test" at src/macros.rs 12:5"#
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

/// Applies `get` and `stack_err_with(...)?` in a chain, this is compatible with
/// many things.
///
/// ```
/// use serde_json::Value;
/// use stacked_errors::{ensure, stacked_get, Result, StackableErr};
///
/// let s = r#"{
///     "Id": "id example",
///     "Created": 2023,
///     "Args": [
///         "--entry-name",
///         "--uuid"
///     ],
///     "State": {
///         "Status": "running",
///         "Running": true
///     }
/// }"#;
///
/// fn ex0(s: &str) -> Result<()> {
///     let value: Value = serde_json::from_str(s).stack()?;
///
///     // the normal `Index`ing of `Values` panics, this
///     // returns a formatted error
///     ensure!(stacked_get!(value["Id"]) == "id example");
///     ensure!(stacked_get!(value["Created"]) == 2023);
///     ensure!(stacked_get!(value["Args"][1]) == "--uuid");
///     ensure!(stacked_get!(value["State"]["Status"]) == "running");
///     ensure!(stacked_get!(value["State"]["Running"]) == true);
///
///     Ok(())
/// }
///
/// ex0(s).unwrap();
///
/// fn ex1(s: &str) -> Result<()> {
///     let value: Value = serde_json::from_str(s).stack()?;
///
///     let _ = stacked_get!(value["State"]["nonexistent"]);
///
///     Ok(())
/// }
///
/// assert!(ex1(s).is_err());
/// ```
#[macro_export]
macro_rules! stacked_get {
    ($value:ident [$inx0:expr] $([$inx1:expr])*) => {{
        // this is unrolled once to avoid a let binding
        // and allow multiple kinds of borrowing
        #[allow(unused)]
        let mut tmp = $crate::StackableErr::stack_err_with($value.get($inx0),
            || {$crate::__private::format!(
                "stacked_get({} ... [{:?}] ...) -> indexing failed",
                $crate::__private::stringify!($value),
                $inx0
            )}
        )?;
        $(
            tmp = $crate::StackableErr::stack_err_with(tmp.get($inx1),
                || $crate::__private::format!(
                    "stacked_get({} ... [{:?}] ...) -> indexing failed",
                    $crate::__private::stringify!($value),
                    $inx1
                )
            )?;
        )*
        tmp
    }};
}

/// Applies `get_mut` and `stack_err_with(...)?` in a chain, this is compatible
/// with many things.
///
/// ```
/// use serde_json::Value;
/// use stacked_errors::{ensure, stacked_get, stacked_get_mut, Result, StackableErr};
///
/// let s = r#"{
///     "Id": "id example",
///     "Created": 2023,
///     "Args": [
///         "--entry-name",
///         "--uuid"
///     ],
///     "State": {
///         "Status": "running",
///         "Running": true
///     }
/// }"#;
///
/// fn ex0(s: &str) -> Result<()> {
///     let mut value: Value = serde_json::from_str(s).stack()?;
///
///     *stacked_get_mut!(value["Id"]) = "other".into();
///     *stacked_get_mut!(value["Created"]) = 0.into();
///     *stacked_get_mut!(value["Args"][1]) = "--other".into();
///     *stacked_get_mut!(value["State"]["Status"]) = "stopped".into();
///     *stacked_get_mut!(value["State"]["Running"]) = false.into();
///
///     // when creating a new field
///     stacked_get_mut!(value["State"])["OtherField"] = "hello".into();
///
///     ensure!(stacked_get!(value["Id"]) == "other");
///     ensure!(stacked_get!(value["Created"]) == 0);
///     ensure!(stacked_get!(value["Args"][1]) == "--other");
///     ensure!(stacked_get!(value["State"]["Status"]) == "stopped");
///     ensure!(stacked_get!(value["State"]["Running"]) == false);
///     ensure!(stacked_get!(value["State"]["OtherField"]) == "hello");
///
///     Ok(())
/// }
///
/// ex0(s).unwrap();
///
/// fn ex1(s: &str) -> Result<()> {
///     let mut value: Value = serde_json::from_str(s).stack()?;
///
///     let _ = stacked_get_mut!(value["State"]["nonexistent"]);
///
///     Ok(())
/// }
///
/// assert!(ex1(s).is_err());
/// ```
#[macro_export]
macro_rules! stacked_get_mut {
    ($value:ident [$inx0:expr] $([$inx1:expr])*) => {{
        #[allow(unused)]
        let mut tmp = $crate::StackableErr::stack_err_with($value.get_mut($inx0),
            || $crate::__private::format!(
                "stacked_get_mut({} ... [{:?}] ...) -> indexing failed",
                $crate::__private::stringify!($value),
                $inx0
            )
        )?;
        $(
            tmp = $crate::StackableErr::stack_err_with(tmp.get_mut($inx1),
                || $crate::__private::format!(
                    "stacked_get_mut({} ... [{:?}] ...) -> indexing failed",
                    $crate::__private::stringify!($value),
                    $inx1
                )
            )?;
        )*
        tmp
    }};
}
