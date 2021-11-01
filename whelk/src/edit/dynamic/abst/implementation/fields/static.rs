use js_sys::Array;
use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::Element;

use crate::edit::dynamic::abst::{
    implementation::{
        color_to_class,
        fields::{RootFieldContext, RootFieldData},
        RootContext, RootHandle, COLORS,
    },
    Color, Field, FieldContext, FieldSetColor, HasField, HasStatic, Static,
};

use super::FieldContextData;

pub struct RootStaticField(RootHandle);

pub struct RootStaticFieldContextData {
    element: Element,
}

impl RootStaticFieldContextData {
    pub fn element(&self) -> &Element {
        &self.element
    }
}

impl FieldContextData for RootStaticField {
    type Data = RootStaticFieldContextData;
}

impl FieldSetColor for RootStaticField {}

impl HasStatic for RootContext {}

impl FieldContext<RootStaticField> for RootFieldContext<RootStaticField> {
    fn set_color(&mut self, color: Color)
    where
        RootStaticField: FieldSetColor,
    {
        let colors = COLORS
            .iter()
            .cloned()
            .map(|a| JsValue::from(color_to_class(a)))
            .collect::<Array>();
        self.data.element.class_list().remove(&colors).unwrap();
        self.data
            .element
            .class_list()
            .add_1(color_to_class(color))
            .unwrap();
    }
}

impl Field for RootStaticField {
    type Handle = RootHandle;

    fn handle(&self) -> Self::Handle {
        self.0.clone()
    }
}

impl HasField<Static> for RootContext {
    type Field = RootStaticField;

    type Initializer = Static;

    fn create_field(&mut self, initializer: Self::Initializer) -> Self::Field {
        let sender = self.sender.clone().unwrap();

        let handle = Uuid::new_v4();

        let document = web_sys::window().unwrap().document().unwrap();
        let span = document.create_element("span").unwrap();

        span.set_text_content(Some(initializer.0.as_str()));

        span.class_list().add_2("abst-field", "static").unwrap();

        self.fields.insert(
            handle.clone(),
            RootFieldData::Static {
                context: RootFieldContext {
                    closures: vec![],
                    data: RootStaticFieldContextData { element: span },
                },
            },
        );

        RootStaticField(RootHandle(handle))
    }

    fn field(&mut self, field: &Self::Field) -> &mut dyn FieldContext<Self::Field> {
        let handle = &(field.0).0;

        let field = self.fields.get_mut(handle).unwrap();

        match field {
            RootFieldData::Static { context } => context,
            _ => panic!(),
        }
    }
}
