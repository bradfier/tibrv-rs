extern crate tibrv;
extern crate futures;
extern crate tokio_core;

use tibrv::context::{RvCtx, TransportBuilder};
use tibrv::async::AsyncQueue;

use futures::prelude::Stream;
use tokio_core::reactor::Core;

/// This example shows how to subscribe to incoming messages using
/// a Futures `Stream`, chain some work on each incoming message,
/// then forward some response (in this case just echoing the message)
/// back via a `Sink`, implemented by the `Transport`.
fn main() {
    let mut core = Core::new().unwrap(); // Create a tokio event loop
    let ctx = RvCtx::new().expect("Couldn't start tibrv context");
    let tp = TransportBuilder::new(&ctx).create().expect("Couldn't create default transport.");

    let event_queue = AsyncQueue::new(&ctx).expect("Couldn't create event queue.");

    // Set up the incoming event stream
    let incoming = event_queue.subscribe(&core.handle(), &tp, "TEST").unwrap();

    let events = incoming.and_then(|mut msg| { // and_then applies some function to each element of the stream, passing the stream onward
        msg.set_send_subject("ECHO").unwrap();
        // Do some useful work...
        Ok(msg)
    }).forward(tp); // Forward the `Stream` of futures to this `Sink`, in our case the default transport.

    // Finally, run the event loop.
    core.run(events).unwrap();
}
