/// Used as a placeholder
#[derive(thiserror::Error, Debug)]
#[error("UnitError")]
pub struct UnitError {}

/// Used to signal timeouts
#[derive(thiserror::Error, Debug)]
#[error("TimeoutError")]
pub struct TimeoutError {}

/// Used to signal to the docker container orchestrator that an error was
/// probably not the root cause
#[derive(thiserror::Error, Debug)]
#[error("ProbablyNotRootCauseError")]
pub struct ProbablyNotRootCauseError {}
