extern crate tibrv;

use std::ffi::CStr;

use tibrv::context::{RvCtx, TransportBuilder};
use tibrv::field::Decodable;
use tibrv::message::Msg;

fn main() {
    let ctx = RvCtx::new().unwrap(); // Create the context, starting Rendezvous internals
    let tp = TransportBuilder::new(ctx.clone())
        .create()
        .expect("Couldn't create default transport");

    let mut msg = Msg::new().unwrap();
    msg.set_send_subject("TEST.SUBJECT").unwrap();

    let response = tp.request(&mut msg, Some(10.0)).unwrap();

    let reply = response.get_field_by_name("reply").unwrap();
    let decoded = <&CStr>::tibrv_try_decode(&reply).unwrap();

    println!("{:?}", decoded);
}
