#![allow(missing_docs)]

use jsonrpc::Message;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Unit;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Newtype([i32; 2]);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Tuple(i32, String);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Struct {
    x: i32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[allow(clippy::enum_variant_names)]
enum Enum {
    UnitVariant,
    NewtypeVariant([i32; 2]),
    TupleVariant(i32, String),
    StructVariant { x: i32 },
}

macro_rules! test_case {
    (
        $name:ident,
        $value:expr, {$($json:tt)*} $(,)?
    ) => {
        paste::paste! {
            #[test]
            fn [< test_serialize_ $name >] () {
                let input = $value;
                let expected_output = serde_json::json!({ $($json)* });

                let output = serde_json::to_value(&input).unwrap();
                assert_eq!(output, expected_output);
            }

            #[test]
            fn [< test_deserialize_ $name >] () {
                let input = serde_json::json!({ $($json)* });
                let expected_output = $value;

                let output = serde_json::from_value(input).unwrap();
                ensure_same_type(&output, &expected_output);

                assert_eq!(output.id, expected_output.id);
                assert_eq!(output.method, expected_output.method);
            }
        }
    }
}

test_case!(
    unit,
    Message { id: Some(1), method: Unit },
    {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "Unit",
    }
);

test_case!(
    newtype,
    Message { id: Some(1), method: Newtype([42, 43]) },
    {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "Newtype",
        "params": [42, 43],
    }
);

test_case!(
    tuple,
    Message { id: Some(1), method: Tuple(42, "foo".into()) },
    {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "Tuple",
        "params": [42, "foo"],
    }
);

test_case!(
    struct,
    Message { id: Some(1), method: Struct { x: 42 } },
    {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "Struct",
        "params": { "x": 42 },
    }
);

test_case!(
    unit_variant,
    Message { id: Some(1), method: Enum::UnitVariant },
    {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "UnitVariant",
    }
);

test_case!(
    newtype_variant,
    Message { id: Some(1), method: Enum::NewtypeVariant([42, 43]) },
    {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "NewtypeVariant",
        "params": [42, 43],
    }
);

test_case!(
    tuple_variant,
    Message { id: Some(1), method: Enum::TupleVariant(42, "foo".into()) },
    {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "TupleVariant",
        "params": [42, "foo"],
    }
);

test_case!(
    struct_variant,
    Message { id: Some(1), method: Enum::StructVariant { x: 42 } },
    {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "StructVariant",
        "params": { "x": 42 },
    }
);

test_case!(
    no_id,
    Message { id: None::<i64>, method: Unit },
    {
        "jsonrpc": "2.0",
        "method": "Unit",
    }
);

const fn ensure_same_type<T>(_: &T, _: &T) {}
