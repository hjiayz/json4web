#![no_std]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate wasm_bindgen_test;

use alloc::borrow::ToOwned;
use alloc::fmt::Debug;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::PartialEq;
use json4web::de::*;
use serde_derive::Deserialize;
use serde::serde_if_integer128;

#[cfg(test)]
fn test<'a, D: serde::Deserialize<'a> + Debug + PartialEq>(expected: D, j: &'a str) {
    assert_eq!(expected, from_str::<'a, D>(j).unwrap());
}

#[test]
#[wasm_bindgen_test]
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
#[wasm_bindgen_test]
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
#[wasm_bindgen_test]
pub fn test_bytes() {
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
}

#[test]
#[wasm_bindgen_test]
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

#[test]
#[wasm_bindgen_test]
fn test_string() {
    test("\"".to_owned(), r#""\"""#);
    test("\\".to_owned(), r#""\\""#);
    test("/\u{8}\u{c}\n\r\t".to_owned(), r#""\/\b\f\n\r\t""#);
}

#[test]
#[wasm_bindgen_test]
fn test_number() {
    test(123u8, r#"123"#);
    test(12345u16, r#"12345"#);
    test(1234512345u32, r#"1234512345"#);
    test(1234512345u64, r#""1234512345""#);
    test(123i8, r#"123"#);
    test(12345i16, r#"12345"#);
    test(1234512345i32, r#"1234512345"#);
    test(1234512345i64, r#""1234512345""#);
    serde_if_integer128!{
        test(12345123451234512345u128, r#""12345123451234512345""#);
        test(12345123451234512345i128, r#""12345123451234512345""#);
    }
    test(1.3f32, r#"1.3"#);
    test(1.3f64, r#"1.3"#);
}
