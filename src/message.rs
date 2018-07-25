//! Interfaces for creating and using Rendezvous Messages

use errors::*;
use failure::ResultExt;
use field::*;
use std;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::mem;
use tibrv_sys::*;

pub struct MsgIter<'m> {
    msg: &'m Msg,
    index: u32,
}

impl<'m> Iterator for MsgIter<'m> {
    type Item = Result<BorrowedMsgField<'m>, TibrvError>;

    fn next(&mut self) -> Option<Self::Item> {
        let num_fields = match self.msg.num_fields() {
            Ok(n) => n,
            Err(e) => return Some(Err(e)),
        };
        let r = if self.index >= num_fields {
            None
        } else {
            Some(self.msg.get_field_by_index(self.index))
        };
        self.index += 1;
        r
    }
}

impl<'m> IntoIterator for &'m Msg {
    type IntoIter = MsgIter<'m>;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        MsgIter {
            msg: self,
            index: 0,
        }
    }
}

/// A struct representing an owned Rendezvous Message.
///
/// The memory allocated to this type of Message is the responsibility
/// of the Rust application. When this type is dropped,
/// `tibrvMsg_Destroy` will be run to free any memory allocated
/// to store the message.
pub struct Msg {
    pub(crate) inner: tibrvMsg,
}

unsafe impl Send for Msg {}

impl Msg {
    /// Construct a new owned Rendezvous Message
    pub fn new() -> Result<Self, TibrvError> {
        let mut ptr: tibrvMsg = unsafe { mem::zeroed() };
        unsafe { tibrvMsg_Create(&mut ptr) }.map(|_| Msg { inner: ptr })
    }

    pub fn try_clone(&self) -> Result<Self, TibrvError> {
        let mut ptr: tibrvMsg = unsafe { mem::zeroed() };
        unsafe { tibrvMsg_CreateCopy(self.inner, &mut ptr) }.map(|_| Msg { inner: ptr })
    }

    /// Add a `MsgField` to this message.
    ///
    /// The referenced field must be marked mutable, as although the
    /// process should not mutate the field, the C library makes no
    /// guarantee.
    ///
    /// The contents of message fields are always copied, therefore
    /// slice types must be `Copy`. A borrowed `MsgField` does not need
    /// to live beyond the point where it is added to the `Msg`.
    pub fn add_field(&mut self, field: &mut MsgField) -> Result<&mut Self, TibrvError> {
        unsafe { tibrvMsg_AddField(self.inner, &mut field.inner) }.map(|_| self)
    }

