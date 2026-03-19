use crate::flavors::{ValueFlavor, ser};
use ::serde::Serialize;

#[derive(Serialize)]
#[serde(bound(serialize = "F::Str: Serialize"))]
pub enum Value<F: ValueFlavor> {
    Unit,

    Bool(bool),

    Str(F::Str),
    Char(char),

    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),

    #[serde(serialize_with = "ser::serialize_list")]
    Array(F::List<Self>),
    #[serde(serialize_with = "ser::serialize_list")]
    Slice(F::List<Self>),

    #[serde(serialize_with = "ser::serialize_list")]
    Tuple(F::List<Self>),

    Struct {
        name: F::Str,
        #[serde(serialize_with = "ser::serialize_list")]
        fields: F::List<(F::Str, Self)>,
    },

    Enum {
        name: F::Str,
        variant: VariantValue<F>,
    },
}

#[derive(Serialize)]
#[serde(bound(serialize = "F::Str: Serialize"))]
pub enum VariantValue<F: ValueFlavor> {
    Unit {
        name: F::Str,
    },

    Tuple {
        name: F::Str,
        #[serde(serialize_with = "ser::serialize_list")]
        fields: F::List<Value<F>>,
    },

    Struct {
        name: F::Str,
        #[serde(serialize_with = "ser::serialize_list")]
        fields: F::List<(F::Str, Value<F>)>,
    },
}

impl<F> core::fmt::Display for Value<F>
where
    F: ValueFlavor,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Value::Unit => write!(f, "()"),
            Value::Bool(v) => write!(f, "{v}"),
            Value::Str(v) => write!(f, "\"{}\"", &**v),
            Value::Char(v) => write!(f, "'{v}'"),

            Value::U8(v) => write!(f, "{v}"),
            Value::U16(v) => write!(f, "{v}"),
            Value::U32(v) => write!(f, "{v}"),
            Value::U64(v) => write!(f, "{v}"),
            Value::I8(v) => write!(f, "{v}"),
            Value::I16(v) => write!(f, "{v}"),
            Value::I32(v) => write!(f, "{v}"),
            Value::I64(v) => write!(f, "{v}"),

            Value::F32(v) => write!(f, "{v}"),
            Value::F64(v) => write!(f, "{v}"),

            Value::Array(values) | Value::Slice(values) => {
                write!(f, "[")?;
                for (i, v) in values.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }

            Value::Tuple(values) => {
                write!(f, "(")?;
                for (i, v) in values.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, ")")
            }

            Value::Struct { name, fields } => {
                write!(f, "{} {{ ", &**name)?;
                for (i, (k, v)) in fields.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", &**k, v)?;
                }
                write!(f, " }}")
            }

            Value::Enum { name, variant } => {
                write!(f, "{}::{}", &**name, variant)
            }
        }
    }
}

impl<F> core::fmt::Display for VariantValue<F>
where
    F: ValueFlavor,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::ops::Deref as _;

        match self {
            VariantValue::Unit { name } => write!(f, "{}", &**name),
            VariantValue::Struct { name, fields } => {
                write!(f, "{}({{ ", &**name)?;
                for (idx, field) in fields.deref().iter().enumerate() {
                    if idx != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", &*field.0, field.1)?;
                }
                write!(f, " }})")
            }
            VariantValue::Tuple { name, fields } => {
                write!(f, "{}(", &**name)?;
                for (idx, field) in fields.deref().iter().enumerate() {
                    if idx != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{field}")?;
                }
                write!(f, ")")
            }
        }
    }
}

impl<F: ValueFlavor> ::core::cmp::PartialEq for VariantValue<F> {
    #[inline]
    fn eq(&self, other: &VariantValue<F>) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (VariantValue::Unit { name: __self_0 }, VariantValue::Unit { name: __arg1_0 }) => {
                    __self_0 == __arg1_0
                }
                (
                    VariantValue::Tuple {
                        name: __self_0,
                        fields: __self_1,
                    },
                    VariantValue::Tuple {
                        name: __arg1_0,
                        fields: __arg1_1,
                    },
                ) => __self_0 == __arg1_0 && __self_1 == __arg1_1,
                (
                    VariantValue::Struct {
                        name: __self_0,
                        fields: __self_1,
                    },
                    VariantValue::Struct {
                        name: __arg1_0,
                        fields: __arg1_1,
                    },
                ) => __self_0 == __arg1_0 && __self_1 == __arg1_1,
                _ => unsafe { ::core::intrinsics::unreachable() },
            }
    }
}

