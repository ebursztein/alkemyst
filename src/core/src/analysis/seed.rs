use serde::{Deserialize, Deserializer, Serializer, de::Error};
pub(crate) fn parse(value: &str) -> Result<u64, &'static str> {
    if value.len() != 16 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("seed must contain exactly 16 hexadecimal digits");
    }
    u64::from_str_radix(value, 16).map_err(|_| "invalid seed")
}
pub(crate) fn encode(seed: u64) -> String {
    format!("{seed:016x}")
}
pub fn serialize<S: Serializer>(seed: &u64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&encode(*seed))
}
pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    parse(&String::deserialize(deserializer)?).map_err(D::Error::custom)
}
