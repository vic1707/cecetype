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

pub struct SerdeCbor;

impl Format for SerdeCbor {
    type Error = ::serde_cbor::Error;
    type Wire = Vec<u8>;

    fn encode<T: Serialize>(value: &T) -> Result<Self::Wire, Self::Error> {
        ::serde_cbor::to_vec(value)
    }

    fn decode<'de, T: Deserialize<'de>>(wire: &'de Self::Wire) -> Result<T, Self::Error> {
        ::serde_cbor::from_slice(wire)
    }

    fn decode_value<'de, T: Schema>(data: &Self::Wire) -> Result<OwnedValue<'de>, Self::Error> {
        let mut de = ::serde_cbor::Deserializer::from_slice(data);

        T::SCHEMA.decode_value(&mut de)
    }
}

pub struct MessagePack;

impl Format for MessagePack {
    type Error = ProtocolError;
    type Wire = Vec<u8>;

    fn encode<T: Serialize>(value: &T) -> Result<Self::Wire, Self::Error> {
        ::rmp_serde::to_vec(value).map_err(ProtocolError::new)
    }

    fn decode<'de, T: Deserialize<'de>>(wire: &'de Self::Wire) -> Result<T, Self::Error> {
        ::rmp_serde::from_slice(wire).map_err(ProtocolError::new)
    }

    fn decode_value<'de, T: Schema>(data: &Self::Wire) -> Result<OwnedValue<'de>, Self::Error> {
        let mut de = ::rmp_serde::Deserializer::new(&**data);

        T::SCHEMA.decode_value(&mut de).map_err(ProtocolError::new)
    }
}

#[derive(Debug)]
pub struct ProtocolError(String);

impl ProtocolError {
    fn new(err: impl fmt::Display) -> Self {
        Self(err.to_string())
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl error::Error for ProtocolError {}
