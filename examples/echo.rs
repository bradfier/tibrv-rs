extern crate tibrv;

use tibrv::context::{RvCtx, TransportBuilder};
use tibrv::event::Queue;

/// This example shows how to subscribe to incoming messages
/// and echo them out again on a different Rendezvous subject.
fn main() {
    let ctx = RvCtx::new().unwrap(); // Create the context, starting Rendezvous internals
    let tp = TransportBuilder::new(&ctx).create().unwrap(); // Create a default transport
    let event_queue = Queue::new(&ctx).unwrap(); // Create an event queue for incoming messages

    // Subscribe to the inbound message subject:
    let subscription = event_queue.subscribe(&tp, "TEST.SUBJECT").unwrap();

    loop {
        let mut msg = subscription.next().unwrap(); // Block, waiting for the next message on this subscription.
        msg.set_send_subject("ECHO.SUBJECT").unwrap(); // Modify the send subject so we don't get an echo loop.
        tp.send(&mut msg).unwrap(); // Try and actually send the message using our transport.
    }
}
