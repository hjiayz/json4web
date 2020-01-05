use crate::{Error, Result};
use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use std::num::ParseFloatError;
use std::num::ParseIntError;
use std::str::FromStr;

pub fn from_slice<'a, T>(input: &'a [u8]) -> Result<T>
where
    T: serde::Deserialize<'a>,
{
    use std::str;
    from_str(str::from_utf8(input)?)
}

pub fn from_str<'a, T>(input: &'a str) -> Result<T>
where
    T: serde::Deserialize<'a>,
{
    let mut des = Deserializer(input);
    T::deserialize(&mut des)
}

pub struct Deserializer<'de>(&'de str);

impl<'de> Deserializer<'de> {
    fn trim_start(&mut self) {
        self.0 = self.0.trim_start();
    }
    fn peek_char(&self) -> Result<char> {
        self.0.chars().next().ok_or(Error::EndOfString)
    }
    fn peek_u8(&self) -> Result<u8> {
        let bytes = self.0.as_bytes();
        if bytes.is_empty() {
            return Err(Error::EndOfString);
        }
        Ok(bytes[0])
    }
    fn next_char(&mut self) -> Result<char> {
        let ch = self.peek_char()?;
        self.0 = &self.0[ch.len_utf8()..];
        Ok(ch)
    }
    fn parse_string(&mut self) -> Result<&'de str> {
        let mut chs = self.0.chars();
        if chs.next() != Some('"') {
            return Err(Error::ParseError("string"));
        }
        let mut escape = false;
        let mut at = 1;
        loop {
            let ch = chs.next().ok_or(Error::EndOfString)?;
            at += ch.len_utf8();
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
            }
            if ch == '"' {
                break;
            }
        }
        let start = &self.0[1..at - 1];
        self.0 = &self.0[at..];
        Ok(start)
    }

    fn parse_bool(&mut self) -> Result<bool> {
        let bytes = self.0.as_bytes();
        let vals: [&[u8]; 4] = [b"1", b"0", b"true", b"false"];
        for (count, s) in vals.iter().enumerate() {
            if bytes.starts_with(s) {
                self.0 = &self.0[s.len()..];
                return Ok(count & 1 == 0);
            }
        }
        Err(Error::ParseError("bool"))
    }

    fn parse_unsigned<T>(&mut self) -> Result<T>
    where
        T: FromStr<Err = ParseIntError>,
    {
        let chs = self.0.chars();
        let mut offset = 0usize;
        for ch in chs {
            if ch.is_ascii_digit() {
                offset += ch.len_utf8();
                continue;
            }
            break;
        }
        let val = T::from_str(&self.0[..offset])?;
        self.0 = &self.0[offset..];
        Ok(val)
    }

    fn parse_signed<T>(&mut self) -> Result<T>
    where
        T: FromStr<Err = ParseIntError>,
    {
        let chs = self.0.chars();
        let mut offset = 0usize;
        for ch in chs {
            if ch.is_ascii_digit() || ch == '-' {
                offset += ch.len_utf8();
                continue;
            }
            break;
        }
        let val = T::from_str(&self.0[..offset])?;
        self.0 = &self.0[offset..];
        Ok(val)
    }

    fn parse_float<T>(&mut self) -> Result<T>
    where
        T: FromStr<Err = ParseFloatError>,
    {
        let chs = self.0.chars();
        let mut offset = 0usize;
        for ch in chs {
            if ch.is_ascii_digit() || ch == '-' || ch == '.' {
                offset += ch.len_utf8();
                continue;
            }
            break;
        }
        let val = T::from_str(&self.0[..offset])?;
        self.0 = &self.0[offset..];
        Ok(val)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        match self.peek_u8()? {
            b'n' => self.deserialize_unit(visitor),
            b't' | b'f' => self.deserialize_bool(visitor),
            b'"' => self.deserialize_str(visitor),
            b'0'..=b'9' | b'-' => self.deserialize_f64(visitor),
            b'[' => self.deserialize_seq(visitor),
            b'{' => self.deserialize_map(visitor),
            _ => Err(Error::Syntax),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        visitor.visit_i8(self.parse_signed()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        visitor.visit_i16(self.parse_signed()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        visitor.visit_i32(self.parse_signed()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        visitor.visit_i64(i64::from_str(self.parse_string()?)?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        visitor.visit_u8(self.parse_unsigned()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        visitor.visit_u16(self.parse_unsigned()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        visitor.visit_u32(self.parse_unsigned()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        visitor.visit_u64(u64::from_str(self.parse_string()?)?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        visitor.visit_f32(self.parse_float()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        visitor.visit_f64(self.parse_float()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        let s = self.parse_string()?;
        if s.len() != 1 {
            return Err(Error::ParseError("char"));
        }
        visitor.visit_char(s.chars().next().unwrap())
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        visitor.visit_borrowed_str(self.parse_string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        let s = self.parse_string()?;
        let b = base64::decode_config(&s, base64::URL_SAFE)?;
        visitor.visit_bytes(&b)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        let s = self.parse_string()?;
        let b = base64::decode_config(&s, base64::URL_SAFE)?;
        visitor.visit_byte_buf(b)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        if self.peek_char()? == 'n' {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        if self.0.starts_with("null") {
            self.0 = &self.0["null".len()..];
            visitor.visit_unit()
        } else {
            Err(Error::ParseError("unit"))
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        if self.next_char()? == '[' {
            let value = visitor.visit_seq(CommaSeparated::new(&mut self))?;
            if self.next_char()? == ']' {
                Ok(value)
            } else {
                Err(Error::ParseError("array end"))
            }
        } else {
            Err(Error::ParseError("array"))
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        if self.next_char()? == '{' {
            let value = visitor.visit_map(CommaSeparated::new(&mut self))?;
            if self.next_char()? == '}' {
                Ok(value)
            } else {
                Err(Error::ParseError("map end"))
            }
        } else {
            Err(Error::ParseError("map"))
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        if self.peek_char()? == '"' {
            visitor.visit_enum(self.parse_string()?.into_deserializer())
        } else if self.next_char()? == '{' {
            let value = visitor.visit_enum(Enum::new(self))?;
            if self.next_char()? == '}' {
                Ok(value)
            } else {
                Err(Error::ParseError("map end"))
            }
        } else {
            Err(Error::ParseError("enum"))
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

struct CommaSeparated<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    first: bool,
}

impl<'a, 'de> CommaSeparated<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        CommaSeparated { de, first: true }
    }
}

impl<'de, 'a> SeqAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.de.peek_char()? == ']' {
            return Ok(None);
        }
        if !self.first && self.de.next_char()? != ',' {
            return Err(Error::ParseError("array comma"));
        }
        self.first = false;
        seed.deserialize(&mut *self.de).map(Some)
    }
}

impl<'de, 'a> MapAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if self.de.peek_char()? == '}' {
            return Ok(None);
        }
        if !self.first && self.de.next_char()? != ',' {
            return Err(Error::ParseError("map comma"));
        }
        self.first = false;
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        if self.de.next_char()? != ':' {
            return Err(Error::ParseError("map colon"));
        }
        seed.deserialize(&mut *self.de)
    }
}

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Enum { de }
    }
}

impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.de)?;
        if self.de.next_char()? == ':' {
            Ok((val, self))
        } else {
            Err(Error::ParseError("map colon"))
        }
    }
}

impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Err(Error::ParseError("string"))
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self.de, visitor)
    }
}
