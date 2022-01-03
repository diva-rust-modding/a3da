use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    converter: Converter,
    property: Property,
    file_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Converter {
    version: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Property {
    version: usize,
}
