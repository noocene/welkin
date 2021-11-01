use wasm_bindgen::{prelude::Closure, JsValue};
use web_sys::Element;

mod container;
mod r#static;
mod string;
pub use container::*;
pub use r#static::*;
pub use string::*;

pub trait FieldContextData {
    type Data;
}

pub struct RootFieldContext<T: FieldContextData> {
    data: T::Data,
    closures: Vec<Closure<dyn FnMut(JsValue)>>,
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
}

impl RootFieldData {
    pub fn container_element(&self) -> &Element {
        match self {
            RootFieldData::String { context } => context.data.element(),
            RootFieldData::Static { context } => context.data.element(),
            RootFieldData::Container { context } => context.data.element(),
        }
    }
}
