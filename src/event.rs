//! Interfaces for dealing with inbound events from Rendezvous

use tibrv_sys::*;
use std::mem;
use std::ffi::CString;
use std::sync::mpsc;
use context::{RvCtx, Transport};
use message::{Msg, BorrowedMsg};
use std::marker::PhantomData;

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
        // We can panic if the channel has been closed, but we have
        // no mechanism by which to report this via Rendezvous, therefore
        // if the .send() method fails, we just ignore it.
        let _ =::std::panic::catch_unwind(move || {
            let sender: Box<mpsc::Sender<Msg>> =
                Box::from_raw(closure as *mut mpsc::Sender<Msg>);
            let msg = BorrowedMsg { inner: message };
            sender.send(msg.detach().unwrap()).unwrap();
        });
    }

    pub fn subscribe(&self, tp: &Transport, subject: &str)
        -> Result<mpsc::Receiver<Msg>, &'static str>  {
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
            Ok(recv)
    }

    // Blocking dispatch
    pub fn dispatch(&self) -> Result<(), &'static str> {
        match unsafe { tibrvQueue_TimedDispatch(self.inner, -1.0) } {
            tibrv_status::TIBRV_OK => Ok(()),
            _ => Err("Bork!"),
        }
    }
}

impl<'a> Drop for Queue<'a> {
    fn drop(&mut self) {
        unsafe {
            tibrvQueue_DestroyEx(self.inner, None, ::std::ptr::null());
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
}
