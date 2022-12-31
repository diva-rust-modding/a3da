use std::{
    collections::{BTreeMap, HashMap},
    marker::PhantomData,
};

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Default, Clone)]
pub enum KeySet<I: Ord = u32, F = f32> {
    /// Set the value to be 0,
    ///
    /// Corresponds to type 0 in A3DA
    #[default]
    None,
    /// Sets the value to the provided initial value.
    ///
    /// Corresponds to type 1 in A3DA
    Static(F),
    /// Corresponds to type 2
    Linear(KeySetInner<I, Frame<F, Linear>>),
    /// Corresponds to type 3
    Hermite(KeySetInner<I, Frame<F, Hermite<F>>>),
    /// Corresponds to type 4
    Step(KeySetInner<I, Frame<F, Step>>),
}

#[derive(Debug, Default, Clone)]
pub struct KeySetInner<I, K> {
    pub keys: BTreeMap<I, K>,
    pub ep_type: ExtendedPlayTypes,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct ExtendedPlayTypes {
    #[serde(rename = "ep_type_pre", default)]
    pub pre: ExtendedPlayType,
    #[serde(rename = "ep_type_post", default)]
    pub post: ExtendedPlayType,
}

impl<'de, I, F> Deserialize<'de> for KeySet<I, F>
where
    I: Ord + Deserialize<'de> + Default,
    F: Deserialize<'de> + Default + PartialEq + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        KeySetDataSerde::deserialize(deserializer).map(Into::into)
    }
}

impl<I, F> Serialize for KeySet<I, F>
where
    I: Ord + Serialize + Clone + Default,
    F: Serialize + Clone + Default + PartialEq + Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        KeySetDataSerde::from(self.clone()).serialize(serializer)
    }
}

// HACK: this is a workaround for
// https://github.com/serde-rs/serde/issues/745
//
// Ideally this should be done as an internally tagged
// enum with the variants named to their index
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(bound(serialize = "I: Clone+Serialize, F: Clone+Serialize"))]
struct KeySetDataSerde<I: Ord, F> {
    #[serde(rename = "type")]
    ty: u8,
    value: Option<F>,
    #[serde(default)]
    key: Vec<KeyframeInner<I, F>>,
    max: Option<I>,
    #[serde(flatten)]
    ep_type: ExtendedPlayTypes,
}

impl<I: Ord, F: Clone + Default + PartialEq, P: FrameInterpolation<F>> KeySetInner<I, Frame<F, P>> {
    fn convert(key: Vec<KeyframeInner<I, F>>, ep_type: ExtendedPlayTypes) -> Self {
        let keys = key.into_iter().map(|x| x.data.decompose()).collect();
        Self { keys, ep_type }
    }
}

impl<I: Ord, F: Clone + Default + PartialEq> From<KeySetDataSerde<I, F>> for KeySet<I, F> {
    fn from(value: KeySetDataSerde<I, F>) -> Self {
        let KeySetDataSerde {
            ty,
            value,
            key,
            max,
            ep_type,
        } = value;
        match ty {
            0 => Self::None,
            1 => Self::Static(value.unwrap()),
            2 => Self::Linear(KeySetInner::convert(key, ep_type)),
            3 => Self::Hermite(KeySetInner::convert(key, ep_type)),
            4 => Self::Step(KeySetInner::convert(key, ep_type)),
            e => unreachable!("Unexpected variant {}", e),
        }
    }
}

