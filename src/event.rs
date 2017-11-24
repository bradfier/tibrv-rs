//! Interfaces for dealing with inbound events from Rendezvous

use tibrv_sys::*;
use std::mem;
use std::ffi::CString;
use std::sync::mpsc;
use context::{RvCtx, Transport};
use message::{Msg, BorrowedMsg};
use errors::*;
use failure::*;
use std::marker::PhantomData;

unsafe extern "C" fn sync_callback(_event: tibrvEvent,
                                   message: tibrvMsg,
                                   closure: *mut ::std::os::raw::c_void)
-> () {
    // If anything goes wrong in this callback, we have no
    // way to indicate that to Rendezvous without causing an abort.
    // Instead we catch any recoverable unwind.
    let _ =::std::panic::catch_unwind(move || {
        let sender: Box<mpsc::Sender<Msg>> =
            Box::from_raw(closure as *mut mpsc::Sender<Msg>);
        let msg = BorrowedMsg { inner: message };
        sender.send(msg.detach().unwrap()).unwrap();
        ::std::mem::forget(sender); // Don't run Drop on the channel
    });
}

/// Struct representing a Rendezvous event queue.
///
/// Represents a queue of events waiting for dispatch, at present
/// only message queues are implemented, although the library supports
/// IO (socket) events and arbitrary timers as well.
pub struct Queue<'a> {
    pub(crate) inner: tibrvQueue,
    phantom: PhantomData<&'a RvCtx>,
}

impl<'a> Queue<'a> {
    /// Constructs a new event queue.
    ///
    /// The supplied `RvCtx` must live at least as long as any created
    /// queues.
    pub fn new(_ctx: &'a RvCtx) -> Result<Self, TibrvError> {
        let mut ptr: tibrvQueue = unsafe { mem::zeroed() };
        unsafe { tibrvQueue_Create(&mut ptr) }
            .and_then(|_| Queue { inner: ptr, phantom: PhantomData })
    }

    /// Get the number of events waiting in the queue.
    pub fn count(&self) -> Result<u32, TibrvError> {
        let mut ptr: u32 = 0;
        unsafe { tibrvQueue_GetCount(self.inner, &mut ptr) }.and_then(|_| ptr)
    }

    /// Subscribe to a message subject.
    ///
    /// Sets up the callback to copy messages from the event
    /// queue into a `mpsc::channel` for consumption from Rust.
    ///
    /// Requires a reference to a valid `Transport` on which to listen.
    ///
    /// Subject must be valid ASCII, wildcards are accepted, although
    /// a wildcard-only subject is not.
    pub fn subscribe(&self, tp: &Transport, subject: &str)
        -> Result<Subscription, TibrvError>  {
        let (send, recv) = mpsc::channel();
        let subject_c = CString::new(subject)
            .context(ErrorKind::StrContentError)?;

        let mut ptr: tibrvEvent = unsafe { mem::zeroed() };
        let send_ptr = Box::into_raw(Box::new(send.clone()));
        unsafe {
            tibrvEvent_CreateListener(
                &mut ptr,
                self.inner,
                Some(sync_callback),
                tp.inner,
                subject_c.as_ptr(),
                send_ptr as *const ::std::os::raw::c_void
                )
        }.and_then(|_| Subscription {
            event: ptr,
            queue: self,
            channel: recv
        })
    }

}

impl<'a> Drop for Queue<'a> {
    fn drop(&mut self) {
        unsafe {
            tibrvQueue_DestroyEx(self.inner, None, ::std::ptr::null());
        }
    }
}

/// Represents a subscription to a subject.
///
/// Wraps the event, the event queue, and the `mpsc::Receiver`
/// containing the `Msg` data.
pub struct Subscription<'a> {
    event: tibrvEvent,
    queue: &'a Queue<'a>,
    channel: mpsc::Receiver<Msg>,
}

impl<'a> Subscription<'a> {
    // Blocking dispatch
    fn dispatch(&self) -> Result<(), TibrvError> {
        unsafe { tibrvQueue_TimedDispatch(self.queue.inner, -1.0) }
            .and_then(|_| ())
    }

    // Non blocking try-dispatch.
    fn poll(&self) -> Result<(), TibrvError> {
        unsafe { tibrvQueue_TimedDispatch(self.queue.inner, 0.0) }
            .and_then(|_| ())
    }

    /// Get the next message available on this subscription.
    ///
    /// Blocks until a message is available in the queue.
    pub fn next(&self) -> Result<Msg, TibrvError> {
        if let Ok(m) = self.channel.try_recv() {
            return Ok(m)
        }
        self.dispatch()?;
        self.channel.recv().context(ErrorKind::QueueError)
            .map_err(|e| TibrvError::from(e))
    }

    pub fn try_next(&self) -> Result<Msg, mpsc::TryRecvError> {
        let _ = self.poll(); // Ignore this "error"
        self.channel.try_recv()
    }
}

impl<'a> Drop for Subscription<'a> {
    fn drop(&mut self) {
        unsafe {
            tibrvEvent_DestroyEx(
                self.event,
                None
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use context::{RvCtx, TransportBuilder};
    use event::Queue;

    #[test]
    fn creation() {
        let ctx = RvCtx::new().unwrap();
        let queue = Queue::new(&ctx);
        assert!(queue.is_ok());
        assert_eq!(0, queue.unwrap().count().unwrap());
    }

    #[ignore] // Requires a running rvd
    #[test]
    fn subscribe() {
        let ctx = RvCtx::new().unwrap();
        let queue = Queue::new(&ctx).unwrap();
        let tp = TransportBuilder::new(&ctx).create().unwrap();
        let sub = queue.subscribe(&tp, "TEST");
        assert!(sub.is_ok());
    }
}
