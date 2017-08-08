#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

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
            tibok!(
                tibrvMsg_Create(&mut message as *mut _),
                tibrvMsg_AddStringEx(message,
                                     name.as_ptr(),
                                     text.as_ptr(),
                                     0 as tibrv_u16));
            let mut returned_ptr: *const c_char = mem::zeroed();
            tibok!(tibrvMsg_GetStringEx(message,
                                        name.as_ptr(),
                                        &mut returned_ptr,
                                        0 as tibrv_u16));
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
            tibok!(
                tibrvMsg_Create(&mut message as *mut _),
                tibrvMsg_AddStringEx(message,
                                     name.as_ptr(),
                                     text.as_ptr(),
                                     0 as tibrv_u16)
            );
            let mut returned_ptr: *const c_char = mem::zeroed();
            tibok!(tibrvMsg_ConvertToString(message, &mut returned_ptr));
            let slice = CStr::from_ptr(returned_ptr);
            assert_eq!("{message=\"Hello World!\"}", slice.to_str().unwrap());
        }
    }
}
