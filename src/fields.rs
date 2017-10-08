use tibrv_sys::*;
use std::ffi::CString;
use chrono::NaiveDateTime;
use std::net::Ipv4Addr;
use std;

pub struct TibRVMsgField {
    pub name: CString,
    pub inner: tibrvMsgField,
}

pub trait Encodable {
    fn tibrv_encode(&self, Option<&str>, Option<u32>) -> TibRVMsgField;

    fn tibrv_try_decode(&TibRVMsgField) -> Option<Self>
        where Self: Sized;
}

macro_rules! encodable {
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

            fn tibrv_try_decode(msg: &TibRVMsgField) -> Option<$base_type> {
                if msg.inner.type_ == $tibrv_flag as u8 {
                    let val = unsafe { msg.inner.data.$local };
                    let decoded: $base_type = val.into();
                    Some(decoded)
                } else {
                    None
                }
            }
        }
    )
}

// Integers
encodable!(u8, tibrv_u8, u8, TIBRVMSG_U8);
encodable!(i8, tibrv_i8, i8, TIBRVMSG_I8);
encodable!(u16, tibrv_u16, u16, TIBRVMSG_U16);
encodable!(i16, tibrv_i16, i16, TIBRVMSG_I16);
encodable!(u32, tibrv_u32, u32, TIBRVMSG_U32);
encodable!(i32, tibrv_i32, i32, TIBRVMSG_I32);
encodable!(u64, tibrv_u64, u64, TIBRVMSG_U64);
encodable!(i64, tibrv_i64, i64, TIBRVMSG_I64);

// Floating point
encodable!(f32, tibrv_f32, f32, TIBRVMSG_F32);
encodable!(f64, tibrv_f64, f64, TIBRVMSG_F64);

encodable!(bool, tibrv_bool, boolean, TIBRVMSG_BOOL);
encodable!(NaiveDateTime, tibrvMsgDateTime, date, TIBRVMSG_DATETIME);
encodable!(Ipv4Addr, tibrv_ipaddr32, ipaddr32, TIBRVMSG_IPADDR32);

// Special cases for u16 IP port encoded in Network Byte Order
fn tibrv_encode_port(port: &u16, name: Option<&str>, id: Option<u32>) -> TibRVMsgField {
    let name_cstr = CString::new(name.unwrap_or("")).unwrap();
    let ptr = match name_cstr.to_bytes().len() {
        0 => std::ptr::null(),
        _ => name_cstr.as_ptr(),
    };

    TibRVMsgField {
        name: name_cstr,
        inner: tibrvMsgField {
            name: ptr,
            size: std::mem::size_of::<u16>() as tibrv_u32,
            count: 1 as tibrv_u32,
            data: tibrvLocalData { ipport16: port.to_be() },
            id: id.unwrap_or(0) as tibrv_u16,
            type_: TIBRVMSG_IPPORT16 as tibrv_u8,
        }
    }
}

fn tibrv_try_decode_port(msg: &TibRVMsgField) -> Option<u16> {
    if msg.inner.type_ == TIBRVMSG_IPPORT16 as u8 {
        let val = unsafe { msg.inner.data.ipport16 };
        let decoded = u16::from_be(val);
        Some(decoded)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_width_encode_size() {
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
    fn primitive_roundtrip() {
        let name = "Primitive Roundtrip";
        let unsigned8: u8 = 0;
        let unsigned16: u16 = 0;
        let unsigned32: u32 = 0;
        let unsigned64: u64 = 0;
        let tib_u8 = unsigned8.tibrv_encode(Some(name), Some(0));
        let tib_u16 = unsigned16.tibrv_encode(Some(name), Some(0));
        let tib_u32 = unsigned32.tibrv_encode(Some(name), Some(0));
        let tib_u64 = unsigned64.tibrv_encode(Some(name), Some(0));
        assert_eq!(unsigned8, u8::tibrv_try_decode(&tib_u8).unwrap());
        assert_eq!(unsigned16, u16::tibrv_try_decode(&tib_u16).unwrap());
        assert_eq!(unsigned32, u32::tibrv_try_decode(&tib_u32).unwrap());
        assert_eq!(unsigned64, u64::tibrv_try_decode(&tib_u64).unwrap());
    }

    #[test]
    fn test_datetime_encode() {
        use chrono::prelude::*;
        let dt = NaiveDate::from_ymd(1970, 1, 1).and_hms_milli(1, 0, 0, 0);
        unsafe {
            assert_eq!(dt.tibrv_encode(Some("DateTime"), Some(0)).inner.data.date.sec, 3600);
        }
    }

    #[test]
    fn test_datetime_roundtrip() {
        use chrono::prelude::*;
        let dt = NaiveDate::from_ymd(2017, 01, 01).and_hms_milli(0, 0, 0, 0);
        let tibdate = dt.tibrv_encode(Some("Date"), None);
        assert_eq!(dt, NaiveDateTime::tibrv_try_decode(&tibdate).unwrap());
    }

    #[test]
    fn test_ipaddr_encode() {
        let addr = Ipv4Addr::new(127, 0, 0, 1);
        let tibaddr = addr.tibrv_encode(Some("IP Address"), None);
        assert_eq!(addr, Ipv4Addr::tibrv_try_decode(&tibaddr).unwrap());
    }

    #[test]
    fn test_ipport_encode() {
        let port = 1;
        let tibport = tibrv_encode_port(&port, Some("Port"), None);
        unsafe {
            assert_eq!(256, tibport.inner.data.ipport16);
        }
        assert_eq!(port, tibrv_try_decode_port(&tibport).unwrap());
    }

    #[test]
    #[should_panic]
    fn test_bogus_decode() {
        // Decoding into the wrong type should panic.
        // Technically this is ok if promoting integer types but that's
        // rather more validation than I feel like doing.
        let unsigned64: u64 = 0;
        let tib_u64 = unsigned64.tibrv_encode(Some("u64"), Some(0));
        assert_eq!(0, u32::tibrv_try_decode(&tib_u64).unwrap())
    }
}
