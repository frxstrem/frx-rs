use core::fmt;

use serde::{
    Deserializer,
    de::{self, DeserializeSeed, Unexpected, Visitor},
};

pub struct ExpectStr(pub &'static str);

impl<'de> DeserializeSeed<'de> for ExpectStr {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_str(ExpectStrVisitor(self.0))
    }
}

struct ExpectStrVisitor(&'static str);

impl<'de> Visitor<'de> for ExpectStrVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "the string {:?}", self.0)
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<(), E> {
        if v == self.0 {
            Ok(())
        } else {
            Err(E::invalid_value(Unexpected::Str(v), &self))
        }
    }
}