    /// Get a specified field from this message.
    ///
    /// Data in scalar fields is copied, and data in pointer fields
    /// is guaranteed to live at least as long as the parent `Msg`.
    ///
    /// This variant retrieves the field by name.
    pub fn get_field_by_name<'a>(&'a self, name: &str) -> Result<BorrowedMsgField<'a>, TibrvError> {
        self.get_field(Some(name), None)
    }

    /// Get a specified field from this message.
    ///
    /// Data in scalar fields is copied, and data in pointer fields
    /// is guaranteed to live at least as long as the parent `Msg`.
    ///
    /// This variant retrieves the field by id.
    pub fn get_field_by_id(&self, id: u32) -> Result<BorrowedMsgField, TibrvError> {
        self.get_field(None, Some(id))
    }

    /// Get a specified field from this message.
    ///
    /// Data in scalar fields is copied, and data in pointer fields
    /// is guaranteed to live at least as long as the parent `Msg`.
    ///
    /// This variant retrieves the field by index.
    pub fn get_field_by_index(&self, index: u32) -> Result<BorrowedMsgField, TibrvError> {
        let mut field: tibrvMsgField = unsafe { mem::zeroed() };

        unsafe {
            tibrvMsg_GetFieldByIndex(
                self.inner,
                &mut field,
                index as tibrv_u32,
            )
        }.and_then(|_| {
            if !field.name.is_null() {
                CString::new(unsafe { CStr::from_ptr(field.name) }.to_string_lossy().into_owned())
                     .context(ErrorKind::StrContentError)
                     .map_err(TibrvError::from)
                     .map(Some)
            } else {
                Ok(None)
            }
        }).map(|field_name| {
            BorrowedMsgField {
                inner: MsgField {
                    name: field_name,
                    inner: field,
                },
                phantom: PhantomData,
            }
        })
    }

    fn get_field<'a>(
        &'a self,
        name: Option<&str>,
        id: Option<u32>,
    ) -> Result<BorrowedMsgField<'a>, TibrvError> {
        assert_ne!(
            name.is_some(),
            id.is_some(),
            "One of id or name must be provided."
        );
        let mut field: tibrvMsgField = unsafe { mem::zeroed() };
        let field_name = name
            .map(|s| CString::new(s).context(ErrorKind::StrContentError))
            .map_or(Ok(None), |n| n.map(Some))?;

        let name_ptr = field_name.as_ref().map_or(std::ptr::null(), |s| s.as_ptr());
        unsafe {
            tibrvMsg_GetFieldEx(
                self.inner,
                name_ptr,
                &mut field,
                id.unwrap_or(0) as tibrv_u16,
            )
        }.map(|_| BorrowedMsgField {
            inner: MsgField {
                name: field_name,
                inner: field,
            },
            phantom: PhantomData,
        })
    }

    /// Remove a specified field from this message.
    ///
    /// Data in scalar fields is copied, and data in pointer fields
    /// is guaranteed to live at least as long as the parent `Msg`.
    ///
    /// This variant retrieves the field by name.
    pub fn remove_field_by_name(&self, name: &str) -> Result<(), TibrvError> {
        self.remove_field(Some(name), None)
    }

    /// Remove a specified field from this message.
    ///
    /// Data in scalar fields is copied, and data in pointer fields
    /// is guaranteed to live at least as long as the parent `Msg`.
    ///
    /// This variant retrieves the field by id.
    pub fn remove_field_by_id(&self, id: u32) -> Result<(), TibrvError> {
        self.remove_field(None, Some(id))
    }

    fn remove_field(&self, name: Option<&str>, id: Option<u32>) -> Result<(), TibrvError> {
        assert_ne!(
            name.is_some(),
            id.is_some(),
            "One of id or name must be provided."
        );
        let field_name = name
            .map(|s| CString::new(s).context(ErrorKind::StrContentError))
            .map_or(Ok(None), |n| n.map(Some))?;

        let name_ptr = field_name.as_ref().map_or(std::ptr::null(), |m| m.as_ptr());
        unsafe { tibrvMsg_RemoveFieldEx(self.inner, name_ptr, id.unwrap_or(0) as u16) }
            .map(|_| ())
    }

    /// Get the number of fields within this message.
    pub fn num_fields(&self) -> Result<u32, TibrvError> {
        let mut ptr: tibrv_u32 = unsafe { mem::zeroed() };
        unsafe { tibrvMsg_GetNumFields(self.inner, &mut ptr) }.map(|_| ptr as u32)
    }

    /// Expand the internal storage of a message.
    ///
    /// Messages automatically expand when adding a field would
    /// overflow the available space, however if adding a large
    /// number of fields it may be useful to preallocate enough
    /// space to hold them all.
    pub fn expand(&mut self, amount: i32) -> Result<&mut Self, TibrvError> {
        unsafe { tibrvMsg_Expand(self.inner, amount as tibrv_i32) }.map(|_| self)
    }

    /// Get the size of the message (in bytes).
    ///
    /// Does not include space allocated but not yet used.
    pub fn byte_size(&self) -> Result<u32, TibrvError> {
        let mut ptr: tibrv_u32 = unsafe { mem::zeroed() };
        unsafe { tibrvMsg_GetByteSize(self.inner, &mut ptr) }.map(|_| ptr as u32)
    }

    /// Set the send subject for the message.
    ///
    /// No wildcards are permitted in sender subjects.
    pub fn set_send_subject(&mut self, subject: &str) -> Result<(), TibrvError> {
        let subject_c = CString::new(subject).context(ErrorKind::StrContentError)?;
        unsafe { tibrvMsg_SetSendSubject(self.inner, subject_c.as_ptr()) }.map(|_| ())
    }
}

// Ensure we clean up messages we're responsible for.
impl Drop for Msg {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                tibrvMsg_Destroy(self.inner);
            }
        }
    }
}

/// A struct representing a borrowed Rendezvous Message.
///
/// The memory referenced by this type of Message is assumed to be
/// the responsibility of the Rendezvous C library, and will not be
/// freed when the `BorrowedMsg` is dropped.
pub struct BorrowedMsg {
    pub(crate) inner: tibrvMsg,
}

impl BorrowedMsg {
    /// Transform a BorrowedMsg into an owned Msg.
    ///
    /// Copies all data within the fields of the message, does not include
    /// any supplementary information attached to the message.
    ///
    /// This function is effectively an allocate and copy.
    pub fn to_owned(&self) -> Result<Msg, TibrvError> {
        let mut ptr: tibrvMsg = unsafe { mem::zeroed() };
        unsafe { tibrvMsg_CreateCopy(self.inner, &mut ptr) }.map(|_| Msg { inner: ptr })
    }

