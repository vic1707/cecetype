use super::TypeSchema;

#[derive(Debug)]
pub struct EnumSchema<T: 'static> {
    pub name: &'static str,
    pub variants: &'static [T],
}

#[derive(Debug)]
pub struct MapVariantSchema {
    pub name: &'static str,
    pub key: u32,
    pub payload: Option<&'static TypeSchema>,
}

#[derive(Debug)]
pub struct ArrayVariantSchema {
    pub name: &'static str,
    pub position: usize,
    pub payload: Option<&'static TypeSchema>,
}

pub type MapEnumSchema = EnumSchema<MapVariantSchema>;
pub type ArrayEnumSchema = EnumSchema<ArrayVariantSchema>;
