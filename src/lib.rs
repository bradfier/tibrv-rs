//! # Rendezvous bindings for Rust
//!
//! This library provides bindings to the `libtibrv` C library, distributed
//! by [TIBCO][1] for interacting with the Rendezvous message-oriented-middleware.
//! The library itself is still a work in progress, so some features may be missing,
//! however those which are implemented should be fully working.
//!
//! [1]: https://www.tibco.com/products/tibco-rendezvous
//!
//! The tibrv library attempts to make using libtibrv as ergonomic as possible,
//! and includes an optional, Tokio-based asynchronous layer.
//!
//! ## Environment Setup
//!
//! To use tibrv, you need to set up your environment so that your Rendezvous
//! distribution (from TIBCO) is pointed to by the `TIBRV` variable, and
//! the libtibrv library must be available in your `LD_LIBRARY_PATH`.
//!
//! For example:
//!
//! ```sh,no_run
//! export TIBRV=$HOME/tibco/tibrv8.4/
//! export LD_LIBRARY_PATH=$TIBRV/lib:$LD_LIBRARY_PATH
//! ```
//!
//! ## Working with Messages
//!
//! A message is the main structure used to encapsulate data sent or received
//! via Rendezvous.
//!
//! ### Creating a message, and setting the subject
//!
//! ```
//! use tibrv::message::Msg;
//!
//! let mut msg = Msg::new().expect("Failed to create message.");
//! msg.set_send_subject("TEST_SUBJECT");
//! ```
//!
//! ### Adding a field to a message
//! All scalar and vector types are copied into the message
//! once `add_field` is called, so the field constructors
//! take borrows to avoid double copying.
//!
//! ```
//! use tibrv::message::Msg;
//! use tibrv::field::Builder;
//!
//! let mut msg = Msg::new().expect("Failed to create message.");
//!
//! let data: u32 = 42;
//! let mut field = Builder::new(&data)
//!     .with_name("fieldName")
//!     .encode();
//!
//! assert!(msg.add_field(&mut field).is_ok())
//! ```
//!
//! ### Sending a message
//!
//! ```no_run
//! use tibrv::message::Msg;
//! use tibrv::field::Builder;
//! use tibrv::context::{RvCtx, TransportBuilder};
//!
//! let ctx = RvCtx::new().unwrap(); // Starts the Rendezvous internal machinery
//! let mut msg = Msg::new().unwrap();
//!
//! let data: u32 = 42;
//! let mut field = Builder::new(&data)
//!     .with_name("fieldName")
//!     .encode();
//!
//! assert!(msg.add_field(&mut field).is_ok()); // Copy the field into the message.
//! assert!(msg.set_send_subject("TEST.SUBJECT").is_ok()); // Set the send subject.
//!
//! let tp = TransportBuilder::new(&ctx).create().unwrap(); // Create a default Rendezvous transport.
//!
//! assert!(tp.send(&mut msg).is_ok());
//! ```
//!
//! ### Receiving a message
//!
//! ```no_run
//! use tibrv::context::{RvCtx, TransportBuilder};
//! use tibrv::event::Queue;
//!
//! let ctx = RvCtx::new().unwrap(); // Starts the Rendezvous internal machinery
//! let tp = TransportBuilder::new(&ctx).create().unwrap(); // Create a default Rendezvous transport.
//!
//! let queue = Queue::new(&ctx).unwrap(); // Create a new event queue.
//! let subscription = queue.subscribe(&tp, "TEST.SUBJECT").unwrap(); // Subscribe to a Rendezvous subject on this event queue.
//!
//! let msg = subscription.next().unwrap(); // Block, waiting for the next message to arrive on the subscribed subject.
//! ```

extern crate tibrv_sys;
extern crate chrono;
extern crate failure;
#[macro_use] extern crate failure_derive;

#[cfg(feature = "tokio")]
extern crate mio;
#[cfg(feature = "tokio")]
extern crate futures;
#[cfg(feature = "tokio")]
#[macro_use]
extern crate tokio_core;

#[macro_use]
pub mod errors;

pub mod field;
pub mod message;
pub mod context;
pub mod event;
#[cfg(feature = "tokio")]
pub mod async;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}

    // Both the send and recv tests require a running RVD on the
    // default port on localhost.
    #[test]
    #[ignore]
    fn send_msg() {
        use std::ffi::CString;
        use context::{RvCtx, TransportBuilder};
        use message::Msg;
        use field::Builder;

        let ctx = RvCtx::new().unwrap();
        let mut msg = Msg::new().unwrap();

        let data = CString::new("Hello, world!").unwrap();
        let cstr = data.as_c_str();
        let mut field = Builder::new(&cstr)
            .with_name("String")
            .encode();
        assert!(msg.add_field(&mut field)
                .is_ok());

        assert!(msg.set_send_subject("DUMMY").is_ok());

        let tp = TransportBuilder::new(&ctx).create().unwrap(); // Create default transport

        assert!(tp.send(&mut msg).is_ok());

    }

    // Receive test requires that something be sending messages to the
    // "TEST" subject with the "DATA" field populated with a string.
    // You can use the `tibrvsend` binary in the Rendezvous distribution
    // to accomplish this.
    //#[test]
    //#[ignore]
    //fn recv_msg() {
    //    use event::Queue;
    //    use context::{RvCtx, TransportBuilder};
    //    use field::Decodable;
    //    use std::ffi::CStr;

    //    let ctx = RvCtx::new().expect("Couldn't create RV machinery");

    //    let tp = TransportBuilder::new(&ctx).create().expect("Couldn't create transport");
    //    let q = Queue::new(&ctx).expect("Couldn't create queue");
    //    let sub = q.subscribe(&tp, "TEST").expect("Couldn't register subscription");
    //    let msg = sub.next().expect("Couldn't get next message.");
    //    let field = msg.get_field_by_name("DATA").expect("Couldn't find DATA Field");
    //    let data = <&CStr>::tibrv_try_decode(&field);
    //    assert!(data.is_ok());
    //}
}
