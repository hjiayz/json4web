#![no_std]
#[macro_use]
extern crate alloc;
#[macro_use]
extern crate wasm_bindgen_test;

use alloc::vec::Vec;
use json4web::ser::*;
use serde::serde_if_integer128;
use serde_derive::Serialize;

#[cfg(test)]
fn test<S: serde::Serialize>(ser: S, expected: &str) {
    assert_eq!(to_string(&ser).unwrap(), expected);
}

#[test]
#[wasm_bindgen_test]
fn test_struct() {
    #[derive(Serialize)]
    struct Test {
        int: u32,
        seq: Vec<&'static str>,
    }

    let t = Test {
        int: 1,
        seq: vec!["a", "b"],
    };
    let expected = r#"{"int":1,"seq":["a","b"]}"#;
    test(t, expected);
}

#[test]
#[wasm_bindgen_test]
fn test_enum() {
    #[derive(Serialize)]
    enum E {
        Unit,
        Newtype(u32),
        Tuple(u32, u32),
        Struct { a: u32 },
    }

    let u = E::Unit;
    let expected = r#""Unit""#;
    test(u, expected);

    let n = E::Newtype(1);
    let expected = r#"{"Newtype":1}"#;
    test(n, expected);

    let t = E::Tuple(1, 2);
    let expected = r#"{"Tuple":[1,2]}"#;
    test(t, expected);

    let s = E::Struct { a: 1 };
    let expected = r#"{"Struct":{"a":1}}"#;
    test(s, expected);
}

#[test]
#[wasm_bindgen_test]
fn test_bytes() {
    use serde_bytes::Bytes;
    let bytes = &Bytes::new(b"bytes test");
    let expected = &format!("\"{}\"", base64::encode_config(bytes, base64::URL_SAFE));
    test(bytes, expected);
}

#[test]
#[wasm_bindgen_test]
fn test_bool() {
    let b = true;
    let expected = "1";
    test(b, expected);

    let b = false;
    let expected = "0";
    test(b, expected);
}

#[test]
#[wasm_bindgen_test]
fn test_string() {
    let s = "\"\\/\x08\x0c\n\r\t";
    let expected = r#""\"\\\/\b\f\n\r\t""#;
    test(s, expected);
    test("𐎅", "\"𐎅\"");
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
    serde_if_integer128! {
        test(12345123451234512345u128, r#""12345123451234512345""#);
        test(12345123451234512345i128, r#""12345123451234512345""#);
    }
    test(1.3f32, r#"1.3"#);
    test(1.3f64, r#"1.3"#);
    test(core::f32::NAN, r#"null"#);
    test(core::f64::NAN, r#"null"#);
}

#[test]
#[wasm_bindgen_test]
fn test_null() {
    test((), r#"null"#);
}
