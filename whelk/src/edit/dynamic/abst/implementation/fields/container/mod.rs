use web_sys::Element;

use crate::edit::dynamic::abst::{
    implementation::{RootContext, RootHandle},
    Container, Field, FieldContext,
};

mod v_stack;
pub use v_stack::*;
mod wrapper;
pub use wrapper::*;

use super::{FieldContextData, RootFieldContext};

pub struct RootContainerField(RootHandle);

pub struct RootContainerFieldContextData {
    element: Element,
    context: RootContext,
}

impl RootContainerFieldContextData {
    pub fn element(&self) -> &Element {
        &self.element
    }
}

impl Container for RootContainerField {
    type Context = RootContext;
}

impl FieldContextData for RootContainerField {
    type Data = RootContainerFieldContextData;
}

impl Field for RootContainerField {
    type Handle = RootHandle;

    fn handle(&self) -> Self::Handle {
        self.0.clone()
    }
}

impl FieldContext<RootContainerField> for RootFieldContext<RootContainerField> {
    fn context(&mut self) -> &mut <RootContainerField as Container>::Context
    where
        RootContainerField: Container,
    {
        &mut self.data.context
    }
}
