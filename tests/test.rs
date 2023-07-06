use stacked_errors::Error;

trait VerifyCapable: Send + Sync {}
impl VerifyCapable for Error {}

#[test]
fn tests() {}
