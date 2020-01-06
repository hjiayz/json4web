#![no_std]

#[macro_use]
extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use json4web::de::*;
use serde_derive::Deserialize;

#[test]
fn test_struct() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        int: u32,
        seq: Vec<String>,
    }

    let j = r#"{"int":1,"seq":["a","b"]}"#;
    let expected = Test {
        int: 1,
        seq: vec!["a".to_owned(), "b".to_owned()],
    };
    assert_eq!(expected, from_str(j).unwrap());
}

#[test]
fn test_enum() {
    #[derive(Deserialize, PartialEq, Debug)]
    enum E {
        Unit,
        Newtype(u32),
        Tuple(u32, u32),
        Struct { a: u32 },
    }

    let j = r#""Unit""#;
    let expected = E::Unit;
    assert_eq!(expected, from_str(j).unwrap());

    let j = r#"{"Newtype":1}"#;
    let expected = E::Newtype(1);
    assert_eq!(expected, from_str(j).unwrap());

    let j = r#"{"Tuple":[1,2]}"#;
    let expected = E::Tuple(1, 2);
    assert_eq!(expected, from_str(j).unwrap());

    let j = r#"{"Struct":{"a":1}}"#;
    let expected = E::Struct { a: 1 };
    assert_eq!(expected, from_str(j).unwrap());
}

#[test]
fn test_bytes() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct B {
        #[serde(with = "serde_bytes")]
        b: Vec<u8>,
    }
    let b = b"bytes test";
    let expected = B { b: b.to_vec() };
    let j = &format!(
        "{{\"b\":\"{}\"}}",
        base64::encode_config(b, base64::URL_SAFE)
    );
    assert_eq!(expected, from_str(j).unwrap());

    /*
    #[derive(Deserialize, PartialEq, Debug)]
    struct B2<'a> {
        #[serde(with = "serde_bytes")]
        b: &'a [u8],
    }
    let expected = B2 { b };
    let j = &format!(
        "{{\"b\":\"{}\"}}",
        base64::encode_config(b, base64::URL_SAFE)
    );
    assert_eq!(expected, from_str(j).unwrap());
    */
}

#[test]
fn test_bool() {
    let expected = true;
    let j = "1";
    assert_eq!(expected, from_str(j).unwrap());

    let j = "true";
    assert_eq!(expected, from_str(j).unwrap());

    let expected = false;
    let j = "0";
    assert_eq!(expected, from_str(j).unwrap());

    let j = "false";
    assert_eq!(expected, from_str(j).unwrap());
}
