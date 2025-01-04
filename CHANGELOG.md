# Changelog

## [0.7.0] - 2025-01-03
### Changes
- All `E: Display + Send + Sync + 'static` can be used with `StackableErr` now
- Removed `ErrorKind`, all types go in a `SmallBox` internally
- `stack_err*` functions that take closures were renamed to `stack_err_with*`. The new `stack_err` now takes an expression directly.
- More swap-in compatibility with `eyre` and `anyhow`
- The error type now implements `core::error::Error`
- The error type deliberately does not implement any `From` impls so that all initial `?`s need to be covered
- Introduced `bail!` macro for easily creating and returning errors
- Discovered an incredibly concise yet more distinctive format. `Debug` mode uses terminal styling.
- The crate is now unconditionally `no_std`
- Moved `stacked_get` and `stacked_get_mut` to this crate
- many small improvements

## [0.6.0] - 2024-11-21
### Changes
- Updates the MSRV feature to 1.81, uses the `Error` in `core` feature to make this crate no-std.
- There is now an "std" feature enabled by default for enabling `std::io::Error` impls
- Renamed `from_err*` functions to `box_from` to emphasize that a boxing allocation is performed

### Crate
- Updated to `thiserror` 2.0

## [0.5.2] - 2024-04-18
### Fixes
- Fixed several minor issues with the `ensure*` macros

## [0.5.1] - 2024-04-04
### Fixes
- `DisplayShortLocation` now works as intended on Windows

## [0.5.0] - 2023-11-07
### Changes
- Added `#[non_exhaustive]` to `ErrorKind`

### Additions
- Added the macros `ensure`, `ensure_eq`, `ensure_ne`
- Added `Cow<'static, str>` to the error kinds

## [0.4.0] - 2023-08-01
### Changes
- General overhaul of the crate. All non-std errors should be boxed now. The `MapAddError` trait has
  been reworked into the `StackableErr` trait with better ergonomics. See the crate documentation
- The `Debug` impl of `Error` now uses `DisplayShortLocation` on its locations

### Additions
- Added `DisplayStr`, moved here from the `super_orchestrator` crate
- Added `DisplayShortLocation`

## [0.3.0] - 07-09-2023
### Changes
- Added  `Send + Sync` bounds to the `BoxedError` variant so that the whole `Error` struct is now `Send + Sync`
- Used `ThinVec` to reduce the size of `Result<(), Error>` to be the same as `usize`
- Refactored the public internals of `Error`
- Used strategic boxing of a few outliers to reduce size of `ErrorKind`
- Changed `Debug` impls

### Additions
- Added `ProbablyNotRootCauseError`
- Added `Default` impls and `Error::new` function and `Error::empty` function

## [0.2.0] - 06-06-2023
### Additions
- `hyper_support`
- `reqwest_support`

## [0.1.0] - 29-05-2023
### Additions
- Initial release with `Error` and `MapAddError`
