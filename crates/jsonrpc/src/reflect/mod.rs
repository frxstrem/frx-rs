mod call_info;
mod expect_str;
mod extract_params;

use serde::ser;

pub use self::call_info::*;
pub use self::expect_str::*;
pub use self::extract_params::*;

mod macros {
    macro_rules! not_a_call_fallback {
        () => {
            type SerializeSeq = serde::ser::Impossible<Self::Ok, Self::Error>;
            type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;
            type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;

            fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
                $crate::reflect::not_a_call()
            }

            fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
                $crate::reflect::not_a_call()
            }

            fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
                $crate::reflect::not_a_call()
            }

            fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
                $crate::reflect::not_a_call()
            }

            fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
                $crate::reflect::not_a_call()
            }

            fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
                $crate::reflect::not_a_call()
            }

            fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
                $crate::reflect::not_a_call()
            }

            fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
                $crate::reflect::not_a_call()
            }

            fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
                $crate::reflect::not_a_call()
            }

            fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
                $crate::reflect::not_a_call()
            }

            fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
                $crate::reflect::not_a_call()
            }

            fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
                $crate::reflect::not_a_call()
            }

            fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
                $crate::reflect::not_a_call()
            }

            fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
                $crate::reflect::not_a_call()
            }

            fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
                $crate::reflect::not_a_call()
            }

            fn serialize_some<T: ?Sized + Serialize>(
                self,
                _value: &T,
            ) -> Result<Self::Ok, Self::Error> {
                $crate::reflect::not_a_call()
            }

            fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
                $crate::reflect::not_a_call()
            }

            fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
                $crate::reflect::not_a_call()
            }

            fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
                $crate::reflect::not_a_call()
            }

            fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
                $crate::reflect::not_a_call()
            }
        };
    }

    pub(super) use not_a_call_fallback;
}

fn not_a_call<T, E: ser::Error>() -> Result<T, E> {
    Err(E::custom("JSON-RPC method must be a struct or enum type"))
}
