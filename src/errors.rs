//! Error types returned by methods in this crate.

use failure::*;
use std::fmt;
use tibrv_sys::tibrv_status;

pub(crate) trait TibrvResult {
    fn map<U, F: FnOnce(Self) -> U>(self, f: F) -> Result<U, TibrvError>
    where
        Self: Sized;
}

/// The error type for operations on the types provided in this crate.
///
/// Errors mostly originate from the underlying TIBCO Rendezvous implementation,
/// but may be generated due to invalid input to the library wrappers.
#[derive(Debug)]
pub struct TibrvError {
    inner: Context<ErrorKind>,
}

/// A list of general error categories.
///
/// This list may grow over time towards and after version 1.0,
/// so it is not recommended to use exhaustive matches.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    /// A provided string could not be converted to a CString.
    #[fail(display = "Invalid CString content")]
    StrContentError,
    /// The external rendezvous machinery failed to initialize.
    #[fail(display = "Failed to initialize Rendezvous")]
    RvInitFailure,
    /// The rendezvous library rejected, or failed to connect to the transport.
    #[fail(display = "Transport error")]
    TransportError,
    /// The producing end of an event queue closed early.
    #[fail(display = "Event queue channel closed prematurely")]
    QueueError,
    /// The Async callback event registration failed to complete properly.
    #[fail(display = "Async event registration failed")]
    AsyncRegError,
    /// A scalar MsgField was passed to a vector decoding method.
    #[fail(display = "Tried to decode a scalar field as a vector")]
    NonVectorFieldError,
    /// There was an attempt to decode a MsgField into a type which didn't
    /// match the internal tag.
    #[fail(display = "Tried to decode a field into an incorrect type")]
    FieldTypeError,
    /// Some other Rendezvous error occurred.
    #[fail(display = "Unknown Error: {}", _0)]
    UnknownError(tibrv_status),
}

// Boilerplate for Failure
// =====================================
impl Fail for TibrvError {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for TibrvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl TibrvError {
    pub fn kind(&self) -> ErrorKind {
        *self.inner.get_context()
    }
}

impl From<ErrorKind> for TibrvError {
    fn from(kind: ErrorKind) -> TibrvError {
        TibrvError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for TibrvError {
    fn from(inner: Context<ErrorKind>) -> TibrvError {
        TibrvError { inner: inner }
    }
}
// =====================================

impl From<tibrv_status> for ErrorKind {
    fn from(status: tibrv_status) -> Self {
        match status {
            tibrv_status::TIBRV_INIT_FAILURE => ErrorKind::RvInitFailure,
            tibrv_status::TIBRV_INVALID_TRANSPORT
            | tibrv_status::TIBRV_SERVICE_NOT_FOUND
            | tibrv_status::TIBRV_NETWORK_NOT_FOUND
            | tibrv_status::TIBRV_DAEMON_NOT_FOUND
            | tibrv_status::TIBRV_DAEMON_NOT_CONNECTED => ErrorKind::TransportError,
            _ => ErrorKind::UnknownError(status),
        }
    }
}

/// Allows easy mapping of `tibrv_error` return codes into
/// `Result<U, TibrvError` types.
///
/// Executes supplied closure if the `tibrv_status` is not `TIBRV_OK`.
impl TibrvResult for tibrv_status {
    fn map<U, F: FnOnce(Self) -> U>(self, f: F) -> Result<U, TibrvError> {
        match self {
            tibrv_status::TIBRV_OK => Ok(f(self)),
            _ => Err(ErrorKind::from(self))?,
        }
    }
}
