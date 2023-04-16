use std::fmt;

// Error returned by `Retry`
#[derive(Debug, PartialEq, Eq)]
pub struct Exhausted(());

impl Exhausted {
    pub(crate) fn new() -> Self {
        Exhausted(())
    }
}

impl fmt::Display for Exhausted {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        "chances have been run out".fmt(fmt)
    }
}

impl std::error::Error for Exhausted {}

impl From<Exhausted> for std::io::Error {
    fn from(_err: Exhausted) -> std::io::Error {
        std::io::ErrorKind::TimedOut.into()
    }
}
