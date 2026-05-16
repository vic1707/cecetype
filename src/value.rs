use crate::{flavors::ValueFlavor, utils::as_static_str};
use ::core::fmt;

/// Struct/enum value data.
#[::derive_where::derive_where(Clone, Debug, PartialEq;)] // prevents compiler bounds check overflow & `VF` bounds
#[non_exhaustive]
pub enum Data<VF: ValueFlavor> {
    Unit,
    NewType {
        field: VF::Ptr<Value<VF>>,
    },
    Tuple {
        fields: VF::List<Value<VF>>,
    },
    Struct {
        fields: VF::List<(VF::Str, Value<VF>)>,
    },
}

/// Runtime value representation. Mirrors [`Schema`](crate::schema::Schema) variants with actual data.
#[::derive_where::derive_where(Clone, Debug, PartialEq;)] // prevents compiler bounds check overflow & `VF` bounds
#[non_exhaustive]
pub enum Value<VF: ValueFlavor> {
    Unit,

    Bool(bool),

    Str(VF::Str),
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
    U128(u128),
    I128(i128),

    Array(VF::List<Self>),
    Slice(VF::List<Self>),
    Map(VF::List<(Self, Self)>),

    Tuple(VF::List<Self>),

    Struct {
        name: VF::Str,
        data: Data<VF>,
    },

    Enum {
        enum_name: VF::Str,
        variant_name: VF::Str,
        discriminant: u32,
        data: Data<VF>,
    },

    Option(Option<VF::Ptr<Self>>),
}

impl<VF> fmt::Display for Value<VF>
where
    VF: ValueFlavor,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unit => write!(f, "()"),
            Self::Bool(val) => write!(f, "{val}"),
            Self::Str(val) => write!(f, "\"{}\"", val.as_ref()),
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
            Self::U128(val) => write!(f, "{val}"),
            Self::I128(val) => write!(f, "{val}"),

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

            Self::Map(entries) => {
                write!(f, "{{")?;
                for (i, (key, val)) in entries.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{key}: {val}")?;
                }
                write!(f, "}}")
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

            Self::Struct { name, data } => match data {
                Data::Unit => write!(f, "{}", name.as_ref()),
                Data::NewType { field } => write!(f, "{} ({})", name.as_ref(), &**field),
                Data::Tuple { fields } => {
                    write!(f, "{} (", name.as_ref())?;
                    for (i, val) in fields.iter().enumerate() {
                        if i != 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{val}")?;
                    }
                    write!(f, ")")
                }
                Data::Struct { fields } => {
                    write!(f, "{} {{ ", name.as_ref())?;
                    for (i, (key, val)) in fields.iter().enumerate() {
                        if i != 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}: {}", key.as_ref(), val)?;
                    }
                    write!(f, " }}")
                }
            },

            Self::Enum {
                enum_name,
                variant_name,
                data,
                ..
            } => {
                write!(f, "{}::", enum_name.as_ref())?;
                match data {
                    Data::Unit => write!(f, "{}", variant_name.as_ref()),
                    Data::NewType { field } => write!(f, "{}({})", variant_name.as_ref(), &**field),
                    Data::Tuple { fields } => {
                        write!(f, "{}(", variant_name.as_ref())?;
                        for (i, val) in fields.iter().enumerate() {
                            if i != 0 {
                                write!(f, ", ")?;
                            }
                            write!(f, "{val}")?;
                        }
                        write!(f, ")")
                    }
                    Data::Struct { fields } => {
                        write!(f, "{{ ")?;
                        for (i, (key, val)) in fields.iter().enumerate() {
                            if i != 0 {
                                write!(f, ", ")?;
                            }
                            write!(f, "{}: {}", key.as_ref(), val)?;
                        }
                        write!(f, " }}")
                    }
                }
            }

            Self::Option(opt) => match opt {
                Some(value) => write!(f, "Some({})", &**value),
                None => write!(f, "None"),
            },
        }
    }
}

impl<VF> ::serde::Serialize for Value<VF>
where
    VF: ValueFlavor,
    VF::Str: ::serde::Serialize,
{
    #[inline]
    fn serialize<S: ::serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Unit => serializer.serialize_unit(),

            Self::Bool(val) => val.serialize(serializer),

            Self::Str(val) => serializer.serialize_str(val.as_ref()),
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

            Self::U128(val) => val.serialize(serializer),
            Self::I128(val) => val.serialize(serializer),

            Self::Slice(val) => val.serialize(serializer),

            Self::Map(entries) => {
                use ::serde::ser::SerializeMap as _;

                let mut map = serializer.serialize_map(Some(entries.len()))?;
                for (key, val) in &**entries {
                    map.serialize_entry(key, val)?;
                }
                map.end()
            }

            Self::Array(values) | Self::Tuple(values) => {
                use ::serde::ser::SerializeTuple as _;

                let mut tup = serializer.serialize_tuple(values.len())?;
                for val in &**values {
                    tup.serialize_element(val)?;
                }
                tup.end()
            }

            Self::Struct { name, data } => match data {
                Data::Unit => serializer.serialize_unit_struct(as_static_str(name)),

                Data::NewType { field } => {
                    serializer.serialize_newtype_struct(as_static_str(name), &**field)
                }

                Data::Tuple { fields } => {
                    use ::serde::ser::SerializeTupleStruct as _;

                    let mut ts =
                        serializer.serialize_tuple_struct(as_static_str(name), fields.len())?;
                    for field in &**fields {
                        ts.serialize_field(field)?;
                    }
                    ts.end()
                }

                Data::Struct { fields } => {
                    use ::serde::ser::SerializeStruct as _;

                    let mut st = serializer.serialize_struct(as_static_str(name), fields.len())?;
                    for (key, val) in &**fields {
                        st.serialize_field(as_static_str(key), val)?;
                    }
                    st.end()
                }
            },

            Self::Enum {
                enum_name,
                variant_name,
                discriminant,
                data,
            } => match data {
                Data::Unit => serializer.serialize_unit_variant(
                    as_static_str(enum_name),
                    *discriminant,
                    as_static_str(variant_name),
                ),

                Data::NewType { field } => serializer.serialize_newtype_variant(
                    as_static_str(enum_name),
                    *discriminant,
                    as_static_str(variant_name),
                    &**field,
                ),

                Data::Tuple { fields } => {
                    use ::serde::ser::SerializeTupleVariant as _;

                    let mut tv = serializer.serialize_tuple_variant(
                        as_static_str(enum_name),
                        *discriminant,
                        as_static_str(variant_name),
                        fields.len(),
                    )?;
                    for field in &**fields {
                        tv.serialize_field(field)?;
                    }
                    tv.end()
                }

                Data::Struct { fields } => {
                    use ::serde::ser::SerializeStructVariant as _;

                    let mut sv = serializer.serialize_struct_variant(
                        as_static_str(enum_name),
                        *discriminant,
                        as_static_str(variant_name),
                        fields.len(),
                    )?;
                    for (key, val) in &**fields {
                        sv.serialize_field(as_static_str(key), val)?;
                    }
                    sv.end()
                }
            },

            Self::Option(opt) => match opt {
                Some(val) => serializer.serialize_some(&**val),
                None => serializer.serialize_none(),
            },
        }
    }
}
