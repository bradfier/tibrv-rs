//! Interface for creating and managing the Rendezvous internal machinery

use errors::*;
use event::{Queue, Subscription};
use failure::*;
use message::Msg;
use std::ffi::{CStr, CString};
use std::mem;
use std::ptr::null;
use tibrv_sys::*;

#[cfg(feature = "tokio")]
use async::{AsyncQueue, AsyncReply, AsyncReq, AsyncSub};
#[cfg(feature = "tokio")]
use futures::prelude::{
    Async, AsyncSink, Future, IntoFuture, Poll, Sink, StartSend, Stream,
};
#[cfg(feature = "tokio")]
use tokio::reactor::Handle;

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
pub struct Transport {
    pub(crate) inner: tibrvTransport,
    context: RvCtx,
}

/// A builder for a Rendezvous transport object.
pub struct TransportBuilder {
    service: Option<CString>,
    daemon: Option<CString>,
    network: Option<CString>,
    context: RvCtx,
}

impl TransportBuilder {
    /// Constructs a new TransportBuilder with the default parameters selected.
    ///
    /// The supplied `RvCtx` must live at least as long as any constructed
    /// transports.
    ///
    /// For further details on `service`, `daemon` and `network`, see the
    /// documentation on [`Transport`].
    ///
    /// [`Transport`]: struct.Transport.html
    pub fn new(ctx: RvCtx) -> Self {
        TransportBuilder {
            service: None,
            daemon: None,
            network: None,
            context: ctx,
        }
    }
    /// Sets the `service` parameter.
    pub fn with_service(mut self, service: &str) -> Result<Self, TibrvError> {
        self.service = Some(CString::new(service).context(ErrorKind::StrContentError)?);
        Ok(self)
    }

    /// Sets the `daemon` parameter.
    pub fn with_daemon(mut self, daemon: &str) -> Result<Self, TibrvError> {
        self.daemon = Some(CString::new(daemon).context(ErrorKind::StrContentError)?);
        Ok(self)
    }

    /// Sets the `network` parameter.
    pub fn with_network(mut self, network: &str) -> Result<Self, TibrvError> {
        self.network = Some(CString::new(network).context(ErrorKind::StrContentError)?);
        Ok(self)
    }

    /// Consumes the `TransportBuilder`, creating a `Transport`.
    pub fn create(self) -> Result<Transport, TibrvError> {
        // 0 is a bogus value, but we need to convince the compiler transport
        // will actually be initialized by _Create
        let mut transport: tibrvTransport = 0;
        let ctx = self.context.clone();

        let result = unsafe {
            tibrvTransport_Create(
                &mut transport,
                self.service.as_ref().map_or(null(), |s| s.as_ptr()),
                self.network.as_ref().map_or(null(), |n| n.as_ptr()),
                self.daemon.as_ref().map_or(null(), |d| d.as_ptr()),
            )
        };
        result.map(|_| Transport {
            inner: transport,
            context: ctx,
        })
    }
}

/// A struct representing the Rendezvous internal machinery which must be
/// set up before attempting to create `Transports` or `Subscriptions`
pub struct RvCtx {}

