//! Interfaces for constructing Rendezvous Message Fields
//!
//! All data to be sent via a Rendezvous message must be tagged
//! and encapsulated in a "message field", represented here by
//! the `MsgField` type.

#![allow(clippy::float_cmp)]

use chrono::NaiveDateTime;
use errors::*;
use message::{BorrowedMsg, Msg};
use std;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::net::Ipv4Addr;
use std::ops::Deref;
use std::os::raw::c_void;
use tibrv_sys::*;

pub enum DecodedField<'a> {
    String(&'a CStr),
    Message(BorrowedMsg),
    U8(u8),
    U8Array(&'a [u8]),
    I8(i8),
    I8Array(&'a [i8]),
    U16(u16),
    U16Array(&'a [u16]),
    I16(i16),
    I16Array(&'a [i16]),
    U32(u32),
    U32Array(&'a [u32]),
    I32(i32),
    I32Array(&'a [i32]),
    U64(u64),
    U64Array(&'a [u64]),
    I64(i64),
    I64Array(&'a [i64]),
    F32(f32),
    F32Array(&'a [f32]),
    F64(f64),
    F64Array(&'a [f64]),
    Bool(bool),
    DateTime(NaiveDateTime),
    Ipv4(Ipv4Addr),
    IpPort(u16),
    Opaque(&'a [u8]),
}

impl<'a> Decodable<'a> for DecodedField<'a> {
    fn tibrv_try_decode(fld: &'a MsgField) -> Result<DecodedField<'a>, TibrvError> {
        match u32::from(fld.inner.type_) {
            TIBRVMSG_STRING => fld.try_decode().map(DecodedField::String),
            TIBRVMSG_MSG => fld.try_decode().map(DecodedField::Message),
            TIBRVMSG_U8 => fld.try_decode().map(DecodedField::U8),
            TIBRVMSG_U8ARRAY => fld.try_decode().map(DecodedField::U8Array),
            TIBRVMSG_I8 => fld.try_decode().map(DecodedField::I8),
            TIBRVMSG_I8ARRAY => fld.try_decode().map(DecodedField::I8Array),
            TIBRVMSG_U16 => fld.try_decode().map(DecodedField::U16),
            TIBRVMSG_U16ARRAY => fld.try_decode().map(DecodedField::U16Array),
            TIBRVMSG_I16 => fld.try_decode().map(DecodedField::I16),
            TIBRVMSG_I16ARRAY => fld.try_decode().map(DecodedField::I16Array),
            TIBRVMSG_U32 => fld.try_decode().map(DecodedField::U32),
            TIBRVMSG_U32ARRAY => fld.try_decode().map(DecodedField::U32Array),
            TIBRVMSG_I32 => fld.try_decode().map(DecodedField::I32),
            TIBRVMSG_I32ARRAY => fld.try_decode().map(DecodedField::I32Array),
            TIBRVMSG_U64 => fld.try_decode().map(DecodedField::U64),
            TIBRVMSG_U64ARRAY => fld.try_decode().map(DecodedField::U64Array),
            TIBRVMSG_I64 => fld.try_decode().map(DecodedField::I64),
            TIBRVMSG_I64ARRAY => fld.try_decode().map(DecodedField::I64Array),
            TIBRVMSG_F32 => fld.try_decode().map(DecodedField::F32),
            TIBRVMSG_F32ARRAY => fld.try_decode().map(DecodedField::F32Array),
            TIBRVMSG_F64 => fld.try_decode().map(DecodedField::F64),
            TIBRVMSG_F64ARRAY => fld.try_decode().map(DecodedField::F64Array),
            TIBRVMSG_BOOL => fld.try_decode().map(DecodedField::Bool),
            TIBRVMSG_DATETIME => fld.try_decode().map(DecodedField::DateTime),
            TIBRVMSG_IPADDR32 => fld.try_decode().map(DecodedField::Ipv4),
            TIBRVMSG_IPPORT16 => tibrv_try_decode_port(fld).map(DecodedField::IpPort),
            TIBRVMSG_OPAQUE => unsafe {
                tibrv_try_decode_opaque::<u8>(fld).map(DecodedField::Opaque)
            },
            _ => Err(ErrorKind::UnknownFieldTypeError(fld.inner.type_).into()),
        }
    }
}

/// A structure wrapping a `tibrvMsgField`
pub struct MsgField {
    pub name: Option<CString>,
    pub inner: tibrvMsgField,
}

impl MsgField {
    pub fn try_decode<'a, T: Decodable<'a>>(&'a self) -> Result<T, TibrvError> {
        <T>::tibrv_try_decode(self)
    }
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
    pub(crate) inner: MsgField,
    pub(crate) phantom: PhantomData<&'a Msg>,
}

impl<'a> Deref for BorrowedMsgField<'a> {
    type Target = MsgField;
    fn deref(&self) -> &MsgField {
        &self.inner
    }
}

/// A builder for `MsgField`.
pub struct Builder<'a, T: 'a>
where
    T: Encodable,
{
    name: Option<&'a str>,
    id: Option<u32>,
    data: &'a T,
}

impl<'a, T> Builder<'a, T>
where
    T: Encodable,
{
    /// Creates a new `Builder` used to construct a `MsgField`.o
    ///
    /// At least one of `with_name` or `with_id` must be called
    /// to supply identifier information for the `MsgField`.
    pub fn new(data: &'a T) -> Builder<'a, T> {
        Builder {
            name: None,
            id: None,
            data,
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
pub trait Decodable<'a> {
    /// Try and decode a supplied `MsgField` as this type.
    ///
    /// Rendezvous message fields include some primitive type information,
    /// but this method may fail if the sending party incorrectly encodes
    /// the data fields.
    fn tibrv_try_decode(msg: &'a MsgField) -> Result<Self, TibrvError>
    where
        Self: Sized;
}

#[rustfmt::skip]
macro_rules! must_name {
    ($name:ident, $id:ident) => (
        if $id.is_some() {
            assert!($name.is_some(), "ID may only be defined along with name.");
        }
    )
}

#[rustfmt::skip]
macro_rules! encodable {
    ($base_type:ty, $tibrv_type:tt, $local:ident, $tibrv_flag:expr) => (
        impl Encodable for $base_type {
            fn tibrv_encode(&self, name: Option<&str>, id: Option<u32>) -> MsgField {
                must_name!(name, id);
                let name_cstr = name.map_or(None, |s| Some(CString::new(s).unwrap()));
                let ptr = name_cstr.as_ref().map_or(std::ptr::null(), |s| s.as_ptr());
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

        impl<'a> Decodable<'a> for $base_type {
            fn tibrv_try_decode(msg: &'a MsgField) -> Result<$base_type, TibrvError> {
                if msg.inner.count > 1 { Err(ErrorKind::NonVectorFieldError)? };
                if msg.inner.type_ == $tibrv_flag as u8 {
                    let val = unsafe { msg.inner.data.$local };
                    let decoded: $base_type = val.into();
                    Ok(decoded)
                } else {
                    Err(ErrorKind::FieldTypeError.into())
                }
            }
        }
    )
}

#[rustfmt::skip]
macro_rules! array_encodable {
    ($base_type:ty, $tibrv_flag:expr) => (
        impl<'a> Encodable for &'a [$base_type] {
            fn tibrv_encode(&self, name: Option<&str>, id: Option<u32>) -> MsgField {
                must_name!(name, id);
                let name_cstr = name.map_or(None, |s| Some(CString::new(s).unwrap()));
                let ptr = name_cstr.as_ref().map_or(std::ptr::null(), |s| s.as_ptr());
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

        impl<'a> Decodable<'a> for &'a [$base_type] {
            fn tibrv_try_decode(msg: &'a MsgField) -> Result<&'a [$base_type], TibrvError> {
                if msg.inner.type_ != $tibrv_flag as u8 {
                    Err(ErrorKind::FieldTypeError)?
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
        must_name!(name, id);
        let name_cstr = name.and_then(|s| Some(CString::new(s).unwrap()));
        let ptr = name_cstr.as_ref().map_or(std::ptr::null(), |s| s.as_ptr());
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

impl<'a> Decodable<'a> for &'a CStr {
    fn tibrv_try_decode(msg: &'a MsgField) -> Result<&'a CStr, TibrvError> {
        if msg.inner.type_ != TIBRVMSG_STRING as u8 {
            Err(ErrorKind::FieldTypeError)?
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
        must_name!(name, id);
        let name_cstr = name.and_then(|s| Some(CString::new(s).unwrap()));
        let ptr = name_cstr.as_ref().map_or(std::ptr::null(), |s| s.as_ptr());
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

impl<'a> Decodable<'a> for BorrowedMsg {
    fn tibrv_try_decode(msg: &'a MsgField) -> Result<Self, TibrvError> {
        if msg.inner.type_ != TIBRVMSG_MSG as u8 {
            Err(ErrorKind::FieldTypeError)?
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
pub fn tibrv_encode_port(port: u16, name: Option<&str>, id: Option<u32>) -> MsgField {
    must_name!(name, id);
    let name_cstr = name.and_then(|s| Some(CString::new(s).unwrap()));
    let ptr = name_cstr.as_ref().map_or(std::ptr::null(), |s| s.as_ptr());
    MsgField {
        name: name_cstr,
        inner: tibrvMsgField {
            name: ptr,
            size: std::mem::size_of::<u16>() as tibrv_u32,
            count: 1 as tibrv_u32,
            data: tibrvLocalData {
                ipport16: port.to_be(),
            },
            id: id.unwrap_or(0) as tibrv_u16,
            type_: TIBRVMSG_IPPORT16 as tibrv_u8,
        },
    }
}

/// Try and decode an IP Port message field.
pub fn tibrv_try_decode_port(msg: &MsgField) -> Result<u16, TibrvError> {
    if msg.inner.count > 1 {
        Err(ErrorKind::NonVectorFieldError)?
    }
    if msg.inner.type_ == TIBRVMSG_IPPORT16 as u8 {
        let val = unsafe { msg.inner.data.ipport16 };
        let decoded = u16::from_be(val);
        Ok(decoded)
    } else {
        Err(ErrorKind::FieldTypeError)?
    }
}

/// Encode a slice as an opaque byte sequence.
pub unsafe fn tibrv_encode_opaque<'a, T: Copy>(
    slice: &'a [T],
    name: Option<&str>,
    id: Option<u32>,
) -> MsgField {
    must_name!(name, id);
    let name_cstr = name.and_then(|s| Some(CString::new(s).unwrap()));
    let ptr = name_cstr.as_ref().map_or(std::ptr::null(), |s| s.as_ptr());
    MsgField {
        name: name_cstr,
        inner: tibrvMsgField {
            name: ptr,
            size: std::mem::size_of_val(slice) as tibrv_u32,
            count: 1 as tibrv_u32,
            data: tibrvLocalData {
                buf: slice.as_ptr() as *const c_void,
            },
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
pub unsafe fn tibrv_try_decode_opaque<T: Copy>(
    msg: &MsgField,
) -> Result<&[T], TibrvError> {
    if msg.inner.type_ != TIBRVMSG_OPAQUE as u8 {
        Err(ErrorKind::FieldTypeError)?
    } else {
        assert!(!msg.inner.data.buf.is_null());
        // `size` in from_raw_parts is helpfully in 'elements' not bytes...
        let elements: usize = msg.inner.size as usize / std::mem::size_of::<T>();
        Ok(std::slice::from_raw_parts(
            msg.inner.data.buf as *const T,
            elements,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rustfmt::skip]
    macro_rules! test_encodable_array {
        ($rt:ty, $df:tt, $bs:expr, $val1:expr, $val2:expr, $val3:expr, $val4:expr) => (
            {
                let array: &[$rt] = &[$val1, $val2, $val3, $val4];
                let fld = array.tibrv_encode(Some("Array"), None);
                assert_eq!($bs, fld.inner.size);
                assert_eq!(4, fld.inner.count);

                match fld.try_decode().unwrap() {
                    DecodedField::$df(slice) => {
                        assert_eq!(4, slice.len());
                        assert_eq!($val1, slice[0]);
                        assert_eq!($val2, slice[1]);
                        assert_eq!($val3, slice[2]);
                        assert_eq!($val4, slice[3]);
                    },
                    _ => panic!("Field did not decode as expected"),
                }
            }
        )
    }

    #[test]
    fn decode_field_arrays() {
        test_encodable_array!(u8, U8Array, 1, 1u8, 2u8, 3u8, 4u8);
        test_encodable_array!(i8, I8Array, 1, -5i8, -6i8, -7i8, -8i8);
        test_encodable_array!(u16, U16Array, 2, 1u16, 2u16, 3u16, 4u16);
        test_encodable_array!(i16, I16Array, 2, -5i16, -6i16, -7i16, -8i16);
        test_encodable_array!(u32, U32Array, 4, 1u32, 2u32, 3u32, 4u32);
        test_encodable_array!(i32, I32Array, 4, -5i32, -6i32, -7i32, -8i32);
        test_encodable_array!(u64, U64Array, 8, 1u64, 2u64, 3u64, 4u64);
        test_encodable_array!(i64, I64Array, 8, -5i64, -6i64, -7i64, -8i64);
        // Floating point
        test_encodable_array!(f32, F32Array, 4, 1f32, -2f32, 3f32, -4f32);
        test_encodable_array!(f64, F64Array, 8, 1f64, -2f64, 3f64, -4f64);
    }

    macro_rules! test_encodable {
        ($df:tt, $val:expr) => {{
            let val = $val;
            let fld = val.tibrv_encode(None, None);
            match fld.try_decode().unwrap() {
                DecodedField::$df(v) => assert_eq!(val, v),
                _ => panic!("Field did not decode as expected"),
            }
        }};
    }

    #[test]
    fn decode_fields() {
        use chrono::NaiveDate;
        test_encodable!(U8, std::u8::MAX);
        test_encodable!(I8, std::i8::MAX);
        test_encodable!(U16, std::u16::MAX);
        test_encodable!(I16, std::i16::MAX);
        test_encodable!(U32, std::u32::MAX);
        test_encodable!(I32, std::i32::MAX);
        test_encodable!(U64, std::u64::MAX);
        test_encodable!(I64, std::i64::MAX);
        // Floating point
        test_encodable!(F32, std::f32::MAX);
        test_encodable!(F64, std::f64::MAX);
        // Custom types
        test_encodable!(Bool, true);
        test_encodable!(DateTime, NaiveDate::from_ymd(2018, 7, 24).and_hms(1, 2, 3));
        test_encodable!(Ipv4, Ipv4Addr::new(127, 0, 0, 1));
        {
            let addr = Ipv4Addr::new(127, 0, 0, 1);
            let fld = addr.tibrv_encode(Some("IP Address"), None);
            match <DecodedField>::tibrv_try_decode(&fld).unwrap() {
                DecodedField::Ipv4(v) => assert_eq!(addr, v),
                _ => panic!("Field did not decode as expected"),
            }
        }
        {
            let port = 1;
            let fld = tibrv_encode_port(port, Some("Port"), None);
            match <DecodedField>::tibrv_try_decode(&fld).unwrap() {
                DecodedField::IpPort(v) => assert_eq!(port, v),
                _ => panic!("Field did not decode as expected"),
            }
        }
        {
            let cs = CString::new("teststring").unwrap();
            let cstr = cs.as_c_str();
            let fld = cstr.tibrv_encode(None, None);
            match <DecodedField>::tibrv_try_decode(&fld).unwrap() {
                DecodedField::String(v) => assert_eq!(cstr, v),
                _ => panic!("Field did not decode as expected"),
            }
        }
        {
            let mut msg = Msg::new().unwrap();
            let data = CString::new("Hello world!").unwrap();
            let mut field = Builder::new(&data.as_c_str()).with_name("string").encode();
            let _ = msg.add_field(&mut field).unwrap();
            let fld = (&msg).tibrv_encode(None, None);
            match <DecodedField>::tibrv_try_decode(&fld).unwrap() {
                DecodedField::Message(v) => {
                    let m = v.to_owned().unwrap();
                    let d = m.get_field_by_index(0).unwrap();
                    let c = <&CStr>::tibrv_try_decode(&d).unwrap();
                    assert_eq!(data.as_c_str(), c);
                }
                _ => panic!("Field did not decode as expected"),
            }
        }
        {
            let slice = &[1u8, 2, 3, 4];
            let fld = unsafe { tibrv_encode_opaque::<u8>(slice, None, None) };
            match <DecodedField>::tibrv_try_decode(&fld).unwrap() {
                DecodedField::Opaque(b) => assert_eq!(slice, b),
                _ => panic!("Field did not decode as expected"),
            }
        }
    }

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
            assert_eq!(
                dt.tibrv_encode(Some("DateTime"), Some(0))
                    .inner
                    .data
                    .date
                    .sec,
                3600
            );
        }
    }

    #[test]
    fn test_datetime_roundtrip() {
        use chrono::prelude::*;
        let dt = NaiveDate::from_ymd(2017, 1, 1).and_hms_milli(0, 0, 0, 0);
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
        let tibport = tibrv_encode_port(port, Some("Port"), None);
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
        let field = Builder::new(&data).with_name("Name").with_id(4).encode();
        assert_eq!(8, field.inner.size);
        assert_eq!(5, field.inner.count);
    }

    #[test]
    #[should_panic]
    fn id_without_name() {
        let unsigned64: u64 = 0;
        let _ = unsigned64.tibrv_encode(None, Some(1));
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
