extern crate tibrv;

use std::ffi::CString;

use tibrv::context::{RvCtx, TransportBuilder};
use tibrv::field::Builder;

fn main() {
    let ctx = RvCtx::new().unwrap(); // Create the context, starting Rendezvous internals
    let tp = TransportBuilder::new(ctx.clone())
        .create()
        .expect("Couldn't create default transport");

    tp.serve("TEST.SUBJECT", |mut msg| {
        let data = CString::new("Reply!").unwrap();
        let mut field = Builder::new(&data.as_c_str()).with_name("reply").encode();
        let _ = msg.add_field(&mut field).unwrap();
        Ok(msg)
    }).unwrap();
}
