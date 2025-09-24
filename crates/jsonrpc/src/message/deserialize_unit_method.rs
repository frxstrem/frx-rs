use core::marker::PhantomData;

use alloc::borrow::Cow;
use serde::{
    Deserializer,
    de::{
        self, DeserializeSeed, EnumAccess, Unexpected, VariantAccess, Visitor,
        value::CowStrDeserializer,
    },
    forward_to_deserialize_any,
};

pub(super) struct MethodOnlyDeserializer<'de, E> {
    pub(super) _phantom: PhantomData<fn() -> E>,
    pub(super) method: Cow<'de, str>,
}

impl<'de, E: de::Error> Deserializer<'de> for MethodOnlyDeserializer<'de, E> {
    type Error = E;

    fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, E> {
        Err(E::invalid_value(
            Unexpected::Other("non-unit value"),
            &"a unit value",
        ))
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, E> {
        match self.method {
            Cow::Borrowed(method) => visitor.visit_borrowed_str(method),
            Cow::Owned(method) => visitor.visit_string(method),
        }
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, E> {
        visitor.visit_unit()
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, E> {
        visitor.visit_enum(MethodOnlyEnumAccess {
            _phantom: PhantomData,
            method: self.method,
        })
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char string bytes byte_buf option unit
        newtype_struct seq tuple tuple_struct map struct identifier ignored_any
    }
}

struct MethodOnlyEnumAccess<'de, E> {
    _phantom: PhantomData<fn() -> E>,
    method: Cow<'de, str>,
}

impl<'de, E: de::Error> EnumAccess<'de> for MethodOnlyEnumAccess<'de, E> {
    type Error = E;
    type Variant = MethodOnlyVariantAccess<E>;

    fn variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), E>
where {
        let value = seed.deserialize(CowStrDeserializer::new(self.method))?;
        Ok((
            value,
            MethodOnlyVariantAccess {
                _phantom: PhantomData,
            },
        ))
    }
}

pub struct MethodOnlyVariantAccess<E> {
    _phantom: PhantomData<fn() -> E>,
}

impl<'de, E: de::Error> VariantAccess<'de> for MethodOnlyVariantAccess<E> {
    type Error = E;

    fn unit_variant(self) -> Result<(), E> {
        Ok(())
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, _seed: T) -> Result<T::Value, E> {
        Err(E::invalid_type(Unexpected::UnitVariant, &"newtype variant"))
    }

    fn tuple_variant<V: Visitor<'de>>(self, _len: usize, _visitor: V) -> Result<V::Value, E> {
        Err(E::invalid_type(Unexpected::UnitVariant, &"tuple variant"))
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, E> {
        Err(E::invalid_type(Unexpected::UnitVariant, &"struct variant"))
    }
}
