use core::{fmt, str::Utf8Error};
use std::{error, io};

pub type Result<T> = std::result::Result<T, Error>;

/// A custom `Error` enum for handling errors in the library.
///
/// # Variants
///
/// * `ConsumeError(String)` - Represents an error that occurs while consuming values from the test case.
#[derive(Debug)]
pub enum Error {
    ConsumeError(String),
    IoError(io::Error),
    CoreIdsUnavailable,
    WritingCrashingInput(io::Error),
    WritingTestcase(io::Error),
    CreatingDir(String),
    TargetNotExecutable(String),
    PathDoesNotExist(String),
    ReadingTestcase(io::Error),
    SpawningTarget(io::Error),
    WaitingForTarget(io::Error),
    NotADir(String),
    NotEmpty(String),
    JoiningThread,
    Fatal(String),
    ConversionError,
}

impl Error {
    /// Creates a new `Error` instance with the given error message.
    ///
    /// # Arguments
    ///
    /// * `s` - A `&str` containing the error message.
    ///
    /// # Returns
    ///
    /// A new `Error` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use errors::Error;
    ///
    /// let err = Error::new("An error occurred");
    /// match err {
    ///     Error::ConsumeError(msg) => assert_eq!(msg, "An error occurred"),
    ///     _ => todo!(),
    /// }
    /// ```
    pub fn new(s: &str) -> Self {
        Self::ConsumeError(s.to_owned())
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Self {
        Self::ConsumeError(err.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::ConsumeError(e) => write!(f, "{e}"),
            Self::IoError(e) => write!(f, "{e}"),
            Self::CoreIdsUnavailable => write!(f, "Core IDs unavailable"),
            Self::WritingCrashingInput(e) => write!(f, "Writing crashing input: {e}"),
            Self::CreatingDir(e) => write!(f, "Creating directory: {e}"),
            Self::TargetNotExecutable(e) => write!(f, "Target not executable: {e}"),
            Self::PathDoesNotExist(e) => write!(f, "Path does not exist: {e}"),
            Self::ReadingTestcase(e) => write!(f, "Reading testcase: {e}"),
            Self::WritingTestcase(e) => write!(f, "Writing testcase: {e}"),
            Self::SpawningTarget(e) => write!(f, "Spawning target: {e}"),
            Self::WaitingForTarget(e) => write!(f, "Waiting for target {e}"),
            Self::NotADir(e) => write!(f, "Not a directory: {e}"),
            Self::NotEmpty(e) => write!(f, "Directory not empty: {e}"),
            Self::JoiningThread => write!(f, "Joining threads"),
            Self::Fatal(e) => write!(f, "Fatal error: {e}"),
            Self::ConversionError => write!(f, "Conversion error: "),
        }
    }
}

impl error::Error for Error {}
