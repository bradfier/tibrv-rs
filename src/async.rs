//! Asynchronous interfaces for integrating Rendezvous with Tokio
//!
//! This module contains all the Tokio support for interacting
//! with Rendezvous event streams asynchronously.

use futures::stream::Stream;
use futures::{Async, Future, Poll};
use mio;
use std::sync::mpsc;
use tibrv_sys::*;
use tokio::reactor::{Handle, PollEvented2};

use context::{RvCtx, Transport};
use errors::*;
use event::{Queue, Subscription};
use failure::*;
use message::Msg;

/// Struct representing an asynchronous Rendezvous event queue.
///
/// Wraps a `Queue` and sets up event callbacks in Rendezvous to
/// drive a `Readiness` stream for use with Tokio.
pub(crate) struct AsyncQueue {
    queue: Queue,
}

impl AsyncQueue {
    /// Construct a new asynchronous event queue.
    pub fn new(ctx: RvCtx) -> Result<Self, TibrvError> {
        Ok(AsyncQueue {
            queue: Queue::new(ctx)?,
        })
    }

    unsafe extern "C" fn callback(
        _queue: tibrvQueue,
        closure: *mut ::std::os::raw::c_void,
    ) {
        // As with the sync version, we can't panic and unwind into the
        // caller, so we catch any recoverable panic and ignore it.
        let _ = ::std::panic::catch_unwind(move || {
            let listen_ptr = closure as *mut mio::SetReadiness;
            let _ =(&*listen_ptr).set_readiness(mio::Ready::readable());
        });
    }

    #[allow(dead_code)]
    fn has_hook(&self) -> bool {
        let mut ptr: tibrvQueueHook = unsafe { ::std::mem::zeroed() };
        let result = unsafe { tibrvQueue_GetHook(self.queue.inner, &mut ptr) };
        match result {
            TIBRV_OK => ptr.is_some(),
            _ => false,
        }
    }

    /// Asynchronously subscribe to a message subject.
    ///
    /// Sets up the channels as in a synchronous subscription and returns
    /// an `AsyncSub` stream.
    pub fn subscribe(
        self,
        handle: &Handle,
        tp: &Transport,
        subject: &str,
    ) -> Result<AsyncSub, TibrvError> {
        let (registration, ready) = mio::Registration::new2();

        let sub = self.queue.subscribe(tp, subject)?;

        // Set up event hook
        let listener = Box::new(ready);
        let l_ptr = &*listener as *const mio::SetReadiness;
        let result = unsafe {
            tibrvQueue_SetHook(
                sub.queue.inner,
                Some(AsyncQueue::callback),
                l_ptr as *mut ::std::os::raw::c_void,
            )
        };
        if result != TIBRV_OK {
            Err(ErrorKind::AsyncRegError)?;
        };

        Ok(AsyncSub {
            sub,
            io: PollEvented2::new_with_handle(registration, handle)
                .context(ErrorKind::AsyncRegError)?,
            _listener: listener,
        })
    }
}

/// A stream returned from the `Transport::async_sub` function representing
/// the incoming messages on the selected subject.
pub struct AsyncSub {
    sub: Subscription,
    io: PollEvented2<mio::Registration>,
    // We need to retain ownership of the SetReadiness side of the mio registration
    _listener: Box<mio::SetReadiness>,
}

impl AsyncSub {
    // TODO Create a more specific ErrorKind for these failures
    fn next(&mut self) -> Result<Async<Option<Msg>>, TibrvError> {
        // It's possible our queue was pushed into from another
        // event, so optimistically check for a message.
        if let Ok(msg) = self.sub.try_next() {
            return Ok(Async::Ready(Some(msg)));
        }
        let ready = mio::Ready::readable();
        if let Ok(Async::NotReady) = self.io.poll_read_ready(ready) {
            return Ok(Async::NotReady);
        }
        match self.sub.try_next() {
            Err(e) => {
                if e == mpsc::TryRecvError::Empty {
                    self.io
                        .clear_read_ready(ready)
                        .expect("Failed clearing mio readiness");
                    return Ok(Async::NotReady);
                }
                // Only other error from a Receiver is a broken stream
                Err(ErrorKind::QueueError.into())
            }
            Ok(msg) => Ok(Async::Ready(Some(msg))),
        }
    }
}

impl Stream for AsyncSub {
    type Item = Msg;
    type Error = TibrvError;

    fn poll(&mut self) -> Poll<Option<Msg>, Self::Error> {
        Ok(self.next()?)
    }
}

/// A `Future` representing an incomplete Rendezvous request.
///
/// This structure is produced by the `Transport::async_req` method.
pub struct AsyncReq {
    sub: AsyncSub,
}

impl AsyncReq {
    pub fn new(sub: AsyncSub) -> Self {
        AsyncReq { sub }
    }
}

impl Future for AsyncReq {
    type Item = Msg;
    type Error = TibrvError;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        match self.sub.poll().unwrap() {
            Async::Ready(Some(v)) => Ok(Async::Ready(v)),
            Async::Ready(None) => Err(ErrorKind::QueueError.into()),
            Async::NotReady => Ok(Async::NotReady),
        }
    }
}

pub(crate) struct AsyncReply<F> {
    pub subject: String,
    pub future: F,
}

impl<F> Future for AsyncReply<F>
where
    F: Future<Item = Msg, Error = TibrvError>,
{
    type Item = F::Item;
    type Error = F::Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        match self.future.poll()? {
            Async::Ready(mut msg) => {
                msg.set_send_subject(&self.subject).unwrap();
                Ok(Async::Ready(msg))
            }
            Async::NotReady => Ok(Async::NotReady),
        }
    }
}

#[cfg(test)]
mod tests {
    use async::AsyncQueue;
    use context::{RvCtx, TransportBuilder};
    use tokio::reactor::Handle;

    #[test]
    fn no_hook() {
        let ctx = RvCtx::new().unwrap();
        let queue = AsyncQueue::new(ctx).unwrap();
        assert_eq!(false, queue.has_hook());
    }

    #[test]
    #[ignore]
    fn has_hook() {
        let handle = Handle::default();

        let ctx = RvCtx::new().unwrap();
        let tp = TransportBuilder::new(ctx.clone()).create().unwrap();
        let queue = AsyncQueue::new(ctx.clone()).unwrap();

        assert_eq!(false, queue.has_hook());
        let _ = queue.subscribe(&handle, &tp, "TEST").unwrap();
    }
}
