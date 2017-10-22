extern crate tibrv_sys;
extern crate chrono;

pub mod field;
pub mod message;
pub mod context;
pub mod event;

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
        use context;
        use message::Msg;
        use field::Builder;

        let ctx = context::RvCtx::new().unwrap();
        let mut msg = Msg::new().unwrap();

        let data = CString::new("Hello, world!").unwrap();
        let cstr = data.as_c_str();
        let mut field = Builder::new(&cstr)
            .with_name("String")
            .encode();
        assert!(msg.add_field(&mut field)
                .is_ok());

        msg.set_send_subject("TEST");

        let tp = ctx.transport().create().unwrap(); // Create default transport

        tp.send(&mut msg);

    }

    // Receive test requires that something be sending messages to the
    // "TEST" subject with the "DATA" field populated with a string.
    // You can use the `tibrvsend` binary in the Rendezvous distribution
    // to accomplish this.
    #[test]
    #[ignore]
    fn recv_msg() {
        use context;
        use field::Decodable;
        use std::ffi::CStr;

        let ctx = context::RvCtx::new().expect("Couldn't create RV machinery");

        let tp = ctx.transport().create().expect("Couldn't create transport");
        let q = ctx.queue().expect("Couldn't create queue");
        let sub = q.subscribe(&tp, "TEST").expect("Couldn't register subscription");
        let msg = sub.next().expect("Couldn't get next message.");
        let field = msg.get_field_by_name("DATA").expect("Couldn't find DATA Field");
        let data = <&CStr>::tibrv_try_decode(&field);
        assert!(data.is_ok());
    }
}
