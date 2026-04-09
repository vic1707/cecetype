use crate::{flavors::ValueFlavor, utils::as_static_str};
use ::{core::fmt, derive_where::derive_where};

#[derive_where(Debug;)] // prevents compiler bounds check overflow & `VF: Debug` bound
#[derive_where(PartialEq;)] // prevents compiler bounds check overflow & `VF: PartialEq` bound
#[non_exhaustive]
pub enum Data<VF: ValueFlavor> {
    Unit {
        name: VF::Str,
    },
    NewType {
        name: VF::Str,
        field: VF::Ptr<Value<VF>>,
    },
    Tuple {
        name: VF::Str,
        fields: VF::List<Value<VF>>,
    },
    Struct {
        name: VF::Str,
        fields: VF::List<(VF::Str, Value<VF>)>,
    },
}

impl<VF: ValueFlavor> Data<VF> {
    #[inline]
    pub fn name(&self) -> &str {
        match self {
            Self::Unit { name }
            | Self::NewType { name, .. }
            | Self::Tuple { name, .. }
            | Self::Struct { name, .. } => name,
        }
    }
}

#[derive_where(Debug;)] // prevents compiler bounds check overflow & `VF: Debug` bound
#[derive_where(PartialEq;)] // prevents compiler bounds check overflow & `VF: PartialEq` bound
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
        // TODO: tuple variant when `yaml_serde` supports nested enums
        data: Data<VF>,
    },

    Enum {
        name: VF::Str,
        discriminant: u32,
        data: Data<VF>,
    },

    Option(Option<VF::Ptr<Self>>),
}

impl<VF> fmt::Display for Data<VF>
where
    VF: ValueFlavor,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unit { name } => write!(f, "{}", &**name),
            Self::NewType { name, field } => {
                write!(f, "{}({})", &**name, &**field)
            }
            Self::Tuple { name, fields } => {
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
        }
    }
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

            Self::Struct { data } => write!(f, "{data}"),

            Self::Enum { name, data, .. } => {
                write!(f, "{}::{data}", &**name)
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

            Self::Struct { data } => serialize_data(data, None, serializer),

            Self::Enum {
                name,
                discriminant,
                data,
            } => serialize_data(data, Some((name, *discriminant)), serializer),

            Self::Option(opt) => match opt {
                Some(val) => serializer.serialize_some(&**val),
                None => serializer.serialize_none(),
            },
        }
    }
}

fn serialize_data<S, VF>(
    data: &Data<VF>,
    enum_ctx: Option<(&VF::Str, u32)>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: ::serde::Serializer,
    VF: ValueFlavor,
    VF::Str: ::serde::Serialize,
{
    match (data, enum_ctx) {
        (Data::Unit { name }, None) => serializer.serialize_unit_struct(as_static_str(name)),
        (Data::Unit { name }, Some((enum_name, discriminant))) => serializer
            .serialize_unit_variant(as_static_str(enum_name), discriminant, as_static_str(name)),

        (Data::NewType { name, field }, None) => {
            serializer.serialize_newtype_struct(as_static_str(name), &**field)
        }
        (Data::NewType { name, field }, Some((enum_name, discriminant))) => serializer
            .serialize_newtype_variant(
                as_static_str(enum_name),
                discriminant,
                as_static_str(name),
                &**field,
            ),

        (Data::Tuple { name, fields }, None) => {
            use ::serde::ser::SerializeTupleStruct as _;

            let mut ts = serializer.serialize_tuple_struct(as_static_str(name), fields.len())?;
            for field in &**fields {
                ts.serialize_field(field)?;
            }
            ts.end()
        }
        (Data::Tuple { name, fields }, Some((enum_name, discriminant))) => {
            use ::serde::ser::SerializeTupleVariant as _;

            let mut tv = serializer.serialize_tuple_variant(
                as_static_str(enum_name),
                discriminant,
                as_static_str(name),
                fields.len(),
            )?;
            for field in &**fields {
                tv.serialize_field(field)?;
            }
            tv.end()
        }

        (Data::Struct { name, fields }, None) => {
            use ::serde::ser::SerializeStruct as _;

            let mut st = serializer.serialize_struct(as_static_str(name), fields.len())?;
            for (key, val) in &**fields {
                st.serialize_field(as_static_str(key), val)?;
            }
            st.end()
        }
        (Data::Struct { name, fields }, Some((enum_name, discriminant))) => {
            use ::serde::ser::SerializeStructVariant as _;

            let mut sv = serializer.serialize_struct_variant(
                as_static_str(enum_name),
                discriminant,
                as_static_str(name),
                fields.len(),
            )?;
            for (key, val) in &**fields {
                sv.serialize_field(as_static_str(key), val)?;
            }
            sv.end()
        }
    }
}
