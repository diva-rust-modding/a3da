use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub converter: Converter,
    pub property: Property,
    pub file_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Converter {
    pub version: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Property {
    pub version: u32,
}