impl<F: ValueFlavor> ::core::cmp::PartialEq for Value<F> {
    #[inline]
    fn eq(&self, other: &Value<F>) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
            && match (self, other) {
                (Value::Bool(__self_0), Value::Bool(__arg1_0)) => __self_0 == __arg1_0,
                (Value::Str(__self_0), Value::Str(__arg1_0)) => __self_0 == __arg1_0,
                (Value::Char(__self_0), Value::Char(__arg1_0)) => __self_0 == __arg1_0,
                (Value::U8(__self_0), Value::U8(__arg1_0)) => __self_0 == __arg1_0,
                (Value::U16(__self_0), Value::U16(__arg1_0)) => __self_0 == __arg1_0,
                (Value::U32(__self_0), Value::U32(__arg1_0)) => __self_0 == __arg1_0,
                (Value::U64(__self_0), Value::U64(__arg1_0)) => __self_0 == __arg1_0,
                (Value::I8(__self_0), Value::I8(__arg1_0)) => __self_0 == __arg1_0,
                (Value::I16(__self_0), Value::I16(__arg1_0)) => __self_0 == __arg1_0,
                (Value::I32(__self_0), Value::I32(__arg1_0)) => __self_0 == __arg1_0,
                (Value::I64(__self_0), Value::I64(__arg1_0)) => __self_0 == __arg1_0,
                (Value::F32(__self_0), Value::F32(__arg1_0)) => __self_0 == __arg1_0,
                (Value::F64(__self_0), Value::F64(__arg1_0)) => __self_0 == __arg1_0,
                (Value::Array(__self_0), Value::Array(__arg1_0)) => __self_0 == __arg1_0,
                (Value::Slice(__self_0), Value::Slice(__arg1_0)) => __self_0 == __arg1_0,
                (Value::Tuple(__self_0), Value::Tuple(__arg1_0)) => __self_0 == __arg1_0,
                (
                    Value::Struct {
                        name: __self_0,
                        fields: __self_1,
                    },
                    Value::Struct {
                        name: __arg1_0,
                        fields: __arg1_1,
                    },
                ) => __self_0 == __arg1_0 && __self_1 == __arg1_1,
                (
                    Value::Enum {
                        name: __self_0,
                        variant: __self_1,
                    },
                    Value::Enum {
                        name: __arg1_0,
                        variant: __arg1_1,
                    },
                ) => __self_0 == __arg1_0 && __self_1 == __arg1_1,
                _ => true,
            }
    }
}
impl<F: ::core::fmt::Debug + ValueFlavor> ::core::fmt::Debug for Value<F> {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            Value::Unit => ::core::fmt::Formatter::write_str(f, "Unit"),
            Value::Bool(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Bool", &__self_0)
            }
            Value::Str(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Str", &__self_0)
            }
            Value::Char(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Char", &__self_0)
            }
            Value::U8(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "U8", &__self_0)
            }
            Value::U16(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "U16", &__self_0)
            }
            Value::U32(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "U32", &__self_0)
            }
            Value::U64(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "U64", &__self_0)
            }
            Value::I8(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "I8", &__self_0)
            }
            Value::I16(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "I16", &__self_0)
            }
            Value::I32(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "I32", &__self_0)
            }
            Value::I64(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "I64", &__self_0)
            }
            Value::F32(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "F32", &__self_0)
            }
            Value::F64(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "F64", &__self_0)
            }
            Value::Array(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Array", &__self_0)
            }
            Value::Slice(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Slice", &__self_0)
            }
            Value::Tuple(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Tuple", &__self_0)
            }
            Value::Struct {
                name: __self_0,
                fields: __self_1,
            } => ::core::fmt::Formatter::debug_struct_field2_finish(
                f, "Struct", "name", __self_0, "fields", &__self_1,
            ),
            Value::Enum {
                name: __self_0,
                variant: __self_1,
            } => ::core::fmt::Formatter::debug_struct_field2_finish(
                f, "Enum", "name", __self_0, "variant", &__self_1,
            ),
        }
    }
}
impl<F: ValueFlavor> ::core::fmt::Debug for VariantValue<F> {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            VariantValue::Unit { name: __self_0 } => {
                ::core::fmt::Formatter::debug_struct_field1_finish(f, "Unit", "name", &__self_0)
            }
            VariantValue::Tuple {
                name: __self_0,
                fields: __self_1,
            } => ::core::fmt::Formatter::debug_struct_field2_finish(
                f, "Tuple", "name", __self_0, "fields", &__self_1,
            ),
            VariantValue::Struct {
                name: __self_0,
                fields: __self_1,
            } => ::core::fmt::Formatter::debug_struct_field2_finish(
                f, "Struct", "name", __self_0, "fields", &__self_1,
            ),
        }
    }
}
