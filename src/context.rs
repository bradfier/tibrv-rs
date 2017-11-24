//! Interface for creating and managing the Rendezvous internal machinery

use tibrv_sys::*;
use message::Msg;
use errors::*;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::ptr::null;
use failure::*;

#[cfg(feature = "tokio")]
use futures::prelude::{Async, AsyncSink, Poll, Sink, StartSend};
#[cfg(feature = "tokio")]
use std::io;

/// A struct representing a Rendezvous transport object.
///
/// A Rendezvous transport can carry messages across a network,
/// between processes, or within a single process.
///
/// `Transport` is configured with three optional parameters:
///
///  - The `service` parameter may be empty, a 'service name' or a port
///  number. The default, or empty, value causes Rendezvous to search for
///  a 'rendezvous' network service in NIS or /etc/services
///  If no such name can be resolved, it defaults to a protocol specific
///  port number.
///
///  - The `network` parameter instructs the Rendezvous daemon to use
///  a particular network for communications on this transport.
///  The parameter consists of up to three parts, separated by semicolons,
///  in the form `network;multicast groups;send address`.
///  For example:
///    * `192.168.1.1` - Network interface only
///    * `eth0;255.1.1.1` - Network interface and multicast group
///
///  - The `daemon` parameter tells the Rendezvous runtime how to
///  find the daemon. The daemon socket is given in the format `hostname:port`.
///  An empty value uses the daemon running on localhost, on the default port.
///
///  Considerable detail on the configuration and behaviour of Rendezvous
///  transports is available in the [TIBCO Rendezvous Concepts][1] guide.
///
///  [1]: https://docs.tibco.com/pub/rv_zos/8.4.5/doc/pdf/TIB_rv_concepts.pdf
pub struct Transport<'a> {
    pub(crate) inner: tibrvTransport,
    phantom: PhantomData<&'a RvCtx>,
}

/// A builder for a Rendezvous transport object.
pub struct TransportBuilder<'a> {
    service: Option<CString>,
    daemon: Option<CString>,
    network: Option<CString>,
    phantom: PhantomData<&'a RvCtx>,
}

impl<'a> TransportBuilder<'a> {
    /// Constructs a new TransportBuilder with the default parameters selected.
    ///
    /// The supplied `RvCtx` must live at least as long as any constructed
    /// transports.
    ///
    /// For further details on `service`, `daemon` and `network`, see the
    /// documentation on [`Transport`].
    ///
    /// [`Transport`]: struct.Transport.html
    pub fn new(_ctx: &'a RvCtx) -> Self {
        TransportBuilder {
            service: None,
            daemon: None,
            network: None,
            phantom: PhantomData,
        }
    }
    /// Sets the `service` parameter.
    pub fn with_service(mut self, service: &str) -> Result<Self, TibrvError> {
        self.service =
            Some(CString::new(service).context(ErrorKind::StrContentError)?);
        Ok(self)
    }

    /// Sets the `daemon` parameter.
    pub fn with_daemon(mut self, daemon: &str) -> Result<Self, TibrvError> {
        self.daemon =
            Some(CString::new(daemon).context(ErrorKind::StrContentError)?);
        Ok(self)
    }

    /// Sets the `network` parameter.
    pub fn with_network(mut self, network: &str) -> Result<Self, TibrvError> {
        self.network =
            Some(CString::new(network).context(ErrorKind::StrContentError)?);
        Ok(self)
    }

    /// Consumes the `TransportBuilder`, creating a `Transport`.
    pub fn create(self) -> Result<Transport<'a>, TibrvError> {
        // 0 is a bogus value, but we need to convince the compiler transport
        // will actually be initialized by _Create
        let mut transport: tibrvTransport = 0;

        let result = unsafe {
            tibrvTransport_Create(
                &mut transport,
                self.service.map_or(null(), |s| s.as_ptr()),
                self.network.map_or(null(), |n| n.as_ptr()),
                self.daemon.map_or(null(), |d| d.as_ptr()),
            )
        };
        result.and_then(|_| Transport {
            inner: transport,
            phantom: PhantomData,
        })
    }
}

