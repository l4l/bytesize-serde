#![doc = include_str!("../README.md")]

use std::fmt;

use bytesize::ByteSize;
use serde::{de, Serialize, Serializer};

pub fn serialize<S>(size: &ByteSize, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if serializer.is_human_readable() {
        <str>::serialize(size.to_string().as_str(), serializer)
    } else {
        size.0.serialize(serializer)
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<ByteSize, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct Helper;
    impl<'de> de::Visitor<'de> for Helper {
        type Value = ByteSize;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("an integer or string")
        }

        // TOML Integer(i64)
        fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
            if let Ok(val) = u64::try_from(value) {
                Ok(ByteSize(val))
            } else {
                Err(E::invalid_value(
                    de::Unexpected::Signed(value),
                    &"integer overflow",
                ))
            }
        }

        // JSON Integer(u64)
        fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
            Ok(ByteSize(value))
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
            if let Ok(val) = value.parse() {
                Ok(val)
            } else {
                Err(E::invalid_value(
                    de::Unexpected::Str(value),
                    &"parsable string",
                ))
            }
        }
    }

    if deserializer.is_human_readable() {
        deserializer.deserialize_any(Helper)
    } else {
        deserializer.deserialize_u64(Helper)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use quickcheck_macros::quickcheck;
    use serde::Deserialize;

    #[derive(Serialize, Deserialize)]
    #[serde(transparent)]
    struct W(#[serde(with = "self")] ByteSize);

    #[quickcheck]
    fn deserializes_any(x: u64) {
        let _: W = serde_json::from_str(&x.to_string()).unwrap();
    }

    #[quickcheck]
    fn serde_with_bincode(x: u64) {
        let w = W(ByteSize::b(x));

        let ser = bincode::serialize(&w).unwrap();
        let de: W = bincode::deserialize(&ser).unwrap();

        assert_eq!(w.0, de.0);
    }

    #[quickcheck]
    fn serializes_any(x: u64) {
        serde_json::to_string(&ByteSize(x).to_string()).unwrap();
    }

    #[test]
    fn deserialize_sizes() {
        #[track_caller]
        fn check_str(s: &str) {
            assert_eq!(
                serde_json::from_str::<W>(&format!("{:?}", s)).unwrap().0,
                s.parse().unwrap()
            );
        }

        #[track_caller]
        fn check(s: &str) {
            assert_eq!(serde_json::from_str::<W>(s).unwrap().0, s.parse().unwrap());
        }

        check_str("5 MB");
        check_str("12.34 KB");
        check("123");
        check("0");
    }
}
