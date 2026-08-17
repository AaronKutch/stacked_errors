//! Checks that the macro expansions resolve everything through `$crate` and do
//! not pick up items from the scope of the caller

// the decoys below being unused is part of what is being checked, if an
// expansion did resolve to one of them the crate would fail to compile
#![allow(unused_macros)]

use stacked_errors::{
    anyhow, bail, bail_locationless, ensure, ensure_eq, ensure_ne, eyre, Result as StackedResult,
    StackedErrorDowncast,
};

// shadow everything that the expansions could otherwise resolve in this scope

#[allow(dead_code)]
enum Shadow {
    Err(u8),
    Ok,
}
use Shadow::{Err, Ok};

#[allow(dead_code)]
struct Error;

macro_rules! format {
    ($($tt:tt)*) => {
        compile_error!("the `format` of the caller was used")
    };
}

macro_rules! format_args {
    ($($tt:tt)*) => {
        compile_error!("the `format_args` of the caller was used")
    };
}

macro_rules! concat {
    ($($tt:tt)*) => {
        compile_error!("the `concat` of the caller was used")
    };
}

macro_rules! stringify {
    ($($tt:tt)*) => {
        "the `stringify` of the caller was used"
    };
}

/// The messages are checked as well, so that a shadowed `stringify` would be
/// caught even though it expands to something valid
#[test]
fn shadowed_scope() {
    fn bail_empty() -> StackedResult<()> {
        bail!()
    }
    fn bail_literal() -> StackedResult<()> {
        bail!("literal")
    }
    fn bail_expr() -> StackedResult<()> {
        bail!(String::from("expr"))
    }
    fn bail_fmt() -> StackedResult<()> {
        bail!("fmt {}", 5)
    }
    fn bail_locationless_() -> StackedResult<()> {
        bail_locationless!("locationless")
    }
    fn ensure_() -> StackedResult<()> {
        ensure!(1 == 2);
        core::result::Result::Ok(())
    }
    fn ensure_msg() -> StackedResult<()> {
        ensure!(1 == 2, "ensure msg");
        core::result::Result::Ok(())
    }
    fn ensure_eq_() -> StackedResult<()> {
        ensure_eq!(1, 2);
        core::result::Result::Ok(())
    }
    fn ensure_ne_() -> StackedResult<()> {
        ensure_ne!(2, 2);
        core::result::Result::Ok(())
    }

    fn root_msg(res: StackedResult<()>) -> String {
        core::format_args!("{}", res.unwrap_err().iter().next().unwrap().get_err()).to_string()
    }

    assert_eq!(root_msg(bail_empty()), "explicit bail");
    assert_eq!(root_msg(bail_literal()), "literal");
    assert_eq!(root_msg(bail_expr()), "expr");
    assert_eq!(root_msg(bail_fmt()), "fmt 5");
    assert_eq!(root_msg(bail_locationless_()), "locationless");
    assert_eq!(root_msg(ensure_()), "ensure(1 == 2) -> assertion failed");
    assert_eq!(root_msg(ensure_msg()), "ensure msg");
    assert_eq!(
        root_msg(ensure_eq_()),
        "ensure_eq(\n lhs: 1\n rhs: 2\n) -> equality assertion failed"
    );
    assert_eq!(
        root_msg(ensure_ne_()),
        "ensure_ne(\n lhs: 2\n rhs: 2\n) -> inequality assertion failed"
    );

    // these are expressions rather than `return`s, but they use the same paths
    assert_eq!(
        root_msg(core::result::Result::Err(eyre!("eyre {}", 0))),
        "eyre 0"
    );
    assert_eq!(
        root_msg(core::result::Result::Err(anyhow!("anyhow"))),
        "anyhow"
    );

    // make sure the shadows are actually in effect
    let shadowed = Err(0);
    assert!(matches!(shadowed, Shadow::Err(0)));
    let shadowed = Ok;
    assert!(matches!(shadowed, Shadow::Ok));
    assert_eq!(stringify!(x), "the `stringify` of the caller was used");
}
