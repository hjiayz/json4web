use crate::{Error, Result};
use alloc::borrow::Cow;
use alloc::str::Chars;
use alloc::string::String;
use core::convert::TryFrom;
use core::num::ParseFloatError;
use core::num::ParseIntError;
use core::str::FromStr;
use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::serde_if_integer128;

pub fn from_slice<'a, T>(input: &'a [u8]) -> Result<T>
where
    T: serde::Deserialize<'a>,
{
    use core::str;
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

fn parse_escape(chs: &mut Chars, buf: &mut String, at: &mut usize) -> Result<()> {
    let ch = chs.next().ok_or(Error::UnexpectedEnd)?;
    let ch = match ch {
        '"' | '\\' | '/' => ch,
        'b' => '\x08',
        'f' => '\x0c',
        'n' => '\n',
        'r' => '\r',
        't' => '\t',
        'u' => {
            let hchs = chs.by_ref().take(4);
            let mut ch = 0u32;
            for c in hchs {
                ch = (ch << 4) + c.to_digit(16).ok_or(Error::InvalidUnicodeEscapeSequence)?;
            }
            *at += 4;
            char::try_from(ch).map_err(|_| Error::UnexpectedUnicodeEscapeSequence(ch))?
        }
        token => return Err(Error::UnexpectedToken(token)),
    };
    *at += 1;
    buf.push(ch);
    Ok(())
}

impl<'de> Deserializer<'de> {
    fn trim_start(&mut self) {
        self.0 = self.0.trim_start();
    }
    fn peek_char(&self) -> Result<char> {
        self.0.chars().next().ok_or(Error::UnexpectedEnd)
    }
    fn peek_u8(&self) -> Result<u8> {
        let bytes = self.0.as_bytes();
        if bytes.is_empty() {
            return Err(Error::UnexpectedEnd);
        }
        Ok(bytes[0])
    }
    fn next_char(&mut self) -> Result<char> {
        let ch = self.peek_char()?;
        self.0 = &self.0[ch.len_utf8()..];
        Ok(ch)
    }
    fn assert_next_char(&mut self, rhs: char) -> Result<()> {
        let ch = self.peek_char()?;
        if ch != rhs {
            return Err(Error::UnexpectedToken(ch));
        }
        self.0 = &self.0[ch.len_utf8()..];
        Ok(())
    }
    fn parse_string(&mut self) -> Result<Cow<'de, str>> {
        let mut chs = self.0.chars();
        let first_char = chs.next().ok_or(Error::UnexpectedEnd)?;
        if first_char != '"' {
            return Err(Error::UnexpectedToken(first_char));
        }
        let mut at = 1;
        let mut buf = None;
        loop {
            let ch = chs.next().ok_or(Error::UnexpectedEnd)?;
            let ch_len = ch.len_utf8();
            if ch == '\\' {
                if buf.is_none() {
                    buf = Some(String::from(&self.0[1..at]));
                }
                at += ch_len;
                parse_escape(&mut chs, buf.as_mut().unwrap(), &mut at)?;
                continue;
            }
            at += ch_len;
            if ch == '"' {
                break;
            }
            if let Some(s) = buf.as_mut() {
                s.push(ch)
            }
        }
        if let Some(buf) = buf {
            self.0 = &self.0[at..];
            return Ok(Cow::Owned(buf));
        }
        let s = &self.0[1..at - 1];
        self.0 = &self.0[at..];
        Ok(Cow::Borrowed(s))
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
        Err(Error::UnexpectedToken(self.peek_char()?))
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
        T: FromStr<Err = ParseFloatError> + From<f32>,
    {
        if self.0.starts_with("null") {
            return Ok(T::from(core::f32::NAN));
        }
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
            _ => Err(Error::UnexpectedToken(self.peek_char()?)),
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
        visitor.visit_i64(i64::from_str(&self.parse_string()?)?)
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
        visitor.visit_u64(u64::from_str(&self.parse_string()?)?)
    }

    serde_if_integer128! {

        fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            self.trim_start();
            visitor.visit_u128(u128::from_str(&self.parse_string()?)?)
        }

        fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            self.trim_start();
            visitor.visit_i128(i128::from_str(&self.parse_string()?)?)
        }

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
        visitor.visit_char(s.chars().next().ok_or(Error::UnexpectedToken('\"'))?)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        match self.parse_string()? {
            Cow::Borrowed(s) => visitor.visit_borrowed_str(s),
            Cow::Owned(s) => visitor.visit_string(s),
        }
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
        let b = base64::decode_config(s.as_ref(), base64::URL_SAFE)?;
        visitor.visit_bytes(&b)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.trim_start();
        let s = self.parse_string()?;
        let b = base64::decode_config(s.as_ref(), base64::URL_SAFE)?;
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
            Err(Error::UnexpectedToken(self.peek_char()?))
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
            self.trim_start();
            self.assert_next_char(']')?;
            Ok(value)
        } else {
            Err(Error::UnexpectedToken(self.peek_char()?))
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
        let start = self.next_char()?;
        if start == '{' {
            let value = visitor.visit_map(CommaSeparated::new(&mut self))?;
            self.trim_start();
            let end = self.next_char()?;
            if end == '}' {
                Ok(value)
            } else {
                Err(Error::UnexpectedToken(end))
            }
        } else {
            Err(Error::UnexpectedToken(start))
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
        let start = self.peek_char()?;
        if start == '"' {
            visitor.visit_enum(self.parse_string()?.into_deserializer())
        } else if start == '{' {
            self.next_char().unwrap();
            let value = visitor.visit_enum(Enum::new(self))?;
            self.trim_start();
            let end = self.next_char()?;
            if end == '}' {
                Ok(value)
            } else {
                Err(Error::UnexpectedToken(end))
            }
        } else {
            Err(Error::UnexpectedToken(start))
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
        self.de.trim_start();
        if self.de.peek_char()? == ']' {
            return Ok(None);
        }
        if !self.first {
            self.de.assert_next_char(',')?;
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
        self.de.trim_start();
        let token = self.de.peek_char()?;
        if token == '}' {
            return Ok(None);
        }
        if !self.first {
            self.de.assert_next_char(',')?;
        }
        self.first = false;
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        self.de.trim_start();
        self.de.assert_next_char(':')?;
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
        self.de.trim_start();
        self.de.assert_next_char(':')?;
        Ok((val, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Err(Error::UnexpectedToken(self.de.peek_char()?))
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
