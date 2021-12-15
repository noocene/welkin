use std::{cell::RefCell, rc::Rc};

use futures::StreamExt;
use uuid::Uuid;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::{Element, KeyboardEvent};

use crate::{
    edit::{
        dynamic::abst::{
            implementation::{fields::RootFieldData, HasFocus, RootContext, RootHandle},
            Field, FieldContext, FieldRead, FieldTriggersRemove, HasField, HasInitializedField,
        },
        zipper::{Cursor, Term},
        UiSection, UiSectionVariance,
    },
    make_scratchpad, ScratchpadContainer,
};

use super::{FieldContextData, RootFieldContext};

pub struct RootTermFieldContextData {
    container: Element,
    pad: Rc<RefCell<ScratchpadContainer>>,
    updated: Rc<RefCell<bool>>,
    triggers_remove: Rc<RefCell<bool>>,
}

impl RootTermFieldContextData {
    pub fn element(&self) -> &Element {
        &self.container
    }
}

pub struct RootTermField(RootHandle);

impl FieldContextData for RootTermField {
    type Data = RootTermFieldContextData;
}

impl Field for RootTermField {
    type Handle = RootHandle;

    fn handle(&self) -> Self::Handle {
        self.0.clone()
    }
}

impl FieldRead for RootTermField {
    type Data = Term<()>;
}

impl FieldTriggersRemove for RootTermField {}

impl HasInitializedField<Term<()>> for RootContext {}

impl FieldContext<RootTermField> for RootFieldContext<RootTermField> {
    fn read(&self) -> Option<Term<()>>
    where
        RootTermField: FieldRead,
    {
        if self.data.updated.replace(false) {
            let mut term = self.data.pad.borrow().data.borrow().clone();

            while !term.is_top() {
                term = term.ascend();
            }

            let term: Term<UiSection> = term.into();

            Some(term.clear_annotation())
        } else {
            None
        }
    }

    fn trigger_remove(&self) -> bool
    where
        RootTermField: FieldTriggersRemove,
    {
        self.data.triggers_remove.replace(false)
    }
}

impl HasField<Term<()>> for RootContext {
    type Initializer = Option<Term<()>>;
    type Field = RootTermField;

    fn create_field(&mut self, initializer: Self::Initializer) -> Self::Field {
        let handle = Uuid::new_v4();

        let sender = self.sender.clone().unwrap();

        let document = web_sys::window().unwrap().document().unwrap();

        let container = document.create_element("span").unwrap();

        container.class_list().add_2("abst-field", "term").unwrap();

        let term = initializer.unwrap_or(Term::Hole(()));
        let (_, mut pad) = make_scratchpad(term, |_| {}, |_| {}, false).unwrap();

        let mut receiver = pad.receiver.take().unwrap();

        let updated = Rc::new(RefCell::new(false));

        let pad = Rc::new(RefCell::new(pad));

        spawn_local({
            let mut sender = sender.clone();
            let updated = updated.clone();
            let pad = pad.clone();
            async move {
                while let Some(()) = receiver.next().await {
                    *updated.borrow_mut() = true;
                    spawn_local({
                        let mut sender = sender.clone();
                        async move {
                            let _ = sender.try_send(());
                        }
                    });
                }
            }
        });

        let triggers_remove = Rc::new(RefCell::new(false));

        {
            let focused = &mut *self.focused.borrow_mut();

            if let HasFocus::None = focused {
                *focused = HasFocus::Element(container.clone());
            }
        }

        let focus_closure = Closure::wrap(Box::new({
            let focused = self.focused.clone();
            let container = container.clone();
            move |_| {
                *focused.borrow_mut() = HasFocus::Element(container.clone());
            }
        }) as Box<dyn FnMut(JsValue)>);

        let keydown_closure = Closure::wrap(Box::new({
            let pad = pad.clone();
            let triggers_remove = triggers_remove.clone();
            let mut sender = sender.clone();
            move |e: JsValue| {
                let e: KeyboardEvent = e.dyn_into().unwrap();
                if e.code() == "Backspace" || e.code() == "Delete" {
                    let cursor = pad.borrow();
                    let cursor = cursor.data.borrow();

                    if let Cursor::Hole(data) = &*cursor {
                        let annotation = data.annotation();
                        if let UiSectionVariance::Hole { p, .. } = &annotation.variant {
                            if p.text_content().unwrap_or("".into()).is_empty() {
                                *triggers_remove.borrow_mut() = true;
                                let _ = sender.try_send(());
                                e.stop_propagation();
                            }
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(JsValue)>);

        let wrapper = pad.borrow().wrapper.clone();

        container.append_child(&wrapper).unwrap();

        container
            .add_event_listener_with_callback("focusin", focus_closure.as_ref().unchecked_ref())
            .unwrap();

        container
            .add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref())
            .unwrap();

        self.fields.insert(
            handle.clone(),
            RootFieldData::Term {
                context: RootFieldContext {
                    closures: vec![focus_closure, keydown_closure],
                    data: RootTermFieldContextData {
                        pad,
                        container,
                        triggers_remove,
                        updated,
                    },
                },
            },
        );

        RootTermField(RootHandle(handle))
    }

    fn field(&mut self, field: &Self::Field) -> &mut dyn FieldContext<Self::Field> {
        let handle = &(field.0).0;

        let field = self.fields.get_mut(handle).unwrap();

        match field {
            RootFieldData::Term { context } => context,
            _ => panic!(),
        }
    }
}
