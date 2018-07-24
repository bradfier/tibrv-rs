extern crate futures;
extern crate tibrv;
extern crate tokio;

use std::ffi::CStr;

use tibrv::context::{RvCtx, TransportBuilder};
use tibrv::field::Decodable;
use tibrv::message::Msg;

use tokio::prelude::*;
use tokio::reactor::Handle;

fn main() {
    let handle = Handle::current();
    let ctx = RvCtx::new().unwrap(); // Create the context, starting Rendezvous internals
    let tp = TransportBuilder::new(ctx.clone())
        .create()
        .expect("Couldn't create default transport");

    let mut msg = Msg::new().unwrap();
    msg.set_send_subject("TEST.SUBJECT").unwrap();

    let response = tp.async_req(&handle, &mut msg).unwrap();

    let events = response.then(|msg| {
        let unwrapped = msg.unwrap();
        let reply = unwrapped.get_field_by_name("reply").unwrap();
        let decoded = <&CStr>::tibrv_try_decode(&reply).unwrap();

        println!("{:?}", decoded);
        Ok(())
    });

    tokio::run(events)
}
