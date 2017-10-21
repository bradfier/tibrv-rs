//! Interfaces for constructing Rendezvous Message Fields

use tibrv_sys::*;
use message::{Msg, BorrowedMsg};
use std::ffi::{CStr, CString};
use chrono::NaiveDateTime;
use std::net::Ipv4Addr;
use std::os::raw::c_void;
use std::ops::Deref;
use std::marker::PhantomData;
use std;

/// A structure wrapping a `tibrvMsgField`
pub struct MsgField {
    pub name: CString,
    pub inner: tibrvMsgField,
}

/// A structure wrapping a `MsgField`
///
/// `BorrowedMsgField` encodes the lifetime that an extracted `MsgField`
/// inherits from its parent `Msg`, using `std::marker::PhantomData`.
///
/// This type implements `Deref<Target = MsgField>`, so any methods
/// which take `&MsgField` (i.e the 'decode' methods) can be used
/// transparently.
pub struct BorrowedMsgField<'a> {
    pub inner: MsgField,
    pub phantom: PhantomData<&'a Msg>,
}

impl<'a> Deref for BorrowedMsgField<'a> {
    type Target = MsgField;
    fn deref(&self) -> &MsgField {
        &self.inner
    }
}

/// A builder for `MsgField`.
pub struct Builder<'a, T: 'a>
    where T: Encodable
{
    name: Option<&'a str>,
    id: Option<u32>,
    data: &'a T,
}

impl<'a, T> Builder<'a, T>
    where T: Encodable
{
    /// Creates a new `Builder` used to construct a `MsgField`.o
    ///
    /// At least one of `with_name` or `with_id` must be called
    /// to supply identifier information for the `MsgField`.
    pub fn new(data: &'a T) -> Builder<'a, T> {
        Builder {
            name: None,
            id: None,
            data: data,
        }
    }

    /// Sets the `MsgField` name.
    pub fn with_name(mut self, name: &'a str) -> Builder<T> {
        self.name = Some(name);
        self
    }

    /// Sets the `MsgField` id.
    pub fn with_id(mut self, id: u32) -> Builder<'a, T> {
        self.id = Some(id);
        self
    }

    /// Consumes the `Builder`, creating a `MsgField`.
    pub fn encode(self) -> MsgField {
        self.data.tibrv_encode(self.name, self.id)
    }
}

/// Trait indicating the type may be encoded into a message field.
///
/// Implementations are provided for all the scalar data types supported
/// by Rendezvous, these scalar types may in turn also be encoded as native
/// arrays.
///
/// Also supported are strings (as `&CStr`), IPv4 Addresses (`std::net::Ipv4Addr`)
/// and date/time, using `NaiveDateTime` from the `chrono` crate.
///
/// Used along with the Decodable trait, these methods allow seamless conversion
/// to and from Rendezvous data structures.
///
/// ### Example
///
/// ```
/// use tibrv::field::{Encodable, Decodable};
///
/// let array: &[u8] = &[1, 2, 3, 4];
///
/// // Encoding
/// let msg = array.tibrv_encode(Some("Array"), None);
/// assert_eq!(1, msg.inner.size);
/// assert_eq!(4, msg.inner.count);
///
/// // Decoding
/// let slice = <&[u8]>::tibrv_try_decode(&msg).unwrap();
/// assert_eq!([1, 2, 3, 4], slice);
/// ```
pub trait Encodable {
    /// Encodes this variable as a message field.
    ///
    /// Scalar types will be copied, but all vector types will be
    /// passed as pointers to the underlying C struct, so the source variable
    /// must live until the message field is added to a TibRVMsg
    ///
    /// ### Arguments
    /// At least one of `name` or `id` must be `Some()`
    fn tibrv_encode(&self, name: Option<&str>, id: Option<u32>) -> MsgField;
}

/// Trait indicating the type may be decoded from a message field.
pub trait Decodable {
    /// Try and decode a supplied `MsgField` as this type.
    ///
    /// Rendezvous message fields include some primitive type information,
    /// but this method may fail if the sending party incorrectly encodes
    /// the data fields.
    fn tibrv_try_decode(msg: &MsgField) -> Result<Self, &'static str>
        where Self: Sized;
}

macro_rules! some_ident {
    ($name:ident, $id:ident) => (
        assert!($name.is_some() || $id.is_some(),
                "At least one of id or name must be defined.");
    )
}