impl RvCtx {
    /// Initialize the Rendezvous context
    pub fn new() -> Result<Self, TibrvError> {
        unsafe { tibrv_Open() }.map(|_| RvCtx {})
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

// TODO: Handle a failed clone?
impl Clone for RvCtx {
    fn clone(&self) -> RvCtx {
        unsafe {
            tibrv_Open();
        }
        RvCtx {}
    }
}

// tibrv is internally reference counted, so we must ensure each
// tibrv_Open() is followed eventually with a _Close()
impl Drop for RvCtx {
    fn drop(&mut self) {
        unsafe { tibrv_Close() };
    }
}

impl Transport {
    /// Extract the daemon parameter from the transport.
    pub fn daemon(&self) -> Result<String, TibrvError> {
        let mut ptr: *const ::std::os::raw::c_char = unsafe { ::std::mem::zeroed() };

        unsafe {
            tibrvTransport_GetDaemon(self.inner, &mut ptr)
                .map(|_| CStr::from_ptr(ptr).to_string_lossy().into_owned())
        }
    }

    /// Extract the network parameter from the transport.
    pub fn network(&self) -> Result<String, TibrvError> {
        let mut ptr: *const ::std::os::raw::c_char = unsafe { ::std::mem::zeroed() };

        unsafe {
            tibrvTransport_GetNetwork(self.inner, &mut ptr)
                .map(|_| CStr::from_ptr(ptr).to_string_lossy().into_owned())
        }
    }

    /// Extract the service parameter from the transport.
    pub fn service(&self) -> Result<String, TibrvError> {
        let mut ptr: *const ::std::os::raw::c_char = unsafe { ::std::mem::zeroed() };

        unsafe {
            tibrvTransport_GetService(self.inner, &mut ptr)
                .map(|_| CStr::from_ptr(ptr).to_string_lossy().into_owned())
        }
    }

    /// Creates an unique inbox subject name.
    ///
    /// The created subject name will be unique for this transport.
    pub fn create_inbox(&self) -> Result<String, TibrvError> {
        let mut buf: Vec<::std::os::raw::c_char> = vec![0; 128];
        unsafe {
            tibrvTransport_CreateInbox(self.inner, buf.as_mut_ptr(), 128).map(|_| {
                CString::from_vec_unchecked(buf.into_iter().map(|c| c as u8).collect())
                    .into_string()
                    .expect("C String from CreateInbox contained invalid data")
                    .trim_end_matches('\0')
                    .to_owned()
            })
        }
    }

    /// Send a `Msg` through this transport.
    ///
    /// Note that `Msg` must be mutable due to the signature
    /// of the C library functions.
    pub fn send(&self, msg: &mut Msg) -> Result<(), TibrvError> {
        unsafe { tibrvTransport_Send(self.inner, msg.inner) }.map(|_| ())
    }

    /// Subscribe to a message subject.
    ///
    /// Sets up a Rendezvous message queue, along with a callback which
    /// copies messages from the event queue into an `mspc::channel` for
    /// consumption from Rust.
    ///
    /// Subject must be valid ASCII, wildcards are accepted, although a
    /// wildcard-only subject is not.
    pub fn subscribe(&self, subject: &str) -> Result<Subscription, TibrvError> {
        Queue::new(self.context.clone())?.subscribe(&self, subject)
    }

    /// Send a synchronous request on the given subject, blocking until
    /// a response is received or `timeout` seconds have elapsed.
    ///
    /// A `None` parameter for `timeout` means block indefinitely.
    pub fn request(
        &self,
        msg: &mut Msg,
        timeout: Option<f64>,
    ) -> Result<Msg, TibrvError> {
        let mut ptr: tibrvMsg = unsafe { mem::zeroed() };
        unsafe {
            tibrvTransport_SendRequest(
                self.inner,
                msg.inner,
                &mut ptr,
                timeout.unwrap_or(-1.0),
            ).map(|_| Msg { inner: ptr })
        }
    }

    /// Listen on this subject and respond to requests using the
    /// supplied closure.
    ///
    /// The closure consumes the incoming request, and you may either re-use
    /// the same structure or create a new `Msg` for your response.
    ///
    /// If `reply_subject` is not set on the received message (i.e it is a
    /// plain message rather than a request), it is not processed and is
    /// dropped.
    pub fn serve<F>(&self, subject: &str, f: F) -> Result<(), TibrvError>
    where
        F: Fn(Msg) -> Result<Msg, TibrvError>,
    {
        let sub = self.subscribe(subject)?;
        loop {
            let incoming = sub.next()?;
            let reply_subj = incoming.get_reply_subject()?;

            if reply_subj.is_some() {
                let mut reply = f(incoming)?;
                reply.set_send_subject(&reply_subj.unwrap())?;
                self.send(&mut reply)?
            }
        }
    }

    #[cfg(feature = "tokio")]
    pub fn async_serve<F, G>(
        self,
        handle: &Handle,
        subject: &str,
        f: F,
    ) -> impl Future<Item = (), Error = TibrvError>
    where
        F: Fn(Msg) -> G,
        G: IntoFuture<Item = Msg, Error = TibrvError>,
    {
        let sub = self.async_sub(handle, subject).unwrap();

        sub.and_then(move |msg| AsyncReply {
            subject: msg.get_reply_subject().unwrap().unwrap(),
            future: f(msg).into_future(),
        }).forward(self)
            .then(|_| Ok(()))
    }

    #[cfg(feature = "tokio")]
    /// Asynchronously subscribe to a message subject.
    ///
    /// Sets up the queue and channels as in a synchronous subscription, and
    /// returns an `AsyncSub` stream.
    pub fn async_sub(
        &self,
        handle: &Handle,
        subject: &str,
    ) -> Result<AsyncSub, TibrvError> {
        AsyncQueue::new(self.context.clone())?.subscribe(handle, &self, subject)
    }

    #[cfg(feature = "tokio")]
    /// Asynchronously send a request on the given subject.
    ///
    /// Returns an `AsyncReq` future representing the response.
    pub fn async_req(
        &self,
        handle: &Handle,
        msg: &mut Msg,
    ) -> Result<AsyncReq, TibrvError> {
        let inbox = self.create_inbox()?;
        let sub = self.async_sub(handle, &inbox)?;

        msg.set_reply_subject(&inbox)?;
        self.send(msg)?;

        Ok(AsyncReq::new(sub))
    }
}

impl Drop for Transport {
    fn drop(&mut self) {
        unsafe { tibrvTransport_Destroy(self.inner) };
    }
}

#[cfg(feature = "tokio")]
impl Sink for Transport {
    type SinkItem = Msg;
    type SinkError = TibrvError;

    // libtibrv doesn't provide an explicit "async send" routine
    // From the documentation it looks like tibrvTransport_Send
    // isn't supposed to block, so we have to just assume it's
    // doing internal buffering.
    fn start_send(
        &mut self,
        mut item: Msg,
    ) -> StartSend<Self::SinkItem, Self::SinkError> {
        // Here we do the send immediately, then always return
        // complete when poll_complete is called later.
        Transport::send(self, &mut item)?;
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
        assert!(!ctx.version().is_empty());
    }

    #[test]
    fn default_transport() {
        let ctx = RvCtx::new().unwrap();
        let tp = TransportBuilder::new(ctx).create();
        let _ = tp.map_err(|e| assert_eq!(ErrorKind::TransportError, e.kind()));
    }

    #[test]
    #[ignore]
    fn create_inbox() {
        let ctx = RvCtx::new().unwrap();
        let tp = TransportBuilder::new(ctx).create().unwrap();
        let inbox = tp.create_inbox().unwrap();
        assert!(!inbox.is_empty());
        assert!(inbox.contains("INBOX"));
        println!("{:?}", inbox);
    }

    #[test]
    #[ignore]
    fn timeout_request() {
        let ctx = RvCtx::new().unwrap();
        let tp = TransportBuilder::new(ctx).create().unwrap();

        let mut msg = Msg::new().unwrap();
        msg.set_send_subject("REQUEST.TEST").unwrap();
        let req = tp.request(&mut msg, Some(1.0));
        assert!(req.is_err());
        let _ = req.map_err(|e| {
            assert_eq!(
                ErrorKind::UnknownError(tibrv_status::TIBRV_TIMEOUT),
                e.kind()
            )
        });
    }
}
