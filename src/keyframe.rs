use std::{
    collections::{BTreeMap, HashMap},
    marker::PhantomData,
};

use serde::{Deserialize, Deserializer, Serialize};

/// A collection of keys
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(bound(serialize = "I: Clone+Serialize, F: Clone+Serialize"))]
pub struct KeySet<I: Default + Ord = u32, F: Default = f32> {
    #[serde(flatten)]
    data: KeySetData<I, F>,
    #[serde(default)]
    ep_type_pre: ExtendedPlayType,
    #[serde(default)]
    ep_type_post: ExtendedPlayType,
}

#[derive(Debug, Default, Clone)]
pub enum KeySetData<I: Ord, F> {
    #[default]
    None,
    Static(F),
    KeySet(BTreeMap<I, Frame<F>>),
}

impl<'de, I, F> Deserialize<'de> for KeySetData<I, F>
where
    I: Ord + Deserialize<'de> + Default,
    F: Deserialize<'de> + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        KeySetDataSerde::deserialize(deserializer).map(Into::into)
    }
}

impl<I, F> Serialize for KeySetData<I, F>
where
    I: Ord + Serialize + Clone + Default,
    F: Serialize + Clone + Default,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        KeySetDataSerde::from(self.clone()).serialize(serializer)
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(bound(serialize = "I: Clone+Serialize, F: Clone+Serialize"))]
struct KeySetDataSerde<I: Ord, F> {
    #[serde(rename = "type")]
    ty: u8,
    value: Option<F>,
    #[serde(default)]
    key: Vec<KeyframeInner<I, F>>,
    max: Option<I>,
}

impl<I: Ord, F> From<KeySetDataSerde<I, F>> for KeySetData<I, F> {
    fn from(value: KeySetDataSerde<I, F>) -> Self {
        let KeySetDataSerde {
            ty,
            value,
            key,
            max,
        } = value;
        let frames = key.into_iter().flat_map(|x| x.data.decompose()).collect();
        match ty {
            0 => Self::None,
            1 => Self::Static(value.unwrap()),
            3 => Self::KeySet(frames),
            e => unreachable!("Unexpected variant {}", e),
        }
    }
}

impl<I: Ord + Default + Clone, F: Default> From<KeySetData<I, F>> for KeySetDataSerde<I, F> {
    fn from(value: KeySetData<I, F>) -> Self {
        match value {
            KeySetData::None => Self {
                ty: 0,
                ..Default::default()
            },
            KeySetData::Static(value) => Self {
                ty: 1,
                value: Some(value),
                key: vec![],
                max: None,
            },
            KeySetData::KeySet(key) => {
                // HACK: use `.last_entry()` when that is stabilized
                let max = key.iter().last().map(|(i, _)| i).cloned();
                let key = key
                    .into_iter()
                    .map(KeyframeData::compose)
                    .map(From::from)
                    .collect();
                Self {
                    ty: 3,
                    value: None,
                    key,
                    max,
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ExtendedPlayType {
    None,
    Linear,
    Cycle,
    CycleOffset,
}

impl Default for ExtendedPlayType {
    fn default() -> Self {
        Self::None
    }
}

// HACK: this is a workaround for
// https://github.com/serde-rs/serde/issues/745
//
// Ideally this should be done as an internally tagged
// enum with the variants named to their index
#[derive(Debug, Serialize, Deserialize)]
struct KeyframeInner<I, F> {
    #[serde(rename = "type")]
    ty: u8,
    data: KeyframeData<I, F>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum KeyframeData<I, F> {
    None(I),
    Linear(I, F),
    Smooth(I, F, F),
    Type3(I, F, F, F),
}

#[derive(Debug, Clone, Copy)]
pub enum Frame<F> {
    Linear(F),
    Smooth(F, F),
    Type3(F, F, F),
}

impl<I, F> KeyframeData<I, F> {
    fn compose((i, frame): (I, Frame<F>)) -> Self {
        match frame {
            Frame::Linear(x) => Self::Linear(i, x),
            Frame::Smooth(x, y) => Self::Smooth(i, x, y),
            Frame::Type3(x, y, z) => Self::Type3(i, x, y, z),
        }
    }

    fn decompose(self) -> Option<(I, Frame<F>)> {
        match self {
            KeyframeData::None(_) => None,
            KeyframeData::Linear(i, x) => Some((i, Frame::Linear(x))),
            KeyframeData::Smooth(i, x, y) => Some((i, Frame::Smooth(x, y))),
            KeyframeData::Type3(i, x, y, z) => Some((i, Frame::Type3(x, y, z))),
        }
    }
}

impl<I, F> From<KeyframeData<I, F>> for KeyframeInner<I, F> {
    fn from(value: KeyframeData<I, F>) -> Self {
        let ty = match value {
            KeyframeData::None(_) => 0,
            KeyframeData::Linear(_, _) => 1,
            KeyframeData::Smooth(_, _, _) => 2,
            KeyframeData::Type3(_, _, _, _) => 3,
        };
        Self { ty, data: value }
    }
}