macro_rules! encodable {
    ($base_type:ty, $tibrv_type:tt, $local:ident, $tibrv_flag:expr) => (
        impl Encodable for $base_type {
            fn tibrv_encode(&self, name: Option<&str>, id: Option<u32>) -> MsgField {
                some_ident!(name, id);
                let name_cstr = CString::new(name.unwrap_or("")).unwrap();
                let ptr = name.map_or(std::ptr::null(), |_| name_cstr.as_ptr());
                MsgField {
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

        impl Decodable for $base_type {
            fn tibrv_try_decode(msg: &MsgField) -> Result<$base_type, &'static str> {
                if msg.inner.count > 1 { return Err("Attempt to decode array MsgField as Scalar") };
                if msg.inner.type_ == $tibrv_flag as u8 {
                    let val = unsafe { msg.inner.data.$local };
                    let decoded: $base_type = val.into();
                    Ok(decoded)
                } else {
                    Err("Mismatched message type flag")
                }
            }
        }
    )
}

macro_rules! array_encodable {
    ($base_type:ty, $tibrv_flag:expr) => (
        impl<'a> Encodable for &'a [$base_type] {
            fn tibrv_encode(&self, name: Option<&str>, id: Option<u32>) -> MsgField {
                some_ident!(name, id);
                let name_cstr = CString::new(name.unwrap_or("")).unwrap();
                let ptr = name.map_or(std::ptr::null(), |_| name_cstr.as_ptr());
                MsgField {
                    name: name_cstr,
                    inner: tibrvMsgField {
                        name: ptr,
                        size: std::mem::size_of::<$base_type>() as tibrv_u32,
                        count: self.len() as tibrv_u32,
                        data: tibrvLocalData { array: self.as_ptr() as *const c_void },
                        id: id.unwrap_or(0) as tibrv_u16,
                        type_: $tibrv_flag as tibrv_u8,
                    }
                }
            }
        }

        impl<'a> Decodable for &'a [$base_type] {
            fn tibrv_try_decode(msg: &MsgField) -> Result<&'a [$base_type], &'static str> {
                if msg.inner.type_ != $tibrv_flag as u8 {
                    Err("Mismatched message type flag")
                } else {
                    let buffer = unsafe { msg.inner.data.array };
                    let slice = unsafe { std::slice::from_raw_parts::<$base_type>(buffer as *const $base_type, msg.inner.count as usize) };
                    Ok(slice)
                }
            }

        }
    )
}

impl<'a> Encodable for &'a CStr {
    fn tibrv_encode(&self, name: Option<&str>, id: Option<u32>) -> MsgField {
        some_ident!(name, id);
        let name_cstr = CString::new(name.unwrap_or("")).unwrap();
        let ptr = name.map_or(std::ptr::null(), |_| name_cstr.as_ptr());
        MsgField {
            name: name_cstr,
            inner: tibrvMsgField {
                name: ptr,
                size: self.to_bytes_with_nul().len() as tibrv_u32,
                count: 1 as tibrv_u32,
                data: tibrvLocalData { str: self.as_ptr() },
                id: id.unwrap_or(0) as tibrv_u16,
                type_: TIBRVMSG_STRING as tibrv_u8,
            },
        }
    }
}

impl<'a> Decodable for &'a CStr {
    fn tibrv_try_decode(msg: &MsgField) -> Result<&'a CStr, &'static str> {
        if msg.inner.type_ != TIBRVMSG_STRING as u8 {
            Err("Mismatched message type flag")
        } else {
            let str_ptr = unsafe { msg.inner.data.str };
            let c_str = unsafe { CStr::from_ptr(str_ptr) };
            Ok(c_str)
        }
    }
}

// You can encode an owned Msg but decoding produces a BorrowedMsg
impl<'a> Encodable for &'a Msg {
    fn tibrv_encode(&self, name: Option<&str>, id: Option<u32>) -> MsgField {
        some_ident!(name, id);
        let name_cstr = CString::new(name.unwrap_or("")).unwrap();
        let ptr = name.map_or(std::ptr::null(), |_| name_cstr.as_ptr());
        MsgField {
            name: name_cstr,
            inner: tibrvMsgField {
                name: ptr,
                size: self.byte_size().unwrap() as tibrv_u32,
                count: 1 as tibrv_u32,
                data: tibrvLocalData { msg: self.inner },
                id: id.unwrap_or(0) as tibrv_u16,
                type_: TIBRVMSG_MSG as tibrv_u8,
            },
        }
    }
}

impl Decodable for BorrowedMsg {
    fn tibrv_try_decode(msg: &MsgField) -> Result<Self, &'static str> {
        if msg.inner.type_ != TIBRVMSG_MSG as u8 {
            Err("Mismatched message type flag.")
        } else {
            let ptr = unsafe { msg.inner.data.msg };
            Ok(BorrowedMsg { inner: ptr })
        }
    }
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

array_encodable!(u8, TIBRVMSG_U8ARRAY);
array_encodable!(i8, TIBRVMSG_I8ARRAY);
array_encodable!(u16, TIBRVMSG_U16ARRAY);
array_encodable!(i16, TIBRVMSG_I16ARRAY);
array_encodable!(u32, TIBRVMSG_U32ARRAY);
array_encodable!(i32, TIBRVMSG_I32ARRAY);
array_encodable!(u64, TIBRVMSG_U64ARRAY);
array_encodable!(i64, TIBRVMSG_I64ARRAY);

// Floating point
encodable!(f32, tibrv_f32, f32, TIBRVMSG_F32);
encodable!(f64, tibrv_f64, f64, TIBRVMSG_F64);
array_encodable!(f32, TIBRVMSG_F32ARRAY);
array_encodable!(f64, TIBRVMSG_F64ARRAY);

// Custom types
encodable!(bool, tibrv_bool, boolean, TIBRVMSG_BOOL);
encodable!(NaiveDateTime, tibrvMsgDateTime, date, TIBRVMSG_DATETIME);
encodable!(Ipv4Addr, tibrv_ipaddr32, ipaddr32, TIBRVMSG_IPADDR32);

/// Encode a `u16` as an IP Port message field.
///
/// Rendezvous has special provisions for network data types,
/// IPv4 Addresses and IP Ports are encoded as special types using
/// Network Byte Ordering.
///
/// Since we already provide an Impl for 'normal' `u16` this function will
/// encode using the special byte ordering.
pub fn tibrv_encode_port(port: &u16,
                         name: Option<&str>,
                         id: Option<u32>)
                         -> MsgField {
    some_ident!(name, id);
    let name_cstr = CString::new(name.unwrap_or("")).unwrap();
    let ptr = name.map_or(std::ptr::null(), |_| name_cstr.as_ptr());

    MsgField {
        name: name_cstr,
        inner: tibrvMsgField {
            name: ptr,
            size: std::mem::size_of::<u16>() as tibrv_u32,
            count: 1 as tibrv_u32,
            data: tibrvLocalData { ipport16: port.to_be() },
            id: id.unwrap_or(0) as tibrv_u16,
            type_: TIBRVMSG_IPPORT16 as tibrv_u8,
        },
    }
}

/// Try and decode an IP Port message field.
pub fn tibrv_try_decode_port(msg: &MsgField) -> Result<u16, &'static str> {
    if msg.inner.count > 1 {
        panic!("Attempt to decode array MsgField as Scalar");
    }
    if msg.inner.type_ == TIBRVMSG_IPPORT16 as u8 {
        let val = unsafe { msg.inner.data.ipport16 };
        let decoded = u16::from_be(val);
        Ok(decoded)
    } else {
        Err("Mismatched message type flag")
    }
}

/// Encode a slice as an opaque byte sequence.
pub unsafe fn tibrv_encode_opaque<'a, T: Copy>(slice: &'a [T],
                                               name: Option<&str>,
                                               id: Option<u32>)
                                               -> MsgField {
    some_ident!(name, id);
    let name_cstr = CString::new(name.unwrap_or("")).unwrap();
    let ptr = name.map_or(std::ptr::null(), |_| name_cstr.as_ptr());
    MsgField {
        name: name_cstr,
        inner: tibrvMsgField {
            name: ptr,
            size: std::mem::size_of_val(&slice) as tibrv_u32,
            count: 1 as tibrv_u32,
            data: tibrvLocalData { buf: slice.as_ptr() as *const c_void },
            id: id.unwrap_or(0) as tibrv_u16,
            type_: TIBRVMSG_OPAQUE as tibrv_u8,
        },
    }
}

