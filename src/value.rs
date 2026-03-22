use crate::{ValueBuilder, flavors::ValueFlavor};
use ::{core::ops::Deref as _, derive_where::derive_where};

#[derive_where(Debug;)] // prevents compiler bounds check overflow & `F: Debug` bound
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

    Array(F::List<Self>),
    Slice(F::List<Self>),

    Tuple(F::List<Self>),

    UnitStruct {
        name: F::Str,
    },
    NewTypeStruct {
        name: F::Str,
        field: F::Ptr<Self>,
    },
    TupleStruct {
        name: F::Str,
        fields: F::List<Self>,
    },
    Struct {
        name: F::Str,
        fields: F::List<(F::Str, Self)>,
    },

    EnumUnit {
        name: F::Str,
        discriminant: u32,
        variant_name: F::Str,
    },

    EnumTuple {
        name: F::Str,
        discriminant: u32,
        variant_name: F::Str,
        fields: F::List<Self>,
    },

    EnumNewType {
        name: F::Str,
        discriminant: u32,
        variant_name: F::Str,
        field: F::Ptr<Self>,
    },

    EnumStruct {
        name: F::Str,
        discriminant: u32,
        variant_name: F::Str,
        fields: F::List<(F::Str, Self)>,
    },

    Option(Option<F::Ptr<Self>>),
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

            Value::EnumUnit {
                name,
                discriminant: _, // TODO: do we print that?
                variant_name,
            } => {
                write!(f, "{}::{}", name.deref(), variant_name.deref())
            }
            Value::EnumStruct {
                name,
                discriminant: _, // TODO: do we print that?
                variant_name,
                fields,
            } => {
                write!(f, "{}::{}({{ ", name.deref(), variant_name.deref())?;
                for (idx, field) in fields.deref().iter().enumerate() {
                    if idx != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", field.0.deref(), field.1)?;
                }
                write!(f, " }})")
            }
            Value::EnumTuple {
                name,
                discriminant: _, // TODO: do we print that?
                variant_name,
                fields,
            } => {
                write!(f, "{}::{}(", name.deref(), variant_name.deref())?;
                for (idx, field) in fields.deref().iter().enumerate() {
                    if idx != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{field}")?;
                }
                write!(f, ")")
            }
            Value::EnumNewType {
                name,
                discriminant: _, // TODO: do we print that?
                variant_name,
                field,
            } => {
                write!(
                    f,
                    "{}::{}({})",
                    name.deref(),
                    variant_name.deref(),
                    field.deref()
                )
            }

            Value::Option(value) => match value {
                Some(value) => write!(f, "Some({})", value.deref()),
                None => write!(f, "None"),
            },
        }
    }
}

impl<F> ::serde::Serialize for Value<F>
where
    F: ValueFlavor + ValueBuilder,
    F::Str: ::serde::Serialize,
{
    fn serialize<S: ::serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Value::Unit => serializer.serialize_unit(),

            Value::Bool(v) => v.serialize(serializer),

            Value::Str(v) => v.serialize(serializer),
            Value::Char(v) => v.serialize(serializer),

            Value::U8(v) => v.serialize(serializer),
            Value::U16(v) => v.serialize(serializer),
            Value::U32(v) => v.serialize(serializer),
            Value::U64(v) => v.serialize(serializer),

            Value::I8(v) => v.serialize(serializer),
            Value::I16(v) => v.serialize(serializer),
            Value::I32(v) => v.serialize(serializer),
            Value::I64(v) => v.serialize(serializer),

            Value::F32(v) => v.serialize(serializer),
            Value::F64(v) => v.serialize(serializer),

            Value::Slice(v) => v.serialize(serializer),

            Value::Array(values) | Value::Tuple(values) => {
                use serde::ser::SerializeTuple as _;

                let mut tup = serializer.serialize_tuple(values.len())?;
                for v in values.deref() {
                    tup.serialize_element(v)?;
                }
                tup.end()
            }

            Value::UnitStruct { name } => {
                serializer.serialize_unit_struct(F::make_static_str(name))
            }

            Value::NewTypeStruct { name, field } => {
                serializer.serialize_newtype_struct(F::make_static_str(name), field.deref())
            }

            Value::TupleStruct { name, fields } => {
                use serde::ser::SerializeTupleStruct as _;

                let mut ts =
                    serializer.serialize_tuple_struct(F::make_static_str(name), fields.len())?;
                for f in fields.deref() {
                    ts.serialize_field(f)?;
                }
                ts.end()
            }

            Value::Struct { name, fields } => {
                use serde::ser::SerializeStruct as _;

                let mut st = serializer.serialize_struct(F::make_static_str(name), fields.len())?;
                for (k, v) in fields.deref() {
                    st.serialize_field(F::make_static_str(k), v)?;
                }
                st.end()
            }

            Value::EnumUnit {
                name,
                discriminant,
                variant_name,
            } => serializer.serialize_unit_variant(
                F::make_static_str(name),
                *discriminant,
                F::make_static_str(variant_name),
            ),

            Value::EnumNewType {
                name,
                discriminant,
                variant_name,
                field,
            } => serializer.serialize_newtype_variant(
                F::make_static_str(name),
                *discriminant,
                F::make_static_str(variant_name),
                field.deref(),
            ),

            Value::EnumTuple {
                name,
                discriminant,
                variant_name,
                fields,
            } => {
                use ::serde::ser::SerializeTupleVariant as _;

                let mut tv = serializer.serialize_tuple_variant(
                    F::make_static_str(name),
                    *discriminant,
                    F::make_static_str(variant_name),
                    fields.len(),
                )?;
                for f in fields.deref() {
                    tv.serialize_field(f)?;
                }
                tv.end()
            }

            Value::EnumStruct {
                name,
                discriminant,
                variant_name,
                fields,
            } => {
                use ::serde::ser::SerializeStructVariant as _;

                let mut sv = serializer.serialize_struct_variant(
                    F::make_static_str(name),
                    *discriminant,
                    F::make_static_str(variant_name),
                    fields.len(),
                )?;
                for (k, v) in fields.deref() {
                    sv.serialize_field(F::make_static_str(k), v)?;
                }
                sv.end()
            }

            Value::Option(opt) => match opt {
                Some(v) => serializer.serialize_some(v.deref()),
                None => serializer.serialize_none(),
            },
        }
    }
}
