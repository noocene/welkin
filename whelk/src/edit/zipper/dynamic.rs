use futures::channel::mpsc::Sender;
use mincodec::{MapDeserialize, MapSerialize, MinCodecRead, MinCodecWrite};
use std::fmt::Debug;
use wasm_bindgen::JsValue;
use web_sys::Node;

use crate::edit::{
    dynamic::{Def, DefData},
    DynamicVariance, UiSection,
};

use super::{decode, Cursor, DynamicCursor, Path, Term};

#[derive(Debug)]
pub enum DynamicReadError {
    TooShort,
    Unknown(u8),
    Invalid,
}

impl MinCodecRead for Dynamic<()> {
    type Deserialize = MapDeserialize<
        DynamicReadError,
        <Vec<u8> as MinCodecRead>::Deserialize,
        Dynamic<()>,
        fn(Vec<u8>) -> Result<Dynamic<()>, DynamicReadError>,
    >;

    fn deserialize() -> Self::Deserialize {
        MapDeserialize::new(|buffer| {
            if let Some(first) = buffer.first() {
                Ok(match *first as char {
                    'D' => {
                        let DefData {
                            binder,
                            expression,
                            body,
                        } = if let Ok(data) = decode(&buffer[1..]) {
                            data
                        } else {
                            return Err(DynamicReadError::Invalid);
                        };
                        Dynamic {
                            annotation: ().into(),
                            term: Box::new(Def::new(body.into(), expression.into(), binder)),
                        }
                    }
                    first => Err(DynamicReadError::Unknown(first as u8))?,
                })
            } else {
                Err(DynamicReadError::TooShort)
            }
        })
    }
}

#[derive(Debug)]
pub enum DynamicWriteError {}

impl MinCodecWrite for Dynamic<()> {
    type Serialize = MapSerialize<Vec<u8>, DynamicWriteError>;

    fn serialize(self) -> Self::Serialize {
        MapSerialize::new(self, |dynamic| {
            Ok({
                let mut buffer = vec![dynamic.term.index()];
                buffer.extend(dynamic.term.encode());
                buffer
            })
        })
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

    fn box_clone(&self) -> Box<dyn DynamicTerm<T>>
    where
        T: Clone;

    fn add_ui(self: Box<Self>, sender: &Sender<()>)
        -> (UiSection, Box<dyn DynamicTerm<UiSection>>);

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
    ) -> (UiSection, Box<dyn DynamicTerm<UiSection>>) {
        U::add_ui(*self, trigger_update)
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
}

impl<T> DynamicCursor<T> {
    pub fn ascend(self) -> Cursor<T> {
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

    pub fn expand(self) -> Term<T> {
        todo!()
    }
}
