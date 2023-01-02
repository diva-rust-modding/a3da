use serde::{Deserialize, Serialize};

pub mod camera;
pub mod keyframe;
pub mod metadata;
pub mod object;

use crate::camera::CameraRoot;
use crate::metadata::Metadata;
use crate::object::Object;

#[derive(Debug, Serialize, Deserialize)]
pub struct A3da {
    #[serde(default, rename = "camera_root")]
    pub camera_roots: Vec<CameraRoot>,
    #[serde(default, rename = "object")]
    pub objects: Vec<Object>,
    pub play_control: PlayControl,
    #[serde(rename = "_")]
    pub metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayControl {
    pub begin: usize,
    pub fps: usize,
    pub size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = include_str!("../assets/CAMPV001_BASE.a3da");

    #[test]
    fn it_works() {
        let a3da = serde_divatree::de::from_str::<A3da>(INPUT);
        dbg!(a3da);
        panic!();
        assert_eq!(2 + 2, 4);
    }
}
