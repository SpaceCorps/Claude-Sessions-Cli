//! Exit codes double as the machine-readable `code:` field in the error envelope. An agent
//! branches on these, so they must stay stable. Numbering matches the other SpaceCorps CLIs.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    Error = 1,
    NotFound = 4,
    InvalidInput = 6,
    NoAccount = 7,
}

impl ErrorCode {
    pub fn name(self) -> &'static str {
        match self {
            ErrorCode::Error => "error",
            ErrorCode::NotFound => "not_found",
            ErrorCode::InvalidInput => "invalid_input",
            ErrorCode::NoAccount => "no_account",
        }
    }
}

#[derive(Debug)]
pub struct Error {
    pub code: ErrorCode,
    pub message: String,
    /// Extra context - the matching profiles, the path that failed, and so on.
    pub detail: Option<String>,
    /// A literal command that fixes this. Agents surface it verbatim.
    pub remediation: Option<String>,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Error { code, message: message.into(), detail: None, remediation: None }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn fix(mut self, remediation: impl Into<String>) -> Self {
        self.remediation = Some(remediation.into());
        self
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Error::new(ErrorCode::InvalidInput, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Error::new(ErrorCode::NotFound, message)
    }

    pub fn no_account(message: impl Into<String>) -> Self {
        Error::new(ErrorCode::NoAccount, message)
    }

    pub fn other(message: impl Into<String>) -> Self {
        Error::new(ErrorCode::Error, message)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::other(e.to_string()).detail("io")
    }
}
