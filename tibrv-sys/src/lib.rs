#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unreachable_patterns)]

extern crate chrono;

use chrono::NaiveDateTime;
use std::fmt;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

impl From<bool> for tibrv_bool {
    fn from(boolean: bool) -> Self {
        match boolean {
            true => tibrv_bool::TIBRV_TRUE,
            false => tibrv_bool::TIBRV_FALSE,
        }
    }
}

impl From<tibrv_bool> for bool {
    fn from(boolean: tibrv_bool) -> Self {
        match boolean {
            tibrv_bool::TIBRV_TRUE => true,
            tibrv_bool::TIBRV_FALSE => false,
        }
    }
}

impl From<NaiveDateTime> for tibrvMsgDateTime {
    fn from(dt: NaiveDateTime) -> Self {
        tibrvMsgDateTime {
            sec: dt.timestamp() as tibrv_i64,
            nsec: dt.timestamp_subsec_nanos() as tibrv_u32,
        }
    }
}

impl Into<NaiveDateTime> for tibrvMsgDateTime {
    fn into(self) -> NaiveDateTime {
        NaiveDateTime::from_timestamp(self.sec, self.nsec)
    }
}

// Generated externally from bindings.rs
impl fmt::Display for tibrv_status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            tibrv_status::TIBRV_OK => write!(f, "TIBRV_OK"),
            tibrv_status::TIBRV_INIT_FAILURE => write!(f, "TIBRV_INIT_FAILURE"),
            tibrv_status::TIBRV_INVALID_TRANSPORT => write!(f, "TIBRV_INVALID_TRANSPORT"),
            tibrv_status::TIBRV_INVALID_ARG => write!(f, "TIBRV_INVALID_ARG"),
            tibrv_status::TIBRV_NOT_INITIALIZED => write!(f, "TIBRV_NOT_INITIALIZED"),
            tibrv_status::TIBRV_ARG_CONFLICT => write!(f, "TIBRV_ARG_CONFLICT"),
            tibrv_status::TIBRV_SERVICE_NOT_FOUND => write!(f, "TIBRV_SERVICE_NOT_FOUND"),
            tibrv_status::TIBRV_NETWORK_NOT_FOUND => write!(f, "TIBRV_NETWORK_NOT_FOUND"),
            tibrv_status::TIBRV_DAEMON_NOT_FOUND => write!(f, "TIBRV_DAEMON_NOT_FOUND"),
            tibrv_status::TIBRV_NO_MEMORY => write!(f, "TIBRV_NO_MEMORY"),
            tibrv_status::TIBRV_INVALID_SUBJECT => write!(f, "TIBRV_INVALID_SUBJECT"),
            tibrv_status::TIBRV_DAEMON_NOT_CONNECTED => write!(f, "TIBRV_DAEMON_NOT_CONNECTED"),
            tibrv_status::TIBRV_VERSION_MISMATCH => write!(f, "TIBRV_VERSION_MISMATCH"),
            tibrv_status::TIBRV_SUBJECT_COLLISION => write!(f, "TIBRV_SUBJECT_COLLISION"),
            tibrv_status::TIBRV_VC_NOT_CONNECTED => write!(f, "TIBRV_VC_NOT_CONNECTED"),
            tibrv_status::TIBRV_NOT_PERMITTED => write!(f, "TIBRV_NOT_PERMITTED"),
            tibrv_status::TIBRV_INVALID_NAME => write!(f, "TIBRV_INVALID_NAME"),
            tibrv_status::TIBRV_INVALID_TYPE => write!(f, "TIBRV_INVALID_TYPE"),
            tibrv_status::TIBRV_INVALID_SIZE => write!(f, "TIBRV_INVALID_SIZE"),
            tibrv_status::TIBRV_INVALID_COUNT => write!(f, "TIBRV_INVALID_COUNT"),
            tibrv_status::TIBRV_NOT_FOUND => write!(f, "TIBRV_NOT_FOUND"),
            tibrv_status::TIBRV_ID_IN_USE => write!(f, "TIBRV_ID_IN_USE"),
            tibrv_status::TIBRV_ID_CONFLICT => write!(f, "TIBRV_ID_CONFLICT"),
            tibrv_status::TIBRV_CONVERSION_FAILED => write!(f, "TIBRV_CONVERSION_FAILED"),
            tibrv_status::TIBRV_RESERVED_HANDLER => write!(f, "TIBRV_RESERVED_HANDLER"),
            tibrv_status::TIBRV_ENCODER_FAILED => write!(f, "TIBRV_ENCODER_FAILED"),
            tibrv_status::TIBRV_DECODER_FAILED => write!(f, "TIBRV_DECODER_FAILED"),
            tibrv_status::TIBRV_INVALID_MSG => write!(f, "TIBRV_INVALID_MSG"),
            tibrv_status::TIBRV_INVALID_FIELD => write!(f, "TIBRV_INVALID_FIELD"),
            tibrv_status::TIBRV_INVALID_INSTANCE => write!(f, "TIBRV_INVALID_INSTANCE"),
            tibrv_status::TIBRV_CORRUPT_MSG => write!(f, "TIBRV_CORRUPT_MSG"),
            tibrv_status::TIBRV_ENCODING_MISMATCH => write!(f, "TIBRV_ENCODING_MISMATCH"),
            tibrv_status::TIBRV_TIMEOUT => write!(f, "TIBRV_TIMEOUT"),
            tibrv_status::TIBRV_INTR => write!(f, "TIBRV_INTR"),
            tibrv_status::TIBRV_INVALID_DISPATCHABLE => write!(f, "TIBRV_INVALID_DISPATCHABLE"),
            tibrv_status::TIBRV_INVALID_DISPATCHER => write!(f, "TIBRV_INVALID_DISPATCHER"),
            tibrv_status::TIBRV_INVALID_EVENT => write!(f, "TIBRV_INVALID_EVENT"),
            tibrv_status::TIBRV_INVALID_CALLBACK => write!(f, "TIBRV_INVALID_CALLBACK"),
            tibrv_status::TIBRV_INVALID_QUEUE => write!(f, "TIBRV_INVALID_QUEUE"),
            tibrv_status::TIBRV_INVALID_QUEUE_GROUP => write!(f, "TIBRV_INVALID_QUEUE_GROUP"),
            tibrv_status::TIBRV_INVALID_TIME_INTERVAL => write!(f, "TIBRV_INVALID_TIME_INTERVAL"),
            tibrv_status::TIBRV_INVALID_IO_SOURCE => write!(f, "TIBRV_INVALID_IO_SOURCE"),
            tibrv_status::TIBRV_INVALID_IO_CONDITION => write!(f, "TIBRV_INVALID_IO_CONDITION"),
            tibrv_status::TIBRV_SOCKET_LIMIT => write!(f, "TIBRV_SOCKET_LIMIT"),
            tibrv_status::TIBRV_OS_ERROR => write!(f, "TIBRV_OS_ERROR"),
            tibrv_status::TIBRV_INSUFFICIENT_BUFFER => write!(f, "TIBRV_INSUFFICIENT_BUFFER"),
            tibrv_status::TIBRV_EOF => write!(f, "TIBRV_EOF"),
            tibrv_status::TIBRV_INVALID_FILE => write!(f, "TIBRV_INVALID_FILE"),
            tibrv_status::TIBRV_FILE_NOT_FOUND => write!(f, "TIBRV_FILE_NOT_FOUND"),
            tibrv_status::TIBRV_IO_FAILED => write!(f, "TIBRV_IO_FAILED"),
            tibrv_status::TIBRV_NOT_FILE_OWNER => write!(f, "TIBRV_NOT_FILE_OWNER"),
            tibrv_status::TIBRV_USERPASS_MISMATCH => write!(f, "TIBRV_USERPASS_MISMATCH"),
            tibrv_status::TIBRV_TOO_MANY_NEIGHBORS => write!(f, "TIBRV_TOO_MANY_NEIGHBORS"),
            tibrv_status::TIBRV_ALREADY_EXISTS => write!(f, "TIBRV_ALREADY_EXISTS"),
            tibrv_status::TIBRV_PORT_BUSY => write!(f, "TIBRV_PORT_BUSY"),
            tibrv_status::TIBRV_DELIVERY_FAILED => write!(f, "TIBRV_DELIVERY_FAILED"),
            #[cfg(feature = "tibrv_8_3")]
            tibrv_status::TIBRV_QUEUE_LIMIT => write!(f, "TIBRV_QUEUE_LIMIT"),
            #[cfg(feature = "tibrv_8_3")]
            tibrv_status::TIBRV_INVALID_CONTENT_DESC => write!(f, "TIBRV_INVALID_CONTENT_DESC"),
            #[cfg(feature = "tibrv_8_3")]
            tibrv_status::TIBRV_INVALID_SERIALIZED_BUFFER => write!(f, "TIBRV_INVALID_SERIALIZED_BUFFER"),
            #[cfg(feature = "tibrv_8_3")]
            tibrv_status::TIBRV_DESCRIPTOR_NOT_FOUND => write!(f, "TIBRV_DESCRIPTOR_NOT_FOUND"),
            #[cfg(feature = "tibrv_8_3")]
            tibrv_status::TIBRV_CORRUPT_SERIALIZED_BUFFER => write!(f, "TIBRV_CORRUPT_SERIALIZED_BUFFER"),
            #[cfg(feature = "tibrv_8_3")]
            tibrv_status::TIBRV_IPM_ONLY => write!(f, "TIBRV_IPM_ONLY"),
            _ => write!(f, "TIBRV_UNSUPPORTED_STATUS_CODE"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;
    use std::os::raw::*;
    use std::ffi::{CString, CStr};

    macro_rules! tibok {
        ( $( $x: expr ),* ) => {
            $(
                assert_eq!($x, tibrv_status::TIBRV_OK);
            )*
        };
    }

    #[cfg(feature = "tibrv_8_3")]
    #[test]
    fn test_is_ipm() {
        unsafe {
            let is_ipm = tibrv_IsIPM();
            assert_eq!(is_ipm, tibrv_bool::TIBRV_FALSE);
        }
    }

    #[test]
    fn get_version() {
        use std::ffi::CStr;
        unsafe {
            let version = CStr::from_ptr(tibrv_Version());
            assert!(version.to_str().is_ok());
        }
    }

    #[test]
    fn create_message() {
        unsafe {
            let mut message: tibrvMsg = mem::zeroed();
            tibok!(tibrvMsg_Create(&mut message as *mut _));
        }
    }

    #[test]
    fn roundtrip_convert_str_message() {
        let text = CString::new("Hello World!").unwrap();
        let name = CString::new("message").unwrap();
        unsafe {
            let mut message: tibrvMsg = mem::zeroed();
            tibok!(tibrvMsg_Create(&mut message as *mut _),
                   tibrvMsg_AddStringEx(message, name.as_ptr(), text.as_ptr(), 0 as tibrv_u16));
            let mut returned_ptr: *const c_char = mem::zeroed();
            tibok!(tibrvMsg_GetStringEx(message, name.as_ptr(), &mut returned_ptr, 0 as tibrv_u16));
            let slice = CStr::from_ptr(returned_ptr);
            assert_eq!("Hello World!", slice.to_str().unwrap());
        }
    }

    #[test]
    fn convert_to_string() {
        let text = CString::new("Hello World!").unwrap();
        let name = CString::new("message").unwrap();
        unsafe {
            let mut message: tibrvMsg = mem::zeroed();
            tibok!(tibrvMsg_Create(&mut message as *mut _),
                   tibrvMsg_AddStringEx(message, name.as_ptr(), text.as_ptr(), 0 as tibrv_u16));
            let mut returned_ptr: *const c_char = mem::zeroed();
            tibok!(tibrvMsg_ConvertToString(message, &mut returned_ptr));
            let slice = CStr::from_ptr(returned_ptr);
            assert_eq!("{message=\"Hello World!\"}", slice.to_str().unwrap());
        }
    }
}