/// Try and decode an opaque byte sequence into a slice
///
/// When using Rendezvous opaque byte sequences, all type information
/// is lost (internally the slice is passed as `void*`).
/// Therefore this function is approximately as unsafe as `std::mem::transmute`,
/// except without the soft and fluffy blanket of size checking.
pub unsafe fn tibrv_try_decode_opaque<T: Copy>
    (msg: &MsgField)
     -> Result<&[T], &'static str> {
    if msg.inner.type_ != TIBRVMSG_OPAQUE as u8 {
        Err("Mismatched message type flag")
    } else {
        assert!(!msg.inner.data.buf.is_null());
        // `size` in from_raw_parts is helpfully in 'elements' not bytes...
        let elements: usize = msg.inner.size as usize /
                              std::mem::size_of::<T>();
        Ok(std::slice::from_raw_parts(msg.inner.data.buf as *const T, elements))
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
            assert_eq!(dt.tibrv_encode(Some("DateTime"), Some(0))
                           .inner
                           .data
                           .date
                           .sec,
                       3600);
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
    fn u8_array() {
        let array: &[u8] = &[1, 2, 3, 4];
        let msg = array.tibrv_encode(Some("Array"), None);
        assert_eq!(1, msg.inner.size);
        assert_eq!(4, msg.inner.count);

        let slice = <&[u8]>::tibrv_try_decode(&msg).unwrap();
        assert_eq!(1, slice[0]);
        assert_eq!(2, slice[1]);
        assert_eq!(3, slice[2]);
        assert_eq!(4, slice[3]);
    }

    #[test]
    fn string_conversion() {
        let name = "Name";
        let sample_string = "Hello world!";
        let other = CString::new(sample_string).unwrap();
        let tib_string = other.as_ref().tibrv_encode(Some(name), None);

        assert_eq!(sample_string.len() + 1, tib_string.inner.size as usize);
        let decoded = <&CStr>::tibrv_try_decode(&tib_string)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(sample_string, decoded);
    }

    #[test]
    fn builder() {
        let data: &[u64] = &[1, 2, 3, 4, 5];
        let field = Builder::new(&data)
            .with_name("Name")
            .with_id(4)
            .encode();
        assert_eq!(8, field.inner.size);
        assert_eq!(5, field.inner.count);
    }

    #[test]
    #[should_panic]
    fn id_and_name_none() {
        let unsigned64: u64 = 0;
        let _ = unsigned64.tibrv_encode(None, None);
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
