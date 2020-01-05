use serde::{ser, Serialize};
use std::io::Write;

use crate::{Error, Result};

pub struct Serializer<W> {
    writer: W,
}

pub fn to_buf<T, B>(value: &T, buf: &mut B) -> Result<()>
where
    T: Serialize,
    B: Write,
{
    let mut serializer = Serializer { writer: buf };
    value.serialize(&mut serializer)
}

pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut vec = Vec::new();
    to_buf(value, &mut vec)?;
    Ok(vec)
}

pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    unsafe { Ok(String::from_utf8_unchecked(to_vec(value)?)) }
}

impl<'a, W> Serializer<W>
where
    W: Write,
{
    fn append(&mut self, data: &str) -> Result<()> {
        self.writer.write_all(data.as_bytes())?;
        Ok(())
    }
}
impl<'a, W> ser::Serializer for &'a mut Serializer<W>
where
    W: Write,
{
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Compound<'a, W>;
    type SerializeTuple = Compound<'a, W>;
    type SerializeTupleStruct = Compound<'a, W>;
    type SerializeTupleVariant = Compound<'a, W>;
    type SerializeMap = Compound<'a, W>;
    type SerializeStruct = Compound<'a, W>;
    type SerializeStructVariant = Compound<'a, W>;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.append(if v { "1" } else { "0" })?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.append(&v.to_string())?;
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.append(&v.to_string())?;
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.append(&v.to_string())?;
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.append(&v.to_string())?;
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.append(&v.to_string())?;
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.append(&v.to_string())?;
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        if v.is_finite() {
            return Err(Error::NaN);
        }
        let mut buffer = ryu::Buffer::new();
        self.append(buffer.format_finite(v))?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        if v.is_finite() {
            return Err(Error::NaN);
        }
        let mut buffer = ryu::Buffer::new();
        self.append(buffer.format_finite(v))?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.append("\"")?;
        self.append(&v.escape_default().to_string())?;
        self.append("\"")?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.serialize_str(&base64::encode_config(v, base64::URL_SAFE))
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
        self.append("null")?;
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
        self.append("{")?;
        variant.serialize(&mut *self)?;
        self.append(":")?;
        value.serialize(&mut *self)?;
        self.append("}")?;
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.append("[")?;
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
        self.append("{")?;
        variant.serialize(&mut *self)?;
        self.append(":[")?;
        Ok(Compound(self, true))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.append("{")?;
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
        self.append("{")?;
        variant.serialize(&mut *self)?;
        self.append(":{")?;
        Ok(Compound(self, true))
    }
}

pub struct Compound<'a, W>(&'a mut Serializer<W>, bool);
impl<'a, W> Compound<'a, W>
where
    W: Write,
{
    fn append(&mut self, data: &str) -> Result<()> {
        self.0.append(data)
    }
    fn first(&mut self) -> bool {
        let b = self.1;
        self.1 = false;
        b
    }
}

impl<'a, W> ser::SerializeSeq for Compound<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.first() {
            self.append(",")?;
        }
        value.serialize(&mut *self.0)
    }

    fn end(mut self) -> Result<()> {
        self.append("]")?;
        Ok(())
    }
}

impl<'a, W> ser::SerializeTuple for Compound<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.first() {
            self.append(",")?;
        }
        value.serialize(&mut *self.0)
    }

    fn end(mut self) -> Result<()> {
        self.append("]")?;
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleStruct for Compound<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.first() {
            self.append(",")?;
        }
        value.serialize(&mut *self.0)
    }

    fn end(mut self) -> Result<()> {
        self.append("]")?;
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleVariant for Compound<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.first() {
            self.append(",")?;
        }
        value.serialize(&mut *self.0)
    }

    fn end(mut self) -> Result<()> {
        self.append("]}")?;
        Ok(())
    }
}

impl<'a, W> ser::SerializeMap for Compound<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.first() {
            self.append(",")?;
        }
        key.serialize(&mut *self.0)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.append(":")?;
        value.serialize(&mut *self.0)
    }

    fn end(mut self) -> Result<()> {
        self.append("}")?;
        Ok(())
    }
}

impl<'a, W> ser::SerializeStruct for Compound<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.first() {
            self.append(",")?;
        }
        key.serialize(&mut *self.0)?;
        self.append(":")?;
        value.serialize(&mut *self.0)
    }

    fn end(mut self) -> Result<()> {
        self.append("}")?;
        Ok(())
    }
}

impl<'a, W> ser::SerializeStructVariant for Compound<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.first() {
            self.append(",")?;
        }
        key.serialize(&mut *self.0)?;
        self.append(":")?;
        value.serialize(&mut *self.0)
    }

    fn end(mut self) -> Result<()> {
        self.append("}}")?;
        Ok(())
    }
}
