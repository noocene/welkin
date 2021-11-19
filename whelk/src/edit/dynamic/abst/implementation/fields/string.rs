use std::{cell::RefCell, rc::Rc};

use js_sys::Array;
use uuid::Uuid;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::{Element, InputEvent, KeyboardEvent};

use crate::edit::{
    configure_contenteditable,
    dynamic::abst::{
        implementation::{
            color_to_class, fields::RootFieldData, HasFocus, RootContext, RootHandle, COLORS,
        },
        Color, Field, FieldContext, FieldFilter, FieldFocus, FieldRead, FieldSetColor,
        FieldTriggersAppend, FieldTriggersRemove, HasField, HasInitializedField,
    },
    focus_contenteditable,
};

use super::{FieldContextData, RootFieldContext};

pub struct RootStringField(RootHandle);

pub struct RootStringFieldContextData {
    element: Element,
    triggers_remove: Rc<RefCell<bool>>,
    triggers_append: Rc<RefCell<bool>>,
    update: Rc<RefCell<Option<String>>>,
    filter: Rc<RefCell<Box<dyn Fn(char) -> bool>>>,
}

impl RootStringFieldContextData {
    pub fn element(&self) -> &Element {
        &self.element
    }
}

impl FieldContextData for RootStringField {
    type Data = RootStringFieldContextData;
}

impl FieldSetColor for RootStringField {}

impl FieldTriggersRemove for RootStringField {}

impl FieldTriggersAppend for RootStringField {}

impl FieldFocus for RootStringField {}

impl FieldFilter for RootStringField {
    type Element = char;
}

impl FieldRead for RootStringField {
    type Data = String;
}

impl FieldContext<RootStringField> for RootFieldContext<RootStringField> {
    fn read(&self) -> Option<String> {
        self.data.update.borrow_mut().take()
    }

    fn set_color(&mut self, color: Color) {
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

    fn trigger_remove(&self) -> bool {
        self.data.triggers_remove.replace(false)
    }

    fn trigger_append(&self) -> bool {
        self.data.triggers_append.replace(false)
    }

    fn focus(&mut self) {
        let element = self.data.element.clone();

        spawn_local(async move {
            focus_contenteditable(&element, true);
        });
    }

    fn set_filter(&self, predicate: Box<dyn Fn(char) -> bool>) {
        *self.data.filter.borrow_mut() = predicate;
    }
}

impl HasInitializedField<String> for RootContext {}

impl Field for RootStringField {
    type Handle = RootHandle;

    fn handle(&self) -> Self::Handle {
        self.0.clone()
    }
}

impl HasField<String> for RootContext {
    type Field = RootStringField;
    type Initializer = Option<String>;

    fn create_field(&mut self, initializer: Option<String>) -> Self::Field {
        let sender = self.sender.clone().unwrap();

        let handle = Uuid::new_v4();

        let triggers_remove = Rc::new(RefCell::new(false));
        let triggers_append = Rc::new(RefCell::new(false));

        let needs_focus = self.needs_focus.clone();

        let update = Rc::new(RefCell::new(None));

        let document = web_sys::window().unwrap().document().unwrap();
        let span = document.create_element("span").unwrap();

        span.set_text_content(initializer.as_ref().map(String::as_str));

        span.class_list().add_2("abst-field", "string").unwrap();

        if self.editable {
            configure_contenteditable(&span);
        }

        let focused = &mut *self.focused.borrow_mut();

        if let HasFocus::None = focused {
            *focused = HasFocus::Editable(span.clone());
        }

        let keydown_closure = Closure::wrap(Box::new({
            let span = span.clone();
            let mut sender = sender.clone();
            let triggers_remove = triggers_remove.clone();
            let triggers_append = triggers_append.clone();
            move |e: JsValue| {
                let e: KeyboardEvent = e.dyn_into().unwrap();

                if (e.code() == "Backspace" || e.code() == "Delete")
                    && span.text_content().unwrap_or("".into()).len() == 0
                {
                    *triggers_remove.borrow_mut() = true;
                    let _ = sender.try_send(());
                    e.prevent_default();
                    e.stop_propagation();
                } else if e.code() == "Enter" {
                    *triggers_append.borrow_mut() = true;
                    let _ = sender.try_send(());
                    e.prevent_default();
                    e.stop_propagation();
                }
            }
        }) as Box<dyn FnMut(JsValue)>);

        span.add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref())
            .unwrap();

        let focus_closure = Closure::wrap(Box::new({
            let needs_focus = needs_focus.clone();
            let sender = RefCell::new(sender.clone());
            let span = span.clone();
            let focused = self.focused.clone();
            move |_| {
                focus_contenteditable(&span, true);
                *needs_focus.borrow_mut() = true;
                if let Ok(mut focused) = focused.try_borrow_mut() {
                    *focused = HasFocus::Editable(span.clone());
                }
                let _ = sender.borrow_mut().try_send(());
            }
        }) as Box<dyn FnMut(JsValue)>);

        span.add_event_listener_with_callback("focus", focus_closure.as_ref().unchecked_ref())
            .unwrap();

        let filter: Rc<RefCell<Box<dyn Fn(char) -> bool>>> =
            Rc::new(RefCell::new(Box::new(|_| true)));

        let input_closure = Closure::wrap(Box::new({
            let span = span.clone();
            let sender = RefCell::new(sender.clone());
            let update = update.clone();
            let filter = filter.clone();
            move |e: JsValue| {
                let e: InputEvent = e.dyn_into().unwrap();
                let filter = &**filter.borrow();
                let mut text_content = span.text_content().unwrap_or(String::new());
                text_content = text_content.chars().filter(|char| filter(*char)).collect();
                span.set_text_content(Some(&text_content));
                *update.borrow_mut() = Some(text_content);
                let _ = sender.borrow_mut().try_send(());
            }
        }) as Box<dyn FnMut(JsValue)>);

        span.add_event_listener_with_callback("input", input_closure.as_ref().unchecked_ref())
            .unwrap();

        self.fields.insert(
            handle.clone(),
            RootFieldData::String {
                context: RootFieldContext {
                    closures: vec![keydown_closure, focus_closure, input_closure],
                    data: RootStringFieldContextData {
                        element: span,
                        update,
                        triggers_remove,
                        triggers_append,
                        filter,
                    },
                },
            },
        );

        RootStringField(RootHandle(handle))
    }

    fn field(&mut self, field: &Self::Field) -> &mut dyn FieldContext<Self::Field> {
        let handle = &(field.0).0;

        let field = self.fields.get_mut(handle).unwrap();

        match field {
            RootFieldData::String { context } => context,
            _ => panic!(),
        }
    }
}
