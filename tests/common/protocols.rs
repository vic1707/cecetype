use ::schema::{OwnedValue, Schema};
use ::{
    core::{error, fmt},
    serde::{Deserialize, Serialize},
};

pub trait Format {
    type Error: fmt::Display + error::Error;
    type Wire: fmt::Debug + PartialEq;

    fn encode<T: Serialize>(value: &T) -> Result<Self::Wire, Self::Error>;
    fn decode<'de, T: Deserialize<'de>>(wire: &'de Self::Wire) -> Result<T, Self::Error>;
    fn decode_value<'de, T: Schema>(data: &Self::Wire) -> Result<OwnedValue<'de>, Self::Error>;
}

pub struct Json;

impl Format for Json {
    type Error = ::serde_json::Error;
    type Wire = String;

    fn encode<T: Serialize>(value: &T) -> Result<Self::Wire, Self::Error> {
        ::serde_json::to_string(value)
    }

    fn decode<'de, T: Deserialize<'de>>(wire: &'de Self::Wire) -> Result<T, Self::Error> {
        ::serde_json::from_str(wire)
    }

    fn decode_value<'de, T: Schema>(data: &Self::Wire) -> Result<OwnedValue<'de>, Self::Error> {
        let mut de = ::serde_json::Deserializer::from_str(data);

        T::SCHEMA.decode_value(&mut de)
    }
}

pub struct Postcard;

impl Format for Postcard {
    type Error = ::postcard::Error;
    type Wire = Vec<u8>;

    fn encode<T: Serialize>(value: &T) -> Result<Self::Wire, Self::Error> {
        ::postcard::to_stdvec(value)
    }

    fn decode<'de, T: Deserialize<'de>>(wire: &'de Self::Wire) -> Result<T, Self::Error> {
        ::postcard::from_bytes(wire)
    }

    fn decode_value<'de, T: Schema>(data: &Self::Wire) -> Result<OwnedValue<'de>, Self::Error> {
        let mut de = ::postcard::Deserializer::from_bytes(data);

        T::SCHEMA.decode_value(&mut de)
    }
}

pub struct Yaml;

impl Format for Yaml {
    type Error = ::yaml_serde::Error;
    type Wire = String;

    fn encode<T: Serialize>(value: &T) -> Result<Self::Wire, Self::Error> {
        ::yaml_serde::to_string(value)
    }

    fn decode<'de, T: Deserialize<'de>>(wire: &'de Self::Wire) -> Result<T, Self::Error> {
        ::yaml_serde::from_str(wire)
    }

    fn decode_value<'de, T: Schema>(data: &Self::Wire) -> Result<OwnedValue<'de>, Self::Error> {
        let de = ::yaml_serde::Deserializer::from_str(data);

        T::SCHEMA.decode_value(de)
    }
}
