use serde::{Deserialize, Deserializer, Serialize};

/// A collection of keys
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(bound(serialize = "I: Clone+Serialize, F: Clone+Serialize"))]
pub struct KeySet<I = u32, F = f32> {
    r#type: u8,
    value: Option<F>,
    /// the last frame
    max: Option<I>,
    #[serde(default)]
    key: Vec<Keyframe<I, F>>,
    #[serde(default)]
    ep_type_pre: ExtendedPlayType,
    #[serde(default)]
    ep_type_post: ExtendedPlayType,
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

#[derive(Debug, Clone)]
pub enum Keyframe<I = u32, F = f32> {
    Default(I),
    Static(F),
    Linear(I, F),
    Hermite(I, F, F),
}

impl<'de, I: Deserialize<'de>, F: Deserialize<'de>> Deserialize<'de> for Keyframe<I, F> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        KeyframeInner::deserialize(deserializer).map(Self::from)
    }
}

impl<I: Clone + Serialize, F: Clone + Serialize> Serialize for Keyframe<I, F> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        KeyframeInner::from(self.clone()).serialize(serializer)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum KeyframesInner {
    Value { value: f32 },
    Keys { key: Vec<KeySet>, max: u32 },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum KeyframeInner<I, F> {
    Value(F),
    Data(KeyframeData<I, F>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum KeyframeData<I, F> {
    None(I),
    Linear(I, F),
    Smooth(I, F, F),
}

impl<I, F> From<KeyframeInner<I, F>> for Keyframe<I, F> {
    fn from(inner: KeyframeInner<I, F>) -> Self {
        match inner {
            KeyframeInner::Value(val) => Self::Static(val),
            KeyframeInner::Data(KeyframeData::None(i)) => Self::Default(i),
            KeyframeInner::Data(KeyframeData::Linear(a, b)) => Self::Linear(a, b),
            KeyframeInner::Data(KeyframeData::Smooth(a, b, c)) => Self::Hermite(a, b, c),
        }
    }
}

impl<I, F> From<Keyframe<I, F>> for KeyframeInner<I, F> {
    fn from(inner: Keyframe<I, F>) -> Self {
        match inner {
            Keyframe::Default(val) => KeyframeInner::Data(KeyframeData::None(val)),
            Keyframe::Static(val) => KeyframeInner::Value(val),
            Keyframe::Linear(a, b) => KeyframeInner::Data(KeyframeData::Linear(a, b)),
            Keyframe::Hermite(a, b, c) => KeyframeInner::Data(KeyframeData::Smooth(a, b, c)),
        }
    }
}
