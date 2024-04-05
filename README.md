# Stacked Errors

 A crate for high level error propogation with programmed backtraces.

 In Rust development, major crates will often have their own error enums that
 work well in their own specialized domain, but when orchestrating many
 domains together we run into issues. `map_err` is very annoying to work
 with. In `async` call stacks we run into an especially annoying problem
 where the same kind of error can be returned from multiple places, and we
 are sometimes forced into `println` debugging to find out where it is actually
 from. This crate introduces the `StackableErr` trait and a "stackable" error
 type that allows for both software-defined error backtraces and easily
 converting errors into the stackable error type.

 Some partial examples of what using the crate looks like:

 ```rust
 f.map_err(|e| Error::from_box(Box::new(e)))?;
 // replace the above with
 f.stack()?; // uses `#[track_caller]` when an error is being propagated
 ```
 ```rust
 let dir = self
     .path
     .parent()
     .stack_err(|| "FileOptions::preacquire() -> empty path")?
     .to_str()
     .stack_err(|| "bad OsStr conversion")?;
 ```
 ```rust
 // if needing to push another arbitrary error onto the stack
 f.stack_err(|| ErrorKind::from_err(arbitrary))?;
 ```
 ```rust
 option.take()
     .stack_err(|| "`Struct` has already been taken")?
     .wait_with_output()
     .await
     .stack_err(|| {
         format!("{self:?}.xyz() -> failed when waiting")
     })?;
 ```
 ```rust
 // strings and some std errors can be created like this,
 return Err(Error::from(format!(
     "failure of {x:?} to complete"
 )))
 // otherwise use this (also note that `Error::from*` includes
 // `#[track_caller]` location, no need to add on a `stack` call)
 return Err(Error::from_err(needs_boxing))
 ```
 ```rust
 // when the error type is already `stacked_errors::Error` you can do this if it is
 // preferable over `map`
 return match ... {
     Ok(ok) => {
         ...
     }
     Err(e) => Err(e.add_kind(format!("myfunction(.., host: {host})"))),
 }
 ```

 ```rust
 use stacked_errors::{Error, Result, StackableErr};

 // Note that `Error` uses `ThinVec` internally, which means that it often
 // takes up only the stack space of a `usize` or the size of the `T` plus
 // a byte.
 fn innermost(s: &str) -> Result<u8> {
     if s == "return error" {
         // When creating the initial `Result<_, Error>` from something that
         // is directly representable in a `ErrorKind` (i.e. not needing
         // `BoxedErr`), use this `Err(Error::from(...))` format. This
         // format is cumbersome relative to the other features of this
         // crate, but it is the best solution because of technicalities
         // related to trait collisions at the design level, `Result` type
         // inference with the return type, wanting to keep the directly
         // representable strings outside of a box for performance, and
         // because of the `Display` impl which special cases them.

         return Err(Error::from("bottom level `StrErr`"))
     }
     if s == "parse invalid" {
         // However, this is the common case where we have some external
         // crate function that returns a `Result<..., E: Error>`. We
         // usually call `StackableErr::stack_err` if we want to attach
         // some message to it right away (it is called with a closure
         // so that it doesn't have impact on the `Ok` cases). Otherwise, we
         // just call `StackableErr::stack` so that just the location is
         // pushed on the stack. We can then use `?` directly.

         let _ = ron::from_str("invalid").stack_err(|| format!("parsing error with \"{s}\""))?;
     }
     Ok(42)
 }

 fn inner(s: &str) -> Result<u16> {
     // Chainable with other combinators. Use `stack_err` with a message for
     // propogating up the stack when the error is something that should
     // have some mid layer information attached for it for quick diagnosis
     // by the user. Otherwise use just `stack` which will also do error
     // conversion if necessary, avoiding needing to wrangle with `map_err`.

     let x = innermost(s)
         .map(|x| u16::from(x))
         .stack_err(|| format!("error from innermost(\"{s}\")"))?;
     Ok(x)
 }

 fn outer(s: &str) -> Result<u64> {
     // ...

     let x = inner(s).stack()?;

     // ...
     Ok(u64::from(x))
 }

 let res = format!("{:?}", outer("valid"));
 assert_eq!(res, "Ok(42)");

 // The line numbers are slightly off because this is a doc test.
 // In order from outer to the innermost call, it lists the location of the
 // `stack` call from `outer`, the location of `stack_err` from `inner`,
 // the associated error message, the location of either the `Error::from`
 // or `stack_err` from `innermost`, and finally the root error message.

 let res = format!("{:?}", outer("return error"));
 assert_eq!(
     res,
     r#"Err(Error { stack: [
 Location { file: "src/lib.rs", line: 54, col: 22 },
 Location { file: "src/lib.rs", line: 47, col: 10 },
 error from innermost("return error")
 Location { file: "src/lib.rs", line: 22, col: 20 },
 bottom level `StrErr`
 ] })"#
 );

 let res = format!("{:?}", outer("parse invalid"));
 assert_eq!(
     res,
     r#"Err(Error { stack: [
 Location { file: "src/lib.rs", line: 54, col: 22 },
 Location { file: "src/lib.rs", line: 47, col: 10 },
 error from innermost("parse invalid")
 parsing error with "parse invalid"
 Location { file: "src/lib.rs", line: 33, col: 42 },
 BoxedError(SpannedError { code: ExpectedUnit, position: Position { line: 1, col: 1 } }),
 ] })"#
 );
 ```

 Also remember that `.stack_err(|| ())` is equivalent to `.stack()`

 ```rust
 // in commonly used functions you may want `_locationless` to avoid adding
 // on unnecessary information if the location is already being added on
 return Err(e.add_err_locationless(ErrorKind::TimeoutError)).stack_err(|| {
     format!(
         "wait_for_ok(num_retries: {num_retries}, delay: {delay:?}) timeout, \
          last error stack was:"
     )
 })
 ```
