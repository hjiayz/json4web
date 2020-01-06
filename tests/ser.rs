#![no_std]
#[macro_use]
extern crate alloc;
#[macro_use]
extern crate wasm_bindgen_test;

use alloc::vec::Vec;
use json4web::ser::*;
use serde_derive::Serialize;

#[test]
#[wasm_bindgen_test]
fn test_struct() {
    #[derive(Serialize)]
    struct Test {
        int: u32,
        seq: Vec<&'static str>,
    }

    let test = Test {
        int: 1,
        seq: vec!["a", "b"],
    };
    let expected = r#"{"int":1,"seq":["a","b"]}"#;
    assert_eq!(to_string(&test).unwrap(), expected);
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
    assert_eq!(to_string(&u).unwrap(), expected);

    let n = E::Newtype(1);
    let expected = r#"{"Newtype":1}"#;
    assert_eq!(to_string(&n).unwrap(), expected);

    let t = E::Tuple(1, 2);
    let expected = r#"{"Tuple":[1,2]}"#;
    assert_eq!(to_string(&t).unwrap(), expected);

    let s = E::Struct { a: 1 };
    let expected = r#"{"Struct":{"a":1}}"#;
    assert_eq!(to_string(&s).unwrap(), expected);
}

#[test]
#[wasm_bindgen_test]
fn test_bytes() {
    use serde_bytes::Bytes;
    let bytes = &Bytes::new(b"bytes test");
    let expected = format!("\"{}\"", base64::encode_config(bytes, base64::URL_SAFE));
    assert_eq!(to_string(bytes).unwrap(), expected);
}

#[test]
#[wasm_bindgen_test]
fn test_bool() {
    let b = true;
    let expected = "1";
    assert_eq!(to_string(&b).unwrap(), expected);

    let b = false;
    let expected = "0";
    assert_eq!(to_string(&b).unwrap(), expected);
}