    /// Detach an inbound message from Rendezvous storage.
    ///
    /// This function is unsafe, as it is only valid for messages
    /// received in a callback invoked from Rendezvous.
    pub unsafe fn detach(self) -> Result<Msg, TibrvError> {
        let ptr = self.inner;
        tibrvMsg_Detach(ptr).map(|_| Msg { inner: ptr })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn create_msg() {
        let msg = Msg::new();
        assert!(msg.is_ok());
    }

    #[test]
    fn iterate_fields() {
        let data = CString::new("A string").unwrap();
        let cstr = data.as_c_str();
        let mut field = Builder::new(&cstr).with_name("StringField").encode();

        let slice: &[u16] = &[1, 2, 3, 4];
        let mut field2 = Builder::new(&slice)
            .with_name("Uint16 field")
            .with_id(2)
            .encode();

        let mut msg = Msg::new().unwrap();
        msg.add_field(&mut field).and_then(|m| m.add_field(&mut field2)).unwrap();

        let mut names = vec![];
        for f in &msg {
            names.push(f.unwrap().name.clone().unwrap().into_string().unwrap());
        }

        assert_eq!(2, names.len());
        assert_eq!(&names[..], &["StringField", "Uint16 field"]);
    }

    #[test]
    fn fields_by_index() {
        let data = CString::new("A string").unwrap();
        let cstr = data.as_c_str();
        let mut field = Builder::new(&cstr).with_name("StringField").encode();

        let slice: &[u16] = &[1, 2, 3, 4];
        let mut field2 = Builder::new(&slice)
            .with_name("Uint16 field")
            .with_id(2)
            .encode();

        let mut msg = Msg::new().unwrap();
        assert!(
            msg.add_field(&mut field).and_then(|m| m.add_field(&mut field2)).is_ok()
        );

        assert_eq!(2, msg.num_fields().unwrap());

        {
            let string_field = msg.get_field_by_index(0);
            assert!(string_field.is_ok());
            let string_field = string_field.unwrap();
            assert!(string_field.name.is_some());
            assert_eq!(string_field.name.as_ref().unwrap(), &CString::new("StringField").unwrap());
        }

        {
            let uint16_field = msg.get_field_by_index(1);
            assert!(uint16_field.is_ok());
            let uint16_field = uint16_field.unwrap();
            assert!(uint16_field.name.is_some());
            assert_eq!(uint16_field.name.as_ref().unwrap(), &CString::new("Uint16 field").unwrap());
        }

        assert!(msg.get_field_by_index(2).is_err());
    }

    #[test]
    fn field_iter() {
        let data = CString::new("A string").unwrap();
        let cstr = data.as_c_str();
        let mut field = Builder::new(&cstr).with_name("StringField").encode();

        let slice: &[u16] = &[1, 2, 3, 4];
        let mut field2 = Builder::new(&slice)
            .with_name("Uint16 field")
            .with_id(2)
            .encode();

        let mut msg = Msg::new().unwrap();
        assert!(
            msg.add_field(&mut field).and_then(|m| m.add_field(&mut field2)).is_ok()
        );

        assert_eq!(2, msg.num_fields().unwrap());

        let names = (&msg).into_iter()
            .map(|f|
                f.unwrap().name.as_ref().map(|c| c.to_string_lossy().into_owned()).unwrap()
            ).collect::<Vec<_>>();
        assert_eq!(names, vec!["StringField", "Uint16 field"]);
    }

    #[test]
    fn add_remove_fields() {
        let data = CString::new("A string").unwrap();
        let cstr = data.as_c_str();
        let mut field = Builder::new(&cstr).with_name("StringField").encode();

        let slice: &[u16] = &[1, 2, 3, 4];
        let mut field2 = Builder::new(&slice)
            .with_name("Uint16 field")
            .with_id(2)
            .encode();

        let mut msg = Msg::new().unwrap();
        assert!(
            msg.add_field(&mut field)
                .unwrap()
                .add_field(&mut field2)
                .is_ok()
        );

        assert_eq!(2, msg.num_fields().unwrap());

        assert!(msg.remove_field_by_id(2).is_ok());
        assert_eq!(1, msg.num_fields().unwrap());

        assert!(msg.remove_field_by_name("StringField").is_ok());
        assert_eq!(0, msg.num_fields().unwrap());
    }

    #[test]
    fn copy_msg() {
        let mut msg = Msg::new().unwrap();
        let slice: &[u16] = &[1, 2, 3, 4];
        let mut field = Builder::new(&slice)
            .with_name("Uint16 field")
            .with_id(2)
            .encode();
        let _ = msg.add_field(&mut field);
        let copy = msg.try_clone().unwrap();
        assert!(msg.inner != copy.inner); // Msgs are behind different pointers
        assert_eq!(34, msg.byte_size().unwrap()); // Both Msgs are populated
        assert_eq!(34, msg.byte_size().unwrap());
    }

    #[test]
    fn roundtrip_slice_msg() {
        let mut msg = Msg::new().unwrap();
        let slice: &[u16] = &[5, 4, 3, 2, 1];
        let mut field = Builder::new(&slice).with_name("slice").encode();
        let _ = msg.add_field(&mut field);
        let extracted = msg.get_field_by_name("slice").unwrap();

        let decoded = <&[u16]>::tibrv_try_decode(&extracted).unwrap();
        assert_eq!(5, decoded[0]);
    }

    #[test]
    fn roundtrip_string_msg() {
        use std::ffi::CStr;

        let mut msg = Msg::new().unwrap();
        let data = CString::new("Hello world!").unwrap();
        let mut field = Builder::new(&data.as_c_str()).with_name("string").encode();
        let _ = msg.add_field(&mut field).unwrap();
        let extracted = msg.get_field_by_name("string").unwrap();
        let decoded = <&CStr>::tibrv_try_decode(&extracted).unwrap();
        assert_eq!(decoded, data.as_c_str());
    }

    #[test]
    fn empty_size() {
        let msg = Msg::new().unwrap();
        assert_eq!(8, msg.byte_size().unwrap());
    }
}
