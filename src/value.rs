use crate::{ValueBuilder, flavors::ValueFlavor};
use ::{
    core::{fmt, ops::Deref as _},
    derive_where::derive_where,
};

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

impl<F> fmt::Display for Value<F>
where
    F: ValueFlavor,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unit => write!(f, "()"),
            Self::Bool(val) => write!(f, "{val}"),
            Self::Str(val) => write!(f, "\"{}\"", &**val),
            Self::Char(val) => write!(f, "'{val}'"),

            Self::U8(val) => write!(f, "{val}"),
            Self::U16(val) => write!(f, "{val}"),
            Self::U32(val) => write!(f, "{val}"),
            Self::U64(val) => write!(f, "{val}"),
            Self::I8(val) => write!(f, "{val}"),
            Self::I16(val) => write!(f, "{val}"),
            Self::I32(val) => write!(f, "{val}"),
            Self::I64(val) => write!(f, "{val}"),

            Self::F32(val) => write!(f, "{val}"),
            Self::F64(val) => write!(f, "{val}"),

            Self::Array(values) | Self::Slice(values) => {
                write!(f, "[")?;
                for (i, val) in values.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{val}")?;
                }
                write!(f, "]")
            }

            Self::Tuple(values) => {
                write!(f, "(")?;
                for (i, val) in values.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{val}")?;
                }
                write!(f, ")")
            }

            Self::UnitStruct { name } => write!(f, "{}", &**name),
            Self::NewTypeStruct { name, field } => {
                write!(f, "{}({})", &**name, &**field)
            }
            Self::TupleStruct { name, fields } => {
                write!(f, "{} (", &**name)?;
                for (i, val) in fields.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{val}")?;
                }
                write!(f, ")")
            }
            Self::Struct { name, fields } => {
                write!(f, "{} {{ ", &**name)?;
                for (i, (key, val)) in fields.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", &**key, val)?;
                }
                write!(f, " }}")
            }

            Self::EnumUnit {
                name, variant_name, ..
            } => {
                write!(f, "{}::{}", &**name, &**variant_name)
            }
            Self::EnumStruct {
                name,
                variant_name,
                fields,
                ..
            } => {
                write!(f, "{}::{}({{ ", &**name, &**variant_name)?;
                for (idx, field) in fields.deref().iter().enumerate() {
                    if idx != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", &*field.0, field.1)?;
                }
                write!(f, " }})")
            }
            Self::EnumTuple {
                name,
                variant_name,
                fields,
                ..
            } => {
                write!(f, "{}::{}(", &**name, &**variant_name)?;
                for (idx, field) in fields.deref().iter().enumerate() {
                    if idx != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{field}")?;
                }
                write!(f, ")")
            }
            Self::EnumNewType {
                name,
                variant_name,
                field,
                ..
            } => {
                write!(f, "{}::{}({})", &**name, &**variant_name, &**field)
            }

            Self::Option(opt) => match opt {
                Some(value) => write!(f, "Some({})", &**value),
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
    #[inline]
    fn serialize<S: ::serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Unit => serializer.serialize_unit(),

            Self::Bool(val) => val.serialize(serializer),

            Self::Str(val) => serializer.serialize_str(val),
            Self::Char(val) => val.serialize(serializer),

            Self::U8(val) => val.serialize(serializer),
            Self::U16(val) => val.serialize(serializer),
            Self::U32(val) => val.serialize(serializer),
            Self::U64(val) => val.serialize(serializer),

            Self::I8(val) => val.serialize(serializer),
            Self::I16(val) => val.serialize(serializer),
            Self::I32(val) => val.serialize(serializer),
            Self::I64(val) => val.serialize(serializer),

            Self::F32(val) => val.serialize(serializer),
            Self::F64(val) => val.serialize(serializer),

            Self::Slice(val) => val.serialize(serializer),

            Self::Array(values) | Self::Tuple(values) => {
                use ::serde::ser::SerializeTuple as _;

                let mut tup = serializer.serialize_tuple(values.len())?;
                for val in &**values {
                    tup.serialize_element(val)?;
                }
                tup.end()
            }

            Self::UnitStruct { name } => serializer.serialize_unit_struct(F::make_static_str(name)),

            Self::NewTypeStruct { name, field } => {
                serializer.serialize_newtype_struct(F::make_static_str(name), &**field)
            }

            Self::TupleStruct { name, fields } => {
                use ::serde::ser::SerializeTupleStruct as _;

                let mut ts =
                    serializer.serialize_tuple_struct(F::make_static_str(name), fields.len())?;
                for field in &**fields {
                    ts.serialize_field(field)?;
                }
                ts.end()
            }

            Self::Struct { name, fields } => {
                use ::serde::ser::SerializeStruct as _;

                let mut st = serializer.serialize_struct(F::make_static_str(name), fields.len())?;
                for (key, val) in &**fields {
                    st.serialize_field(F::make_static_str(key), val)?;
                }
                st.end()
            }

            Self::EnumUnit {
                name,
                discriminant,
                variant_name,
            } => serializer.serialize_unit_variant(
                F::make_static_str(name),
                *discriminant,
                F::make_static_str(variant_name),
            ),

            Self::EnumNewType {
                name,
                discriminant,
                variant_name,
                field,
            } => serializer.serialize_newtype_variant(
                F::make_static_str(name),
                *discriminant,
                F::make_static_str(variant_name),
                &**field,
            ),

            Self::EnumTuple {
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
                for field in &**fields {
                    tv.serialize_field(field)?;
                }
                tv.end()
            }

            Self::EnumStruct {
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
                for (key, val) in &**fields {
                    sv.serialize_field(F::make_static_str(key), val)?;
                }
                sv.end()
            }

            Self::Option(opt) => match opt {
                Some(val) => serializer.serialize_some(&**val),
                None => serializer.serialize_none(),
            },
        }
    }
}
