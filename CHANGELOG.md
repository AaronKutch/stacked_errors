# Changelog

## [0.6.0] - 2024-11-21
### Changes
- Use the `Error` in `core` feature to make this crate no-std.
- There is now an "std" feature enabled by default (with the original MSRV of 1.69).
  When disabled for no-std mode, Rust 1.81 is required.
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
