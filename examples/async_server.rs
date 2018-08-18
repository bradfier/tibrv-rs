extern crate futures;
extern crate tibrv;
extern crate tokio;

use tibrv::context::{RvCtx, TransportBuilder};

use futures::prelude::Stream;
use std::ffi::CString;
use tibrv::field::Builder;
use tokio::prelude::*;
use tokio::reactor::Handle;

/// This example shows how to subscribe to incoming messages using
/// a Futures `Stream`, chain some work on each incoming message,
/// then forward some response (in this case just echoing the message)
/// back via a `Sink`, implemented by the `Transport`.
fn main() {
    let handle = Handle::current(); // Get a handle to the current reactor
    let ctx = RvCtx::new().expect("Couldn't start tibrv context");
    let tp = TransportBuilder::new(ctx.clone())
        .create()
        .expect("Couldn't create default transport.");

    // Set up the incoming event stream
    let events =
        tp.async_serve(&handle, "TEST.SUBJECT", |mut msg| {
            let data = CString::new("Reply!").unwrap();
            let mut field = Builder::new(&data.as_c_str()).with_name("reply").encode();
            let _ = msg.add_field(&mut field).unwrap();
            Ok(msg)
        }).then(|_| Ok(()));

    // Finally, run the event loop.
    tokio::run(events);
}
