//! Interface for creating and managing the Rendezvous internal machinery

use tibrv_sys::*;
use message::Msg;
use event::Queue;
use std::ffi::{CString,CStr};
use std::marker::PhantomData;
use std::ptr::null;

#[cfg(feature = "tokio")]
use async::AsyncQueue;
#[cfg(feature = "tokio")]
use futures::prelude::{Sink, Async, AsyncSink, StartSend, Poll};
#[cfg(feature = "tokio")]
use std::io;


/// A struct representing a Rendezvous transport object.
///
/// A Rendezvous transport can carry messages across a network,
/// between processes, or within a single process.
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
    /// Sets the `service` parameter.
    pub fn with_service(mut self, service: &str) -> Result<Self, &'static str> {
        self.service = Some(CString::new(service).map_err(|_| "Bork!")?);
        Ok(self)
    }

    /// Sets the `daemon` parameter.
    pub fn with_daemon(mut self, daemon: &str) -> Result<Self, &'static str> {
        self.daemon = Some(CString::new(daemon).map_err(|_| "Bork!")?);
        Ok(self)
    }

    /// Sets the `network` parameter.
    pub fn with_network(mut self, network: &str) -> Result<Self, &'static str> {
        self.network = Some(CString::new(network).map_err(|_| "Bork!")?);
        Ok(self)
    }

    /// Consumes the `TransportBuilder`, creating a `Transport`.
    pub fn create(self) -> Result<Transport<'a>, &'static str> {
        // 0 is a bogus value, but we need to convince the compiler transport
        // will actually be initialized by _Create
        let mut transport: tibrvTransport = 0;

        let result = unsafe {
            tibrvTransport_Create(
                &mut transport,
                self.service.map_or(null(), |s| s.as_ptr()),
                self.network.map_or(null(), |n| n.as_ptr()),
                self.daemon.map_or(null(), |d| d.as_ptr())
            )
        };

        match result {
            tibrv_status::TIBRV_OK => Ok(
                Transport {
                    inner: transport,
                    phantom: PhantomData,
                }
            ),
            _ => Err("Bork!"),
        }
    }
}


/// A struct representing the Rendezvous internal machinery which must be
/// set up before attempting to create `Transports` or `Queues` 
pub struct RvCtx { }

impl RvCtx {
    /// Initialize the Rendezvous context
    pub fn new() -> Result<Self, &'static str> {
        match unsafe { tibrv_Open() } {
            tibrv_status::TIBRV_OK => Ok(RvCtx { }),
            _ => Err("Bork!"),
        }
    }

    /// Get the version of Rendezvous this context has bound to.
    pub fn version(&self) -> String {
        unsafe {
            CStr::from_ptr(tibrv_Version())
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Gets a transport builder, with the default parameters set.
    pub fn transport(&self) -> TransportBuilder {
        TransportBuilder {
            service: None,
            daemon: None,
            network: None,
            phantom: PhantomData,
        }
    }

    /// Creates and returns an event queue.
    pub fn queue<'a>(&'a self) -> Result<Queue, &'static str> {
        Queue::new(self)
    }

    #[cfg(feature = "tokio")]
    /// Creates and returns an asynchronous event queue.
    pub fn async_queue<'a>(&'a self) -> Result<AsyncQueue, &'static str> {
        AsyncQueue::new(self)
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
    pub fn daemon(&self) -> Result<String, &'static str> {
        let mut ptr: *const ::std::os::raw::c_char
            = unsafe { ::std::mem::zeroed() };

        let result = unsafe {
            tibrvTransport_GetDaemon(self.inner, &mut ptr)
        };

        match result {
            tibrv_status::TIBRV_OK => Ok(
                unsafe {
                    CStr::from_ptr(ptr).to_string_lossy().into_owned()
                }
            ),
            _ => Err("Bork!"),
        }
    }

    /// Extract the network parameter from the transport.
    pub fn network(&self) -> Result<String, &'static str> {
        let mut ptr: *const ::std::os::raw::c_char
            = unsafe { ::std::mem::zeroed() };

        let result = unsafe {
            tibrvTransport_GetNetwork(self.inner, &mut ptr)
        };

        match result {
            tibrv_status::TIBRV_OK => Ok(
                unsafe {
                    CStr::from_ptr(ptr).to_string_lossy().into_owned()
                }
            ),
            _ => Err("Bork!"),
        }
    }

    /// Extract the service parameter from the transport.
    pub fn service(&self) -> Result<String, &'static str> {
        let mut ptr: *const ::std::os::raw::c_char
            = unsafe { ::std::mem::zeroed() };

        let result = unsafe {
            tibrvTransport_GetService(self.inner, &mut ptr)
        };

        match result {
            tibrv_status::TIBRV_OK => Ok(
                unsafe {
                    CStr::from_ptr(ptr).to_string_lossy().into_owned()
                }
            ),
            _ => Err("Bork!"),
        }
    }

    /// Send a `Msg` through this transport.
    ///
    /// Note that `Msg` must be mutable due to the signature
    /// of the C library functions.
    pub fn send(&self, msg: &mut Msg) -> Result<(), &'static str> {
        match unsafe { tibrvTransport_Send(self.inner, msg.inner) } {
            tibrv_status::TIBRV_OK => Ok(()),
            _ => Err("Bork!"),
        }
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
}
