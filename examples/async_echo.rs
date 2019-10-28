extern crate futures;
extern crate tibrv;
extern crate tokio;

use tibrv::context::{RvCtx, TransportBuilder};

use futures::prelude::Stream;
use tokio::prelude::*;
use tokio::reactor::Handle;

/// This example shows how to subscribe to incoming messages using
/// a Futures `Stream`, chain some work on each incoming message,
/// then forward some response (in this case just echoing the message)
/// back via a `Sink`, implemented by the `Transport`.
fn main() {
    let handle = Handle::default(); // Get a handle to the default reactor
    let ctx = RvCtx::new().expect("Couldn't start tibrv context");
    let tp = TransportBuilder::new(ctx.clone())
        .create()
        .expect("Couldn't create default transport.");

    // Set up the incoming event stream
    let incoming = tp.async_sub(&handle, "TEST").unwrap();

    let events = incoming
        .and_then(|mut msg| {
            // and_then applies some function to each element of the stream, passing the stream onward
            msg.set_send_subject("ECHO").unwrap();
            // Do some useful work...
            Ok(msg)
        })
        .forward(tp)
        .then(|_| Ok(())); // Forward the `Stream` of futures to this `Sink`, in our case the default transport.

    // Finally, run the event loop.
    tokio::run(events);
}
