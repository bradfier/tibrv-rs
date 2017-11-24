use std::fmt;
use tibrv_sys::tibrv_status;
use failure::*;

pub(crate) trait TibrvResult {
    fn and_then<U, F: FnOnce(Self) -> U>(self, f: F) -> Result<U, TibrvError>
        where Self: Sized;
}

#[derive(Debug)]
pub struct TibrvError {
    inner: Context<ErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "Invalid CString content")]
    StrContentError,
    #[fail(display = "Failed to initialize Rendezvous")]
    RvInitFailure,
    #[fail(display = "Transport error")]
    TransportError,
    #[fail(display = "Event queue channel closed prematurely")]
    QueueError,
    #[fail(display = "Async event registration failed")]
    AsyncRegError,
    #[fail(display = "Tried to decode a scalar field as a vector")]
    NonVectorFieldError,
    #[fail(display = "Tried to decode a field into an incorrect type")]
    FieldTypeError,
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
        TibrvError { inner: Context::new(kind) }
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
            | tibrv_status::TIBRV_DAEMON_NOT_CONNECTED
            => ErrorKind::TransportError,
            _ => ErrorKind::UnknownError(status),
        }
    }
}

impl TibrvResult for tibrv_status {
    fn and_then<U, F: FnOnce(Self) -> U>(self, f: F) -> Result<U, TibrvError> {
        match self {
            tibrv_status::TIBRV_OK => Ok(f(self)),
            _ => Err(ErrorKind::from(self))?,
        }
    }
}
