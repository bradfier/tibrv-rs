extern crate tibrv_sys;
extern crate chrono;

pub mod field;
pub mod message;
pub mod context;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}

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
}
