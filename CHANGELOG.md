# Changelog

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
