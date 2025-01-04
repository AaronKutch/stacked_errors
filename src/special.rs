/// Used internally when an error needs to be pushed but only the location is
/// important
#[derive(thiserror::Error, Debug)]
#[error("UnitError")]
pub struct UnitError {}

/// Used to signal timeouts
#[derive(thiserror::Error, Debug)]
#[error("TimeoutError")]
pub struct TimeoutError {}

/// Used to signal to crates like `super_orchestrator` that an error was
/// probably not the root cause
#[derive(thiserror::Error, Debug)]
#[error("ProbablyNotRootCauseError")]
pub struct ProbablyNotRootCauseError {}
