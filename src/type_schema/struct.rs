use super::TypeSchema;

pub type MapStructSchema = StructSchema<MapFieldSchema>;
pub type ArrayStructSchema = StructSchema<ArrayFieldSchema>;

#[derive(Debug)]
pub struct StructSchema<T: 'static> {
    pub name: &'static str,
    pub fields: &'static [T],
}

#[derive(Debug)]
pub struct MapFieldSchema {
    pub name: &'static str,
    pub key: u32,
    pub ty: &'static TypeSchema,
}

#[derive(Debug)]
pub struct ArrayFieldSchema {
    pub name: &'static str,
    pub position: usize,
    pub ty: &'static TypeSchema,
}
