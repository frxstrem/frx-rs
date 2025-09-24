use core::{fmt, marker::PhantomData};

use alloc::borrow::Cow;
use serde::{
    Deserialize, Deserializer,
    de::{
        self, DeserializeSeed, EnumAccess, VariantAccess, Visitor, value::BorrowedStrDeserializer,
    },
    forward_to_deserialize_any,
};

pub(super) struct MethodDeserializeSeed<'de, T> {
    pub(super) _phantom: PhantomData<fn() -> T>,
    pub(super) method: Cow<'de, str>,
}

impl<'de, T: Deserialize<'de>> DeserializeSeed<'de> for MethodDeserializeSeed<'de, T> {
    type Value = T;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        T::deserialize(MethodDeserializer {
            method_name: self.method,
            deserializer,
        })
    }
}

pub(super) struct MethodDeserializer<'de, D> {
    pub(super) method_name: Cow<'de, str>,
    pub(super) deserializer: D,
}

impl<'de, D: Deserializer<'de>> Deserializer<'de> for MethodDeserializer<'de, D> {
    type Error = D::Error;

    fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(de::Error::custom(
            "can only deserialize JSON-RPC method as struct or enum",
        ))
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserializer.deserialize_unit_struct(name, visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserializer.deserialize_newtype_struct(name, visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserializer
            .deserialize_tuple_struct(name, len, visitor)
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserializer.deserialize_struct(name, fields, visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        let method_name = variants
            .iter()
            .copied()
            .find(|v| v == &self.method_name)
            .ok_or_else(|| de::Error::unknown_variant(&self.method_name, variants))?;

        visitor.visit_enum(MethodEnumAccess {
            method_name,
            deserializer: self.deserializer,
        })
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf option unit
        seq tuple map identifier ignored_any
    }
}

struct MethodEnumAccess<D> {
    method_name: &'static str,
    deserializer: D,
}

impl<'de, D: Deserializer<'de>> EnumAccess<'de> for MethodEnumAccess<D> {
    type Error = D::Error;
    type Variant = MethodVariantAccess<D>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = seed.deserialize(BorrowedStrDeserializer::new(self.method_name))?;

        Ok((
            value,
            MethodVariantAccess {
                method_name: self.method_name,
                deserializer: self.deserializer,
            },
        ))
    }
}

struct MethodVariantAccess<D> {
    method_name: &'static str,
    deserializer: D,
}

impl<'de, D: Deserializer<'de>> VariantAccess<'de> for MethodVariantAccess<D> {
    type Error = D::Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        struct UnitVisitor;

        impl<'de> Visitor<'de> for UnitVisitor {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a unit value")
            }

            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(())
            }
        }

        self.deserializer
            .deserialize_unit_struct(self.method_name, UnitVisitor)
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(
        self,
        seed: T,
    ) -> Result<T::Value, Self::Error> {
        seed.deserialize(self.deserializer)
    }

    fn tuple_variant<V: Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserializer
            .deserialize_tuple_struct(self.method_name, len, visitor)
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserializer
            .deserialize_struct(self.method_name, fields, visitor)
    }
}
