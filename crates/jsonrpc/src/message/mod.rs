mod deserialize_method;
mod deserialize_unit_method;

use core::{fmt, marker::PhantomData};

use alloc::borrow::Cow;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, MapAccess, Visitor},
    ser::SerializeStruct,
};

use crate::reflect::{ExpectStr, ExtractParams, call_info};

use self::{
    deserialize_method::{MethodDeserializeSeed, MethodDeserializer},
    deserialize_unit_method::MethodOnlyDeserializer,
};

/// A plain JSON-RPC message.
///
/// # Serialization
///
/// During serialization and deserialization, the `method` field will serialize
/// to both the `method` and `params` fields. If `T` is a struct, then `method`
/// will be the serialized struct name and `params` will be the serialized struct.
/// If `T` is an enum, then `method` will be the serialized variant name and `params`
/// will be the serialized variant. If `method` contains a unit type or variant,
/// then `params` is omitted.
///
/// If `T` is any type other than a struct (regular struct, unit struct, newtype
/// struct or tuple struct) or enum, serialization will fail. Untagged enums and
/// adjacently tagged enums are also not supported.
///
/// # Examples
///
/// ```
/// # use serde::{Serialize, Deserialize};
/// # use jsonrpc::Message;
/// #[derive(Serialize, Deserialize)]
/// #[serde(rename_all = "snake_case")]
/// enum Request {
///     Add { x: i32, y: i32 }
/// }
///
/// assert_eq!(
///     serde_json::to_string(&Message { id: Some(123), method: Request::Add { x: 4, y: 5 } }).unwrap(),
///     r#"{"jsonrpc":"2.0","id":123,"method":"add","params":{"x":4,"y":5}}"#,
/// );
/// ```
#[derive(Debug)]
pub struct Message<T: ?Sized, I> {
    /// The ID of the JSON-RPC message.
    ///
    /// If this is `None`, then the `id` field is not serialized.
    /// To support `null` values as well as absent values, let `I = Option<_>`.
    pub id: Option<I>,

    /// The method and params of the JSON-RPC message.
    ///
    /// See the [type-level docs](Message) for how this field is serialized.
    pub method: T,
}

impl<T: ?Sized + Serialize, I: Serialize> Serialize for Message<T, I> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let call_info = call_info(&self.method)?;

        let mut s = serializer.serialize_struct(
            "JsonRpc",
            2 + if self.id.is_some() { 1 } else { 0 } + if call_info.has_params { 1 } else { 0 },
        )?;

        s.serialize_field("jsonrpc", "2.0")?;
        if let Some(id) = &self.id {
            s.serialize_field("id", id)?;
        } else {
            s.skip_field("id")?;
        }
        s.serialize_field("method", call_info.name)?;
        if call_info.has_params {
            s.serialize_field("params", &ExtractParams(&self.method))?;
        } else {
            s.skip_field("params")?;
        }

        s.end()
    }
}

impl<'de, T: Deserialize<'de>, I: Deserialize<'de>> Deserialize<'de> for Message<T, I> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_struct(
            "JsonRpc",
            &["jsonrpc", "id", "method", "params"],
            RawMessageVisitor {
                _phantom: PhantomData,
            },
        )
    }
}

struct RawMessageVisitor<T, I> {
    _phantom: PhantomData<fn() -> Message<T, I>>,
}

impl<'de, T: Deserialize<'de>, I: Deserialize<'de>> Visitor<'de> for RawMessageVisitor<T, I> {
    type Value = Message<T, I>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a JSON-RPC message")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        #[derive(Deserialize)]
        enum Field {
            #[serde(rename = "jsonrpc")]
            JsonRpc,
            #[serde(rename = "id")]
            Id,
            #[serde(rename = "method")]
            Method,
            #[serde(rename = "params")]
            Params,
        }

        let mut has_jsonrpc = false;
        let mut id = None::<I>;

        let mut call = PartialCall::<T>::Empty;

        while let Some(field) = map.next_key::<Field>()? {
            match field {
                Field::JsonRpc => {
                    if has_jsonrpc {
                        return Err(de::Error::duplicate_field("jsonrpc"));
                    }
                    has_jsonrpc = true;

                    map.next_value_seed(ExpectStr("2.0"))?
                }

                Field::Id => {
                    if id.is_some() {
                        return Err(de::Error::duplicate_field("id"));
                    }

                    id = Some(map.next_value()?);
                }

                Field::Method => {
                    call = call.with_method(&mut map)?;
                }
                Field::Params => {
                    call = call.with_params(&mut map)?;
                }
            }
        }

        if !has_jsonrpc {
            return Err(de::Error::missing_field("jsonrpc"));
        }

        let call = call.finish()?;

        Ok(Message { id, method: call })
    }
}

enum PartialCall<'de, T> {
    Empty,
    MethodOnly { method: Cow<'de, str> },
    ParamsOnly { params: serde_json::Value },
    Call(T),
}

impl<'de, T: Deserialize<'de>> PartialCall<'de, T> {
    fn finish<E: de::Error>(self) -> Result<T, E> {
        match self {
            Self::MethodOnly { method } => T::deserialize(MethodOnlyDeserializer {
                _phantom: PhantomData,
                method,
            }),

            Self::Call(call) => Ok(call),

            Self::Empty | Self::ParamsOnly { .. } => Err(E::missing_field("method")),
        }
    }

    fn with_method<A: MapAccess<'de>>(self, map: &mut A) -> Result<Self, A::Error> {
        match self {
            Self::Empty => {
                let method = map.next_value()?;
                Ok(Self::MethodOnly { method })
            }

            Self::ParamsOnly { params } => {
                let method: Cow<'de, str> = map.next_value()?;

                let call = T::deserialize(MethodDeserializer {
                    method_name: method,
                    deserializer: params,
                })
                .map_err(de::Error::custom)?;

                Ok(Self::Call(call))
            }

            Self::MethodOnly { .. } | Self::Call { .. } => {
                Err(de::Error::duplicate_field("method"))
            }
        }
    }

    fn with_params<A: MapAccess<'de>>(self, map: &mut A) -> Result<Self, A::Error>
    where
        T: Deserialize<'de>,
    {
        match self {
            Self::Empty => {
                let params = map.next_value()?;
                Ok(Self::ParamsOnly { params })
            }

            Self::MethodOnly { method } => {
                let call = map.next_value_seed(MethodDeserializeSeed {
                    _phantom: PhantomData,
                    method,
                })?;

                Ok(Self::Call(call))
            }

            Self::ParamsOnly { .. } | Self::Call { .. } => {
                Err(de::Error::duplicate_field("params"))
            }
        }
    }
}