impl<I: Ord + Default + Clone, F: Default + Clone + PartialEq> From<KeySet<I, F>>
    for KeySetDataSerde<I, F>
{
    fn from(value: KeySet<I, F>) -> Self {
        fn convert<
            I: Ord + Default + Clone,
            F: Default + Clone + PartialEq,
            P: FrameInterpolation<F>,
        >(
            tag: usize,
            key: KeySetInner<I, Frame<F, P>>,
        ) -> KeySetDataSerde<I, F> {
            // HACK: use `.last_entry()` when that is stabilized
            let ep_type = key.ep_type;
            let max = key.keys.iter().last().map(|(i, _)| i).cloned();
            let key = key
                .keys
                .into_iter()
                .map(KeyframeData::compose)
                .map(From::from)
                .collect();
            KeySetDataSerde {
                ty: 2,
                value: None,
                key,
                max,
                ep_type,
            }
        }

        match value {
            KeySet::None => Self {
                ty: 0,
                ..Default::default()
            },
            KeySet::Static(value) => Self {
                ty: 1,
                value: Some(value),
                ..Default::default()
            },
            KeySet::Linear(key) => convert(2, key),
            KeySet::Hermite(key) => convert(3, key),
            KeySet::Step(key) => convert(4, key),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

/// Compact holder for keyframe data
///
/// A3DA keyframes can be represented in 4 different ways inside the textual format,
/// depending on which way is more efficient.
///
/// | Variant | Value | Tangents  |
/// |---------+-------+-----------|
/// | Type0  | 0     | 0         |
/// | Type1  | set   | 0         |
/// | Type2  | set   | equal     |
/// | Type3  | set   | different |
///
/// Note that the case changes depending on the keyframe in question.
/// Therefore it is not uncommon to see the case change within the same keyset.
/// As the game perfers the most compact representation according to the above table.
///
/// Consequently, it makes sense for `Frame<F, Linear>` and `Frame<F, Step>` to be stored as `Tuple2` or `Tuple1`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum KeyframeData<I, F> {
    Type0(I),
    Type1(I, F),
    Type2(I, F, F),
    Type3(I, F, F, F),
}

trait FrameInterpolation<F>: Default + From<F> + From<(F, F)> + Into<(F, F)> {}

#[derive(Default, Debug, Clone, Copy)]
pub struct Linear;

// HACK: due specialization not being stablized,
// we have to put extra trait bounds to avoid the `From<T> for T` case
impl<F: Default + PartialEq + Clone> From<F> for Linear {
    fn from(value: F) -> Self {
        Self
    }
}

impl<F: Default> Into<(F, F)> for Linear {
    fn into(self) -> (F, F) {
        (F::default(), F::default())
    }
}

impl<F: Default + PartialEq + Clone> FrameInterpolation<F> for Linear {}

#[derive(Default, Debug, Clone, Copy)]
pub struct Step;

// HACK: due specialization not being stablized,
// we have to put extra trait bounds to avoid the `From<T> for T` case
impl<F: Default + PartialEq + Clone> From<F> for Step {
    fn from(value: F) -> Self {
        Self
    }
}

impl<F: Default> Into<(F, F)> for Step {
    fn into(self) -> (F, F) {
        (F::default(), F::default())
    }
}

impl<F: Default + PartialEq + Clone> FrameInterpolation<F> for Step {}

#[derive(Default, Debug, Clone, Copy)]
pub struct Hermite<F> {
    pre: F,
    post: F,
}

impl<F: Clone> From<F> for Hermite<F> {
    fn from(f: F) -> Self {
        Self {
            pre: f.clone(),
            post: f,
        }
    }
}

impl<F> From<(F, F)> for Hermite<F> {
    fn from((pre, post): (F, F)) -> Self {
        Self { pre, post }
    }
}

impl<F> Into<(F, F)> for Hermite<F> {
    fn into(self) -> (F, F) {
        (self.pre, self.post)
    }
}

impl<F: Clone + Default> FrameInterpolation<F> for Hermite<F> {}

#[derive(Debug, Default, Clone, Copy)]
pub struct Frame<F, P> {
    pub value: F,
    pub interpolation: P,
}

impl<I, F: Default + PartialEq> KeyframeData<I, F> {
    fn compose<P: FrameInterpolation<F>>((i, f): (I, Frame<F, P>)) -> Self {
        if f.value == F::default() {
            Self::Type0(i)
        } else {
            let (t1, t2) = f.interpolation.into();
            if t1 == t2 {
                if t1 == F::default() {
                    Self::Type1(i, f.value)
                } else {
                    Self::Type2(i, f.value, t1)
                }
            } else {
                Self::Type3(i, f.value, t1, t2)
            }
        }
    }

    fn decompose<P: FrameInterpolation<F>>(self) -> (I, Frame<F, P>) {
        match self {
            KeyframeData::Type0(i) => (i, Frame::default()),
            KeyframeData::Type1(i, value) => (
                i,
                Frame {
                    value,
                    interpolation: P::default(),
                },
            ),
            KeyframeData::Type2(i, value, x) => (
                i,
                Frame {
                    value,
                    interpolation: x.into(),
                },
            ),
            KeyframeData::Type3(i, value, x, y) => (
                i,
                Frame {
                    value,
                    interpolation: (x, y).into(),
                },
            ),
        }
    }
}

impl<I, F> From<KeyframeData<I, F>> for KeyframeInner<I, F> {
    fn from(value: KeyframeData<I, F>) -> Self {
        let ty = match value {
            KeyframeData::Type0(_) => 0,
            KeyframeData::Type1(_, _) => 1,
            KeyframeData::Type2(_, _, _) => 2,
            KeyframeData::Type3(_, _, _, _) => 3,
        };
        Self { ty, data: value }
    }
}