/// A struct representing the Rendezvous internal machinery which must be
/// set up before attempting to create `Transports` or `Queues`
pub struct RvCtx {}

impl RvCtx {
    /// Initialize the Rendezvous context
    pub fn new() -> Result<Self, TibrvError> {
        unsafe { tibrv_Open() }.and_then(|_| RvCtx {})
    }

    /// Get the version of Rendezvous this context has bound to.
    pub fn version(&self) -> String {
        unsafe {
            CStr::from_ptr(tibrv_Version())
                .to_string_lossy()
                .into_owned()
        }
    }
}

// tibrv is internally reference counted, so we must ensure each
// tibrv_Open() is followed eventually with a _Close()
impl Drop for RvCtx {
    fn drop(&mut self) {
        unsafe { tibrv_Close() };
    }
}

impl<'a> Transport<'a> {
    /// Extract the daemon parameter from the transport.
    pub fn daemon(&self) -> Result<String, TibrvError> {
        let mut ptr: *const ::std::os::raw::c_char =
            unsafe { ::std::mem::zeroed() };

        unsafe {
            tibrvTransport_GetDaemon(self.inner, &mut ptr).and_then(|_| {
                CStr::from_ptr(ptr).to_string_lossy().into_owned()
            })
        }
    }

    /// Extract the network parameter from the transport.
    pub fn network(&self) -> Result<String, TibrvError> {
        let mut ptr: *const ::std::os::raw::c_char =
            unsafe { ::std::mem::zeroed() };

        unsafe {
            tibrvTransport_GetNetwork(self.inner, &mut ptr).and_then(|_| {
                CStr::from_ptr(ptr).to_string_lossy().into_owned()
            })
        }
    }

    /// Extract the service parameter from the transport.
    pub fn service(&self) -> Result<String, TibrvError> {
        let mut ptr: *const ::std::os::raw::c_char =
            unsafe { ::std::mem::zeroed() };

        unsafe {
            tibrvTransport_GetService(self.inner, &mut ptr).and_then(|_| {
                CStr::from_ptr(ptr).to_string_lossy().into_owned()
            })
        }
    }

    /// Send a `Msg` through this transport.
    ///
    /// Note that `Msg` must be mutable due to the signature
    /// of the C library functions.
    pub fn send(&self, msg: &mut Msg) -> Result<(), TibrvError> {
        unsafe { tibrvTransport_Send(self.inner, msg.inner) }.and_then(|_| ())
    }
}

impl<'a> Drop for Transport<'a> {
    fn drop(&mut self) {
        unsafe { tibrvTransport_Destroy(self.inner) };
    }
}

#[cfg(feature = "tokio")]
impl<'a> Sink for Transport<'a> {
    type SinkItem = Msg;
    type SinkError = io::Error; // Should eventually be tibrv::Error

    // libtibrv doesn't provide an explicit "async send" routine
    // From the documentation it looks like tibrvTransport_Send
    // isn't supposed to block, so we have to just assume it's
    // doing internal buffering.
    fn start_send(&mut self, mut item: Msg)
        -> StartSend<Self::SinkItem, Self::SinkError> {
        // Here we do the send immediately, then always return
        // complete when poll_complete is called later.
        Transport::send(self, &mut item).map_err(|_| {
            io::Error::new(io::ErrorKind::Other, "Unable to send on transport")
        })?;
        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        Ok(Async::Ready(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version() {
        let ctx = RvCtx::new().unwrap();
        assert!(ctx.version().len() > 0);
    }

    #[test]
    fn default_transport() {
        let ctx = RvCtx::new().unwrap();
        let tp = TransportBuilder::new(&ctx).create();
        let _ = tp.map_err(|e| assert_eq!(ErrorKind::TransportError, e.kind()));
    }
}
