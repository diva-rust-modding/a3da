use serde::{Deserialize, Serialize};

pub mod camera;
#[cfg(feature = "ffi")]
pub mod ffi;
pub mod keyframe;
pub mod metadata;

use crate::camera::CameraRoot;
use crate::metadata::Metadata;

#[derive(Debug, Serialize, Deserialize)]
pub struct A3da {
    pub camera_root: Vec<CameraRoot>,
    pub play_control: PlayControl,
    #[serde(rename = "_")]
    pub metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayControl {
    pub begin: u32,
    pub fps: u32,
    pub size: u32,
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
