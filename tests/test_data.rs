use base64::{engine::general_purpose, Engine as _};
use once_cell::sync::Lazy;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct JVarbitInt {
    pub v: i64,
    #[serde(deserialize_with = "deserialize_bytes")]
    pub e: Vec<u8>,
}

#[derive(Deserialize, Debug)]
pub struct JVarbitUint {
    pub v: u64,
    #[serde(deserialize_with = "deserialize_bytes")]
    pub e: Vec<u8>,
}

#[derive(Deserialize, Debug)]
pub struct JUvarint {
    pub v: u64,
    #[serde(deserialize_with = "deserialize_bytes")]
    pub e: Vec<u8>,
}

#[derive(Deserialize, Debug)]
pub struct JSample {
    pub ts: i64,
    pub v: f64,
}

#[derive(Deserialize, Debug)]
pub struct JChunk {
    pub s: Vec<JSample>,
    #[serde(deserialize_with = "deserialize_bytes")]
    pub e: Vec<u8>,
}

#[derive(Deserialize, Debug)]
pub struct TestJson {
    pub varbit_ints: Vec<JVarbitInt>,
    pub varbit_uints: Vec<JVarbitUint>,
    pub uvarints: Vec<JUvarint>,
    pub chunks: Vec<JChunk>,
}

fn deserialize_bytes<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    general_purpose::STANDARD_NO_PAD
        .decode(s)
        .map_err(serde::de::Error::custom)
}

pub static TEST_DATA: Lazy<TestJson> = Lazy::new(|| {
    let file_content = include_str!("test_data.json");
    serde_json::from_str(file_content).expect("Failed to parse test_data.json")
});
