use futures::channel::mpsc::Sender;
use mincodec::{MapDeserialize, MapSerialize, MinCodecRead, MinCodecWrite};
use serde::{de, Deserialize, Serialize};
use std::fmt::Debug;
use thiserror::Error;
use wasm_bindgen::JsValue;
use web_sys::Node;

use crate::edit::{
    dynamic::{
        abst::{controls::ControlData, implementation::Root},
        Def, DefData,
    },
    DynamicVariance, UiSection,
};

use super::{decode, Cursor, DynamicCursor, Path, Term};

#[derive(Debug, Error)]
pub enum DynamicReadError {
    #[error("buffer too short")]
    TooShort,
    #[error("unknown dynamic variant {0}")]
    Unknown(u8),
    #[error("invalid dynamic data")]
    Invalid,
}

impl Dynamic<()> {
    pub fn to_buffer(self) -> Vec<u8> {
        let mut buffer = vec![self.term.index()];
        buffer.extend(self.term.encode());
        buffer
    }

    pub fn from_buffer(data: Vec<u8>) -> Result<Dynamic<()>, DynamicReadError> {
        if let Some(first) = data.first() {
            Ok(match *first as char {
                'D' => {
                    let DefData {
                        binder,
                        expression,
                        body,
                    } = if let Ok(data) = decode(&data[1..]) {
                        data
                    } else {
                        return Err(DynamicReadError::Invalid);
                    };
                    Dynamic {
                        annotation: ().into(),
                        term: Box::new(Def::new(body.into(), expression.into(), binder)),
                    }
                }
                'i' => {
                    let data: ControlData = if let Ok(data) = decode(&data[1..]) {
                        data
                    } else {
                        return Err(DynamicReadError::Invalid);
                    };
                    Dynamic {
                        annotation: ().into(),
                        term: Box::new(Root::new(data.to_control())),
                    }
                }
                first => Err(DynamicReadError::Unknown(first as u8))?,
            })
        } else {
            Err(DynamicReadError::TooShort)
        }
    }
}

impl Serialize for Dynamic<()> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.clone().to_buffer())
    }
}

impl<'de> Deserialize<'de> for Dynamic<()> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(
            Dynamic::from_buffer(Deserialize::deserialize(deserializer)?)
                .map_err(|e| <D::Error as de::Error>::custom(e))?,
        )
    }
}

impl MinCodecRead for Dynamic<()> {
    type Deserialize = MapDeserialize<
        DynamicReadError,
        <Vec<u8> as MinCodecRead>::Deserialize,
        Dynamic<()>,
        fn(Vec<u8>) -> Result<Dynamic<()>, DynamicReadError>,
    >;

    fn deserialize() -> Self::Deserialize {
        MapDeserialize::new(|buffer| Dynamic::from_buffer(buffer))
    }
}

#[derive(Debug)]
pub enum DynamicWriteError {}

impl MinCodecWrite for Dynamic<()> {
    type Serialize = MapSerialize<Vec<u8>, DynamicWriteError>;

    fn serialize(self) -> Self::Serialize {
        MapSerialize::new(self, |dynamic| Ok(dynamic.to_buffer()))
    }
}

pub struct Dynamic<T> {
    pub(super) annotation: T,
    pub(super) term: Box<dyn DynamicTerm<T>>,
}

impl<T> Dynamic<T> {
    pub fn new<U: DynamicTerm<T> + 'static>(annotation: T, term: U) -> Self {
        Dynamic {
            annotation,
            term: Box::new(term),
        }
    }

    pub fn term(&self) -> &dyn DynamicTerm<T> {
        self.term.as_ref()
    }

    pub fn into_inner(self) -> (T, Box<dyn DynamicTerm<T>>) {
        (self.annotation, self.term)
    }
}

impl<T: Clone> Clone for Dynamic<T> {
    fn clone(&self) -> Self {
        Self {
            annotation: self.annotation.clone(),
            term: self.term.box_clone(),
        }
    }
}

impl<T: Debug> Debug for Dynamic<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dynamic")
            .field("annotation", &self.annotation)
            .field("term", &TermDebugWrapper(self.term.as_ref()))
            .finish()
    }
}

