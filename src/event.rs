//! Interfaces for dealing with inbound events from Rendezvous

use tibrv_sys::*;
use std::mem;
use std::ffi::CString;
use std::sync::mpsc;
use context::{RvCtx, Transport};
use message::{Msg, BorrowedMsg};
use std::marker::PhantomData;

/// Struct representing a Rendezvous event queue.
///
/// Represents a queue of events waiting for dispatch, at present
/// only message queues are implemented, although the library supports
/// IO (socket) events and arbitrary timers as well.
pub struct Queue<'a> {
    inner: tibrvQueue,
    phantom: PhantomData<&'a RvCtx>,
}

impl<'a> Queue<'a> {
    pub(crate) fn new(_ctx: &'a RvCtx) -> Result<Self, &'static str> {
        let mut ptr: tibrvQueue = unsafe { mem::zeroed() };
        match unsafe { tibrvQueue_Create(&mut ptr) } {
            tibrv_status::TIBRV_OK => Ok(
                Queue {
                    inner: ptr,
                    phantom: PhantomData,
                }
            ),
            _ => Err("Bork!"),
        }
    }

    /// Get the number of events waiting in the queue.
    pub fn count(&self) -> Result<u32, &'static str> {
        let mut ptr: u32 = 0;
        match unsafe { tibrvQueue_GetCount(self.inner, &mut ptr) } {
            tibrv_status::TIBRV_OK => Ok(ptr),
            _ => Err("Bork!"),
        }
    }

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
        });
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
        -> Result<Subscription, &'static str>  {
            let (send, recv) = mpsc::channel();
            let subject_c = CString::new(subject).map_err(|_| "Bork!")?;
            let mut ptr: tibrvEvent = unsafe { mem::zeroed() };
            let send_ptr = Box::into_raw(Box::new(send.clone()));
            let result = unsafe {
                tibrvEvent_CreateListener(
                    &mut ptr,
                    self.inner,
                    Some(Queue::sync_callback),
                    tp.inner,
                    subject_c.as_ptr(),
                    send_ptr as *const ::std::os::raw::c_void
                    )
            };
            if result != tibrv_status::TIBRV_OK {
                return Err("Bork!");
            };
            Ok(Subscription { event: ptr, queue: self, channel: recv })
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
    fn dispatch(&self) -> Result<(), &'static str> {
        match unsafe { tibrvQueue_TimedDispatch(self.queue.inner, -1.0) } {
            tibrv_status::TIBRV_OK => Ok(()),
            _ => Err("Bork!"),
        }
    }

    /// Get the next message available on this subscription.
    ///
    /// Blocks until a message is available in the queue.
    pub fn next(&self) -> Result<Msg, &'static str> {
        self.dispatch()?;
        self.channel.recv().map_err(|_| "Bork!")
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
    use context::RvCtx;

    #[test]
    fn creation() {
        let ctx = RvCtx::new().unwrap();
        let queue = ctx.queue();
        assert!(queue.is_ok());
        assert_eq!(0, queue.unwrap().count().unwrap());
    }

    #[ignore] // Requires a running rvd
    #[test]
    fn subscribe() {
        let ctx = RvCtx::new().unwrap();
        let queue = ctx.queue().unwrap();
        let tp = ctx.transport().create().unwrap();
        let sub = queue.subscribe(&tp, "TEST");
        assert!(sub.is_ok());
    }
}
