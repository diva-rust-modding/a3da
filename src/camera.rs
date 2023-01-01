use serde::{Deserialize, Serialize};

use crate::keyframe::KeySet;

/// Represents a camera.
#[derive(Debug, Serialize, Deserialize)]
pub struct CameraRoot {
    /// the target the camera is pointing at
    pub interest: ModelTransform,
    #[serde(flatten)]
    pub transform: ModelTransform,
    /// camera properties
    pub view_point: ViewPoint,
}

pub struct CameraAuxilary {
    gamma: KeySet,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ModelTransform {
    #[serde(rename = "trans")]
    pub translation: Vec3<KeySet>,
    #[serde(rename = "rot")]
    pub rotation: Vec3<KeySet>,
    pub scale: Vec3<KeySet>,
    pub visibility: KeySet,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ViewPoint {
    #[serde(rename = "aspect")]
    pub aspect_ratio: f32,
    #[serde(flatten)]
    pub transform: ModelTransform,
    #[serde(default)]
    pub focal_length: KeySet,
    #[serde(default)]
    pub roll: KeySet,
    #[serde(flatten)]
    pub fov: FieldOfView,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldOfView {
    #[serde(rename = "fov_is_horizontal")]
    horizontal: u8,
    /// The angle of view in radians
    #[serde(rename = "fov", default)]
    value: KeySet,
}
