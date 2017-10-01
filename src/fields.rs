use tibrv_sys::*;
use std::ffi::CString;
use chrono::NaiveDateTime;
use std;

pub struct TibRVMsgField {
    pub name: CString,
    pub inner: tibrvMsgField,
}

pub trait Encodable {
    fn tibrv_encode(&self, Option<&str>, Option<u32>) -> TibRVMsgField;
}

macro_rules! fixed_width_encodable {
    ($base_type:ty, $tibrv_type:ty, $local:ident, $tibrv_flag:expr) => (
        impl Encodable for $base_type {
            fn tibrv_encode(&self, name: Option<&str>, id: Option<u32>) -> TibRVMsgField {
                let name_cstr = CString::new(name.unwrap_or("")).unwrap();
                let ptr = match name_cstr.to_bytes().len() {
                    0 => std::ptr::null(),
                    _ => name_cstr.as_ptr(),
                };

                TibRVMsgField {
                    name: name_cstr,
                    inner: tibrvMsgField {
                        name: ptr,
                        size: std::mem::size_of::<$base_type>() as tibrv_u32,
                        count: 1 as tibrv_u32,
                        data: tibrvLocalData { $local: self.clone() as $tibrv_type },
                        id: id.unwrap_or(0) as tibrv_u16,
                        type_: $tibrv_flag as tibrv_u8,
                    }
                }
            }
        }
    )
}

macro_rules! from_encodable {
    ($base_type:ty, $tibrv_type:tt, $local:ident, $tibrv_flag:expr) => (
        impl Encodable for $base_type {
            fn tibrv_encode(&self, name: Option<&str>, id: Option<u32>) -> TibRVMsgField {
                let name_cstr = CString::new(name.unwrap_or("")).unwrap();
                let ptr = match name_cstr.to_bytes().len() {
                    0 => std::ptr::null(),
                    _ => name_cstr.as_ptr(),
                };
                TibRVMsgField {
                    name: name_cstr,
                    inner: tibrvMsgField {
                        name: ptr,
                        size: std::mem::size_of::<$tibrv_type>() as tibrv_u32,
                        count: 1 as tibrv_u32,
                        data: tibrvLocalData { $local: $tibrv_type::from(self.clone()) },
                        id: id.unwrap_or(0) as tibrv_u16,
                        type_: $tibrv_flag as tibrv_u8,
                    }
                }
            }
        }
    )
}

// Integers
fixed_width_encodable!(u8, tibrv_u8, u8, TIBRVMSG_U8);
fixed_width_encodable!(i8, tibrv_i8, i8, TIBRVMSG_I8);
fixed_width_encodable!(u16, tibrv_u16, u16, TIBRVMSG_U16);
fixed_width_encodable!(i16, tibrv_i16, i16, TIBRVMSG_I16);
fixed_width_encodable!(u32, tibrv_u32, u32, TIBRVMSG_U32);
fixed_width_encodable!(i32, tibrv_i32, i32, TIBRVMSG_I32);
fixed_width_encodable!(u64, tibrv_u64, u64, TIBRVMSG_U64);
fixed_width_encodable!(i64, tibrv_i64, i64, TIBRVMSG_I64);

// Floating point
fixed_width_encodable!(f32, tibrv_f32, f32, TIBRVMSG_F32);
fixed_width_encodable!(f64, tibrv_f64, f64, TIBRVMSG_F64);

from_encodable!(bool, tibrv_bool, boolean, TIBRVMSG_BOOL);
from_encodable!(NaiveDateTime, tibrvMsgDateTime, date, TIBRVMSG_DATETIME);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_width_encode() {
        let name = "Fixed Width Scalar";
        let unsigned8: u8 = 0;
        let unsigned16: u16 = 0;
        let unsigned32: u32 = 0;
        let unsigned64: u64 = 0;
        assert_eq!(unsigned8.tibrv_encode(Some(name), Some(0)).inner.size, 1);
        assert_eq!(unsigned16.tibrv_encode(Some(name), Some(0)).inner.size, 2);
        assert_eq!(unsigned32.tibrv_encode(Some(name), Some(0)).inner.size, 4);
        assert_eq!(unsigned64.tibrv_encode(Some(name), Some(0)).inner.size, 8);
    }

    #[test]
    fn test_datetime_encode() {
        use chrono::prelude::*;
        let dt = NaiveDate::from_ymd(1970, 1, 1).and_hms_milli(1, 0, 0, 0);
        unsafe {
            assert_eq!(dt.tibrv_encode(Some("DateTime"), Some(0)).inner.data.date.sec, 3600);
        }
    }

}
