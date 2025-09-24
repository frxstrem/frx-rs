use core::marker::PhantomData;

use serde::{
    Serialize, Serializer,
    ser::{
        self, SerializeStruct, SerializeStructVariant, SerializeTupleStruct, SerializeTupleVariant,
    },
};

use super::macros::not_a_call_fallback;

pub struct CallInfo {
    pub name: &'static str,
    pub has_params: bool,
}

pub fn call_info<T: ?Sized + Serialize, E: ser::Error>(value: &T) -> Result<CallInfo, E> {
    T::serialize(
        value,
        CallInfoSerializer {
            _phantom: PhantomData,
        },
    )
}

struct CallInfoSerializer<E> {
    _phantom: PhantomData<fn() -> E>,
}

impl<E: ser::Error> Serializer for CallInfoSerializer<E> {
    type Ok = CallInfo;
    type Error = E;

    not_a_call_fallback!();

    type SerializeTupleStruct = GotCallInfo<E>;
    type SerializeTupleVariant = GotCallInfo<E>;
    type SerializeStruct = GotCallInfo<E>;
    type SerializeStructVariant = GotCallInfo<E>;

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(CallInfo {
            name,
            has_params: false,
        })
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(CallInfo {
            name: variant,
            has_params: false,
        })
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(CallInfo {
            name,
            has_params: true,
        })
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(CallInfo {
            name: variant,
            has_params: true,
        })
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(GotCallInfo {
            _phantom: PhantomData,
            call_info: CallInfo {
                name,
                has_params: true,
            },
        })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(GotCallInfo {
            _phantom: PhantomData,
            call_info: CallInfo {
                name: variant,
                has_params: true,
            },
        })
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(GotCallInfo {
            _phantom: PhantomData,
            call_info: CallInfo {
                name,
                has_params: true,
            },
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(GotCallInfo {
            _phantom: PhantomData,
            call_info: CallInfo {
                name: variant,
                has_params: true,
            },
        })
    }
}

pub struct GotCallInfo<E> {
    _phantom: PhantomData<fn() -> E>,
    call_info: CallInfo,
}

impl<E: ser::Error> SerializeTupleStruct for GotCallInfo<E> {
    type Ok = CallInfo;
    type Error = E;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<(), E> {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, E> {
        Ok(self.call_info)
    }
}

impl<E: ser::Error> SerializeTupleVariant for GotCallInfo<E> {
    type Ok = CallInfo;
    type Error = E;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<(), E> {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, E> {
        Ok(self.call_info)
    }
}

impl<E: ser::Error> SerializeStruct for GotCallInfo<E> {
    type Ok = CallInfo;
    type Error = E;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<(), E> {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, E> {
        Ok(self.call_info)
    }
}

impl<E: ser::Error> SerializeStructVariant for GotCallInfo<E> {
    type Ok = CallInfo;
    type Error = E;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<(), E> {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, E> {
        Ok(self.call_info)
    }
}
