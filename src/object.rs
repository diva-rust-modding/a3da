use serde::{Deserialize, Serialize};

use crate::camera::ModelTransform;

#[derive(Debug, Deserialize, Serialize)]
pub struct Object {
    pub name: String,
    pub uid_name: String,
    #[serde(flatten)]
    pub transform: ModelTransform,
}
