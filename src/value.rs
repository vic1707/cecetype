use crate::flavors::{ValueFlavor, ser};
use ::{core::ops::Deref as _, derive_where::derive_where, serde::Serialize};

#[derive(Serialize)]
#[serde(bound(serialize = "F::Str: Serialize"))]
#[derive_where(Debug; )] // prevents compiler bounds check overflow & `F: Debug` bound
#[derive_where(PartialEq;)] // prevents compiler bounds check overflow & `F: PartialEq` bound
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

    UnitStruct {
        name: F::Str,
    },
    NewTypeStruct {
        name: F::Str,
        #[serde(serialize_with = "ser::serialize_ptr")]
        field: F::Ptr<Self>,
    },
    TupleStruct {
        name: F::Str,
        #[serde(serialize_with = "ser::serialize_list")]
        fields: F::List<Self>,
    },
    Struct {
        name: F::Str,
        #[serde(serialize_with = "ser::serialize_list")]
        fields: F::List<(F::Str, Self)>,
    },

    Enum {
        name: F::Str,
        variant: VariantValue<F>,
    },

    #[serde(serialize_with = "ser::serialize_opt_ptr")]
    Option(Option<F::Ptr<Value<F>>>),
}

#[derive(Serialize)]
#[serde(bound(serialize = "F::Str: Serialize"))]
#[derive_where(Debug; )] // prevents compiler bounds check overflow & `F: Debug` bound
#[derive_where(PartialEq; )] // prevents compiler bounds check overflow & `F: PartialEq` bound
pub enum VariantValue<F: ValueFlavor> {
    Unit {
        name: F::Str,
    },

    Tuple {
        name: F::Str,
        #[serde(serialize_with = "ser::serialize_list")]
        fields: F::List<Value<F>>,
    },

    NewType {
        name: F::Str,
        #[serde(serialize_with = "ser::serialize_ptr")]
        field: F::Ptr<Value<F>>,
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
            Value::Str(v) => write!(f, "\"{}\"", v.deref()),
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

            Value::UnitStruct { name } => write!(f, "{}", name.deref()),
            Value::NewTypeStruct { name, field } => {
                write!(f, "{}({})", name.deref(), field.deref())
            }
            Value::TupleStruct { name, fields } => {
                write!(f, "{} (", name.deref())?;
                for (i, v) in fields.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, ")")
            }
            Value::Struct { name, fields } => {
                write!(f, "{} {{ ", name.deref())?;
                for (i, (k, v)) in fields.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k.deref(), v)?;
                }
                write!(f, " }}")
            }

            Value::Enum { name, variant } => {
                write!(f, "{}::{}", name.deref(), variant)
            }

            Value::Option(value) => match value {
                Some(value) => write!(f, "Some({})", value.deref()),
                None => write!(f, "None"),
            },
        }
    }
}

impl<F> core::fmt::Display for VariantValue<F>
where
    F: ValueFlavor,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            VariantValue::Unit { name } => write!(f, "{}", name.deref()),
            VariantValue::Struct { name, fields } => {
                write!(f, "{}({{ ", name.deref())?;
                for (idx, field) in fields.deref().iter().enumerate() {
                    if idx != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", field.0.deref(), field.1)?;
                }
                write!(f, " }})")
            }
            VariantValue::Tuple { name, fields } => {
                write!(f, "{}(", name.deref())?;
                for (idx, field) in fields.deref().iter().enumerate() {
                    if idx != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{field}")?;
                }
                write!(f, ")")
            }
            VariantValue::NewType { name, field } => {
                write!(f, "{}({})", name.deref(), field.deref())
            }
        }
    }
}
