use alloc::borrow::Cow;
use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use serde::serde_if_integer128;
use serde::{ser, Serialize};

use crate::{Error, Result};

pub struct Serializer(Vec<Cow<'static, [u8]>>);

pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let mut serializer = Serializer(Vec::new());
    value.serialize(&mut serializer)?;
    let mut len = 0;
    for s in serializer.0.iter() {
        len += s.len();
    }
    let mut result = Vec::with_capacity(len);
    for s in serializer.0.iter() {
        result.extend_from_slice(&s);
    }
    Ok(unsafe { String::from_utf8_unchecked(result) })
}

impl Serializer {
    fn append(&mut self, data: &'static str) {
        self.0.push(Cow::Borrowed(data.as_bytes()))
    }
    fn append_string(&mut self, data: String) {
        self.0.push(Cow::Owned(data.into_bytes()))
    }
    fn serialize_simple_string(&mut self, num: String) {
        self.append("\"");
        self.append_string(num);
        self.append("\"");
    }
}
impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Compound<'a>;
    type SerializeTuple = Compound<'a>;
    type SerializeTupleStruct = Compound<'a>;
    type SerializeTupleVariant = Compound<'a>;
    type SerializeMap = Compound<'a>;
    type SerializeStruct = Compound<'a>;
    type SerializeStructVariant = Compound<'a>;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.append(if v { "1" } else { "0" });
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.append_string(v.to_string());
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.append_string(v.to_string());
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.append_string(v.to_string());
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.serialize_simple_string(v.to_string());
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.append_string(v.to_string());
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.append_string(v.to_string());
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.append_string(v.to_string());
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.serialize_simple_string(v.to_string());
        Ok(())
    }

    serde_if_integer128! {

        fn serialize_u128(self, v: u128) -> Result<()> {
            self.serialize_simple_string(v.to_string());
            Ok(())
        }

        fn serialize_i128(self, v: i128) -> Result<()> {
            self.serialize_simple_string(v.to_string());
            Ok(())
        }

    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        if v.is_finite() {
            return Err(Error::NaN);
        }
        let mut buffer = ryu::Buffer::new();
        self.append_string(buffer.format_finite(v).to_owned());
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        if v.is_finite() {
            return Err(Error::NaN);
        }
        let mut buffer = ryu::Buffer::new();
        self.append_string(buffer.format_finite(v).to_owned());
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.append("\"");
        self.append_string(v.escape_default().to_string());
        self.append("\"");
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.serialize_simple_string(base64::encode_config(v, base64::URL_SAFE));
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        self.append("null");
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.append("{");
        variant.serialize(&mut *self)?;
        self.append(":");
        value.serialize(&mut *self)?;
        self.append("}");
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.append("[");
        Ok(Compound(self, true))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.append("{");
        variant.serialize(&mut *self)?;
        self.append(":[");
        Ok(Compound(self, true))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.append("{");
        Ok(Compound(self, true))
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.append("{");
        variant.serialize(&mut *self)?;
        self.append(":{");
        Ok(Compound(self, true))
    }
}

pub struct Compound<'a>(&'a mut Serializer, bool);
impl<'a> Compound<'a> {
    fn append(&mut self, data: &'static str) {
        self.0.append(data)
    }
    fn first(&mut self) -> bool {
        let b = self.1;
        self.1 = false;
        b
    }
}

impl<'a> ser::SerializeSeq for Compound<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.first() {
            self.append(",");
        }
        value.serialize(&mut *self.0)
    }

    fn end(mut self) -> Result<()> {
        self.append("]");
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for Compound<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.first() {
            self.append(",");
        }
        value.serialize(&mut *self.0)
    }

    fn end(mut self) -> Result<()> {
        self.append("]");
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for Compound<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.first() {
            self.append(",");
        }
        value.serialize(&mut *self.0)
    }

    fn end(mut self) -> Result<()> {
        self.append("]");
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for Compound<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.first() {
            self.append(",");
        }
        value.serialize(&mut *self.0)
    }

    fn end(mut self) -> Result<()> {
        self.append("]}");
        Ok(())
    }
}

impl<'a> ser::SerializeMap for Compound<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.first() {
            self.append(",");
        }
        key.serialize(&mut *self.0)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.append(":");
        value.serialize(&mut *self.0)
    }

    fn end(mut self) -> Result<()> {
        self.append("}");
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for Compound<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.first() {
            self.append(",");
        }
        key.serialize(&mut *self.0)?;
        self.append(":");
        value.serialize(&mut *self.0)
    }

    fn end(mut self) -> Result<()> {
        self.append("}");
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for Compound<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.first() {
            self.append(",");
        }
        key.serialize(&mut *self.0)?;
        self.append(":");
        value.serialize(&mut *self.0)
    }

    fn end(mut self) -> Result<()> {
        self.append("}}");
        Ok(())
    }
}
