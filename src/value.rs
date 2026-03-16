use crate::flavors::{ValueFlavor, ser};
use ::serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(bound(serialize = "F::Str: Serialize"))]
pub enum Value<'v, F: ValueFlavor<'v>> {
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
    Array(F::List<Value<'v, F>>),
    #[serde(serialize_with = "ser::serialize_list")]
    Slice(F::List<Value<'v, F>>),

    #[serde(serialize_with = "ser::serialize_list")]
    Tuple(F::List<Value<'v, F>>),

    Struct {
        name: F::Str,
        #[serde(serialize_with = "ser::serialize_list")]
        fields: F::List<(F::Str, Value<'v, F>)>,
    },

    Enum {
        name: F::Str,
        #[serde(serialize_with = "ser::serialize_ptr")]
        variant: F::Ptr<VariantValue<'v, F>>,
    },
}

#[derive(Debug, Serialize)]
#[serde(bound(serialize = "F::Str: Serialize"))]
pub enum VariantValue<'v, F: ValueFlavor<'v>> {
    Unit {
        name: F::Str,
    },

    Tuple {
        name: F::Str,
        #[serde(serialize_with = "ser::serialize_list")]
        fields: F::List<Value<'v, F>>,
    },

    Struct {
        name: F::Str,
        #[serde(serialize_with = "ser::serialize_list")]
        fields: F::List<(F::Str, Value<'v, F>)>,
    },
}

impl<'s, F> core::fmt::Display for Value<'s, F>
where
    F: ValueFlavor<'s>,
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
                    write!(f, "{}", &**v)?;
                }
                write!(f, "]")
            }

            Value::Tuple(values) => {
                write!(f, "(")?;
                for (i, v) in values.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", &**v)?;
                }
                write!(f, ")")
            }

            Value::Struct { name, fields } => {
                write!(f, "{} {{ ", &**name)?;
                for (i, tuple) in fields.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    let (k, v) = &**tuple;
                    write!(f, "{}: {}", &**k, v)?;
                }
                write!(f, " }}")
            }

            Value::Enum { name, variant } => {
                write!(f, "{}::{}", &**name, &**variant)
            }
        }
    }
}

impl<'s, F> core::fmt::Display for VariantValue<'s, F>
where
    F: ValueFlavor<'s>,
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
                    write!(f, "{}", field.deref())?;
                }
                write!(f, ")")
            }
        }
    }
}
