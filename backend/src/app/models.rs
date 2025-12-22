use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Note {
    #[serde(serialize_with = "serialize_u64_as_string", deserialize_with = "deserialize_u64_from_string_or_number")]
    pub id: u64,
    pub created_ms: u128,
    pub updated_ms: u128,
    pub pinned: bool,
    pub tags: Vec<String>,
    pub content: String,
    pub changes: Vec<Change>,
}

fn serialize_u64_as_string<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

fn deserialize_u64_from_string_or_number<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Unexpected, Visitor};
    use std::fmt;
    struct U64Visitor;
    impl<'de> Visitor<'de> for U64Visitor {
        type Value = u64;
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a u64 as a string or number")
        }
        fn visit_u64<E>(self, value: u64) -> Result<u64, E> {
            Ok(value)
        }
        fn visit_str<E>(self, value: &str) -> Result<u64, E>
        where
            E: de::Error,
        {
            value.parse::<u64>().map_err(|_| de::Error::invalid_value(Unexpected::Str(value), &self))
        }
    }
    deserializer.deserialize_any(U64Visitor)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Change {
    pub change_date_ms: u128,
    pub pin_change: PinChange,
    pub tag_change: TagChange,
    pub content_change: ContentChange,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PinChange {
    pub before: bool,
    pub after: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TagChange {
    pub before: Vec<String>,
    pub after: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContentChange {
    pub before: String,
    pub after: String,
}