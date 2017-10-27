//! Asynchronous interfaces for integrating Rendezvous with Tokio

use tibrv_sys::*;
use std::io;
use mio;
use std::sync::{mpsc, Arc, Mutex};
use tokio_core::reactor::{PollEvented, Handle};
use futures::stream::Stream;
use futures::{Async, Poll};

use event::{Queue, Subscription};
use context::{RvCtx, Transport};
use message::Msg;

/// Struct representing an asynchronous Rendezvous event queue.
///
/// Wraps a `Queue` and sets up event callbacks in Rendezvous to
/// drive a `Readiness` stream for use with Tokio.
pub struct AsyncQueue<'a> {
    queue: Queue<'a>,
    listeners: Arc<Mutex<Vec<mio::SetReadiness>>>,
}

impl<'a> AsyncQueue<'a> {
    /// Construct a new asynchronous event queue.
    pub fn new(ctx: &'a RvCtx) -> Result<Self, &'static str> {
        Ok(AsyncQueue {
            queue: Queue::new(ctx)?,
            listeners: Arc::new(Mutex::new(Vec::new())),
        })
    }

    unsafe extern "C" fn callback(_queue: tibrvQueue,
                                  closure: *mut ::std::os::raw::c_void)
                                  -> () {
        // As with the sync version, we can't panic and unwind into the
        // caller, so we catch any recoverable panic and ignore it.
        let _ = ::std::panic::catch_unwind(move || {
            let listen_ptr = closure as *const Mutex<Vec<mio::SetReadiness>>;
            let vec_mutex = Arc::from_raw(listen_ptr);
            {
                let vec = vec_mutex.lock().unwrap();
                for l in &*vec {
                    let _ = l.set_readiness(mio::Ready::readable());
                }
            }
            // Don't run Drop on the listener list
            ::std::mem::forget(vec_mutex);
        });
    }

    fn has_hook(&self) -> bool {
        let mut ptr: tibrvQueueHook = unsafe { ::std::mem::zeroed() };
        let result = unsafe { tibrvQueue_GetHook(self.queue.inner, &mut ptr) };
        match result {
            tibrv_status::TIBRV_OK => ptr.is_some(),
            _ => false,
        }
    }

    /// Asynchronously subscribe to a message subject.
    ///
    /// Sets up the channels as in a synchronous subscription and returns
    /// an `AsyncSub` stream.
    pub fn subscribe(&self, handle: &Handle, tp: &Transport, subject: &str)
                     -> Result<AsyncSub, &'static str> {
        let (registration, ready) = mio::Registration::new2();
        let mut listeners = self.listeners.lock().map_err(|_| "Bork!")?;
        listeners.push(ready);
        let sub = self.queue.subscribe(tp, subject)?;

        if !self.has_hook() {
            // Set up event hook
            let l_arc = Arc::clone(&self.listeners);
            let l_ptr = Arc::into_raw(l_arc);
            let result = unsafe {
                tibrvQueue_SetHook(
                    self.queue.inner,
                    Some(AsyncQueue::callback),
                    l_ptr as *mut ::std::os::raw::c_void,
                )
            };
            if result != tibrv_status::TIBRV_OK {
                return Err("Bork!");
            };
        }
        Ok(AsyncSub {
            sub: sub,
            io: PollEvented::new(registration, handle).map_err(|_| "Bork!")?,
        })
    }
}

/// A stream returned from the `AsyncQueue::subscribe` function representing
/// the incoming messages on the selected subject.
pub struct AsyncSub<'a> {
    sub: Subscription<'a>,
    io: PollEvented<mio::Registration>,
}

impl<'a> AsyncSub<'a> {
    fn next(&self) -> io::Result<Msg> {
        loop {
            // It's possible our queue was pushed into from another
            // event, so optimistically check for a message.
            if let Ok(msg) = self.sub.try_next() {
                return Ok(msg)
            }
            if let Async::NotReady = self.io.poll_read() {
                return Err(io::Error::new(
                        io::ErrorKind::WouldBlock, "not ready"
                ))
            }
            match self.sub.try_next() {
                Err(e) => {
                    if e == mpsc::TryRecvError::Empty {
                        self.io.need_read();

                        return Err(
                            io::Error::new(
                                io::ErrorKind::WouldBlock, "no messages"
                            )
                        )
                    }
                    // Only other error from a Receiver is a broken stream
                    return Err(io::Error::new(io::ErrorKind::BrokenPipe,
                                              "channel closed"));
                }
                Ok(msg) => return Ok(msg),
            }
        }
    }
}

impl<'a> Stream for AsyncSub<'a> {
    type Item = Msg;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Msg>, io::Error> {
        Ok(Async::Ready(Some(try_nb!(self.next()))))
    }
}

#[cfg(test)]
mod tests {
    use context::{RvCtx, TransportBuilder};
    use async::AsyncQueue;
    use tokio_core::reactor::Core;

    #[test]
    fn no_hook() {
        let ctx = RvCtx::new().unwrap();
        let queue = AsyncQueue::new(&ctx).unwrap();
        assert_eq!(false, queue.has_hook());
    }

    #[test]
    #[ignore]
    fn has_hook() {
        let mut core = Core::new().unwrap();

        let ctx = RvCtx::new().unwrap();
        let tp = TransportBuilder::new(&ctx).create().unwrap();
        let queue = AsyncQueue::new(&ctx).unwrap();

        assert_eq!(false, queue.has_hook());
        let sub = queue.subscribe(&core.handle(), &tp, "TEST").unwrap();
        assert_eq!(true, queue.has_hook());
    }
}
