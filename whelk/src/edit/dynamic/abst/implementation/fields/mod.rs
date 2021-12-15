use wasm_bindgen::{prelude::Closure, JsValue};
use web_sys::Element;

mod container;
mod r#static;
mod string;
mod term;
pub use container::*;
pub use r#static::*;
pub use string::*;
pub use term::*;

pub trait FieldContextData {
    type Data;
}

pub struct RootFieldContext<T: FieldContextData> {
    data: T::Data,
    closures: Vec<Closure<dyn FnMut(JsValue)>>,
}

impl<T: FieldContextData> RootFieldContext<T> {
    pub fn data(&self) -> &T::Data {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut T::Data {
        &mut self.data
    }
}

pub enum RootFieldData {
    String {
        context: RootFieldContext<RootStringField>,
    },
    Static {
        context: RootFieldContext<RootStaticField>,
    },
    Container {
        context: RootFieldContext<RootContainerField>,
    },
    Term {
        context: RootFieldContext<RootTermField>,
    },
}

impl RootFieldData {
    pub fn container_element(&self) -> &Element {
        match self {
            RootFieldData::String { context } => context.data.element(),
            RootFieldData::Static { context } => context.data.element(),
            RootFieldData::Container { context } => context.data.element(),
            RootFieldData::Term { context } => context.data.element(),
        }
    }
}