struct TermDebugWrapper<'a, T>(&'a dyn DynamicTerm<T>);

impl<'a, T: Debug> Debug for TermDebugWrapper<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.debug(f)
    }
}

impl<T: Debug> Debug for Box<dyn DynamicTerm<T>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.debug(f)
    }
}

impl<T: Clone> Clone for Box<dyn DynamicTerm<T>> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

pub trait DynamicTerm<T> {
    fn debug(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    where
        T: Debug;

    fn index(&self) -> u8;
    fn encode(self: Box<Self>) -> Vec<u8>;

    fn expand(self: Box<Self>) -> Term<()>;

    fn box_clone(&self) -> Box<dyn DynamicTerm<T>>
    where
        T: Clone;

    fn add_ui(
        self: Box<Self>,
        sender: &Sender<()>,
        editable: bool,
    ) -> (UiSection, Box<dyn DynamicTerm<UiSection>>);

    fn apply_mutations(
        self: Box<Self>,
        up: Path<UiSection>,
        annotation: Box<dyn DynamicVariance>,
        focused: &mut Option<Cursor<UiSection>>,
        sender: &Sender<()>,
    ) -> Result<Cursor<UiSection>, JsValue>
    where
        Term<T>: Into<Term<UiSection>>;

    fn render_to(
        &self,
        up: &Path<UiSection>,
        annotation: &dyn DynamicVariance,
        node: &Node,
    ) -> Result<(), JsValue>;

    fn clear_annotation(self: Box<Self>) -> Box<dyn DynamicTerm<()>>;
}

impl<T, U: DynamicTerm<T> + ?Sized> DynamicTerm<T> for Box<U> {
    fn debug(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    where
        T: Debug,
    {
        U::debug(self.as_ref(), f)
    }

    fn box_clone(&self) -> Box<dyn DynamicTerm<T>>
    where
        T: Clone,
    {
        U::box_clone(self.as_ref())
    }

    fn add_ui(
        self: Box<Self>,
        trigger_update: &Sender<()>,
        editable: bool,
    ) -> (UiSection, Box<dyn DynamicTerm<UiSection>>) {
        U::add_ui(*self, trigger_update, editable)
    }

    fn apply_mutations(
        self: Box<Self>,
        up: Path<UiSection>,
        annotation: Box<dyn DynamicVariance>,
        focused: &mut Option<Cursor<UiSection>>,
        sender: &Sender<()>,
    ) -> Result<Cursor<UiSection>, JsValue>
    where
        Term<T>: Into<Term<UiSection>>,
    {
        U::apply_mutations(*self, up, annotation, focused, sender)
    }

    fn render_to(
        &self,
        up: &Path<UiSection>,
        annotation: &dyn DynamicVariance,
        node: &Node,
    ) -> Result<(), JsValue> {
        U::render_to(self, up, annotation, node)
    }

    fn clear_annotation(self: Box<Self>) -> Box<dyn DynamicTerm<()>> {
        U::clear_annotation(*self)
    }

    fn index(&self) -> u8 {
        U::index(self)
    }

    fn encode(self: Box<Self>) -> Vec<u8> {
        U::encode(*self)
    }

    fn expand(self: Box<Self>) -> Term<()> {
        U::expand(*self)
    }
}

impl<T> DynamicCursor<T> {
    pub fn ascend(self) -> Cursor<T>
    where
        T: 'static,
    {
        Cursor::ascend_helper(
            self.up,
            Term::Dynamic(Dynamic {
                annotation: self.annotation,
                term: self.term,
            }),
        )
        .unwrap_or_else(|(path, term)| Cursor::from_term_and_path(term, path))
    }

    pub fn annotation(&self) -> &T {
        &self.annotation
    }

    pub fn annotation_mut(&mut self) -> &mut T {
        &mut self.annotation
    }

    pub fn term(&self) -> &dyn DynamicTerm<T> {
        &self.term
    }

    pub fn term_mut(&mut self) -> &mut dyn DynamicTerm<T> {
        &mut self.term
    }

    pub fn expand(self) -> Term<()> {
        self.term.expand()
    }
}
