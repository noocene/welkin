use std::{borrow::Cow, cell::RefCell, collections::HashMap, fmt, rc::Rc};

use futures::channel::mpsc::Sender;
use js_sys::Array;
use uuid::Uuid;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::{Element, KeyboardEvent, Node};

use crate::edit::{
    configure_contenteditable, focus_contenteditable, focus_element, ui_section,
    zipper::{dynamic::DynamicTerm, encode, Cursor, DynamicCursor, HoleCursor, Path, Term},
    DynamicVariance, UiSection, UiSectionVariance,
};

use super::{
    AbstractDynamic, Color, Container, DynamicContext, Field, FieldContext, FieldFocus, FieldRead,
    FieldSetColor, FieldTriggersAppend, FieldTriggersRemove, HasContainer, HasField,
    HasInitializedField, HasStatic, Static, VStack, Wrapper,
};

pub enum HasFocus {
    Element(Element),
    Editable(Element),
    None,
}

pub enum RootMutation {
    Remove,
}

pub struct RootContext {
    fields: HashMap<Uuid, RootFieldData>,
    container: Element,
    focused: Rc<RefCell<HasFocus>>,
    needs_focus: Rc<RefCell<bool>>,
    needs_remove: Rc<RefCell<bool>>,
    sender: Option<Sender<()>>,
    is_root: bool,
}

impl RootContext {
    fn new_child(&self, container: Element) -> Self {
        Self {
            fields: HashMap::new(),
            container,
            focused: self.focused.clone(),
            needs_focus: self.needs_focus.clone(),
            needs_remove: self.needs_remove.clone(),
            sender: self.sender.clone(),
            is_root: false,
        }
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
}

impl RootFieldData {
    fn container_element(&self) -> &Element {
        match self {
            RootFieldData::String { context } => &context.data.element,
            RootFieldData::Static { context } => &context.data.element,
            RootFieldData::Container { context } => &context.data.element,
        }
    }
}

#[derive(Clone)]
pub struct RootHandle(Uuid);

pub struct RootStringField(RootHandle);
pub struct RootStaticField(RootHandle);

#[derive(Clone)]
pub struct RootVariance {
    focused: Rc<RefCell<HasFocus>>,
    needs_remove: Rc<RefCell<bool>>,
    container: Element,
    sender: Sender<()>,
}

pub trait FieldContextData {
    type Data;
}

pub struct RootFieldContext<T: FieldContextData> {
    data: T::Data,
    closures: Vec<Closure<dyn FnMut(JsValue)>>,
}

pub struct RootStringFieldContextData {
    element: Element,
    triggers_remove: Rc<RefCell<bool>>,
    triggers_append: Rc<RefCell<bool>>,
    update: Rc<RefCell<Option<String>>>,
}

pub struct RootContainerFieldContextData {
    element: Element,
    context: RootContext,
}

pub struct RootStaticFieldContextData {
    element: Element,
}

pub struct RootContainerField(RootHandle);

impl Container for RootContainerField {
    type Context = RootContext;
}

impl FieldContextData for RootContainerField {
    type Data = RootContainerFieldContextData;
}

impl FieldContextData for RootStringField {
    type Data = RootStringFieldContextData;
}

impl FieldContextData for RootStaticField {
    type Data = RootStaticFieldContextData;
}

fn color_to_class(color: Color) -> &'static str {
    match color {
        Color::Data => "color-data",
        Color::Reference => "color-reference",
        Color::Binding => "color-binding",
        Color::Hole => "color-hole",
        Color::Type => "color-type",
    }
}

const COLORS: &'static [Color] = &[
    Color::Data,
    Color::Reference,
    Color::Binding,
    Color::Hole,
    Color::Type,
];

impl FieldContext<RootStringField> for RootFieldContext<RootStringField> {
    fn read(&self) -> Option<String>
    where
        RootStringField: FieldRead,
    {
        self.data.update.borrow_mut().take()
    }

    fn set_color(&mut self, color: Color)
    where
        RootStringField: FieldSetColor,
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
}

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

impl FieldContext<RootContainerField> for RootFieldContext<RootContainerField> {
    fn context(&mut self) -> &mut <RootContainerField as Container>::Context
    where
        RootContainerField: Container,
    {
        &mut self.data.context
    }
}

impl DynamicVariance for RootVariance {
    fn box_clone(&self) -> Box<dyn DynamicVariance> {
        Box::new(self.clone())
    }

    fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        todo!()
    }

    fn focus(&self) {
        match &*self.focused.borrow() {
            HasFocus::Element(el) => {
                focus_element(el, false);
            }
            HasFocus::Editable(el) => {
                focus_contenteditable(el, false);
            }
            HasFocus::None => {}
        }
    }

    fn remove(&self) {
        self.container.remove();
        *self.needs_remove.borrow_mut() = true;
        let _ = self.sender.clone().try_send(());
    }

    fn focused_el(&self) -> Cow<'_, Element> {
        Cow::Owned(
            match &*self.focused.borrow() {
                HasFocus::Element(el) => el,
                HasFocus::Editable(el) => el,
                HasFocus::None => panic!("no focused element"),
            }
            .clone(),
        )
    }
}

impl Field for RootStringField {
    type Handle = RootHandle;

    fn handle(&self) -> Self::Handle {
        self.0.clone()
    }
}

impl Field for RootStaticField {
    type Handle = RootHandle;

    fn handle(&self) -> Self::Handle {
        self.0.clone()
    }
}

impl Field for RootContainerField {
    type Handle = RootHandle;

    fn handle(&self) -> Self::Handle {
        self.0.clone()
    }
}

impl FieldRead for RootStringField {
    type Data = String;
}

impl FieldSetColor for RootStringField {}

impl FieldSetColor for RootStaticField {}

impl FieldTriggersRemove for RootStringField {}

impl FieldTriggersAppend for RootStringField {}

impl FieldFocus for RootStringField {}

impl HasInitializedField<String> for RootContext {}

impl HasStatic for RootContext {}

impl HasField<VStack> for RootContext {
    type Field = RootContainerField;

    type Initializer = ();

    fn create_field(&mut self, initializer: Self::Initializer) -> Self::Field {
        let sender = self.sender.clone().unwrap();

        let handle = Uuid::new_v4();

        let document = web_sys::window().unwrap().document().unwrap();
        let span = document.create_element("div").unwrap();

        span.class_list().add_2("abst-field", "vstack").unwrap();

        self.fields.insert(
            handle.clone(),
            RootFieldData::Container {
                context: RootFieldContext {
                    closures: vec![],
                    data: RootContainerFieldContextData {
                        element: span.clone(),
                        context: self.new_child(span),
                    },
                },
            },
        );

        RootContainerField(RootHandle(handle))
    }

    fn field(&mut self, field: &Self::Field) -> &mut dyn FieldContext<Self::Field> {
        let handle = &(field.0).0;

        let field = self.fields.get_mut(handle).unwrap();

        match field {
            RootFieldData::Container { context } => context,
            _ => panic!(),
        }
    }
}

impl HasContainer<VStack> for RootContext {}

impl HasField<Wrapper> for RootContext {
    type Field = RootContainerField;

    type Initializer = ();

    fn create_field(&mut self, initializer: Self::Initializer) -> Self::Field {
        let sender = self.sender.clone().unwrap();

        let handle = Uuid::new_v4();

        let document = web_sys::window().unwrap().document().unwrap();
        let span = document.create_element("div").unwrap();

        span.class_list().add_2("abst-field", "wrapper").unwrap();

        self.fields.insert(
            handle.clone(),
            RootFieldData::Container {
                context: RootFieldContext {
                    closures: vec![],
                    data: RootContainerFieldContextData {
                        element: span.clone(),
                        context: self.new_child(span),
                    },
                },
            },
        );

        RootContainerField(RootHandle(handle))
    }

    fn field(&mut self, field: &Self::Field) -> &mut dyn FieldContext<Self::Field> {
        let handle = &(field.0).0;

        let field = self.fields.get_mut(handle).unwrap();

        match field {
            RootFieldData::Container { context } => context,
            _ => panic!(),
        }
    }
}

impl HasContainer<Wrapper> for RootContext {}

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
        configure_contenteditable(&span);

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

        let input_closure = Closure::wrap(Box::new({
            let span = span.clone();
            let sender = RefCell::new(sender.clone());
            let update = update.clone();
            move |_| {
                *update.borrow_mut() = Some(span.text_content().unwrap_or(String::new()));
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
                    },
                },
            },
        );

        RootStringField(RootHandle(handle))
    }

    fn field(&mut self, field: &Self::Field) -> &mut dyn super::FieldContext<Self::Field> {
        let handle = &(field.0).0;

        let field = self.fields.get_mut(handle).unwrap();

        match field {
            RootFieldData::String { context } => context,
            _ => panic!(),
        }
    }
}

impl DynamicContext for RootContext {
    type Handle = RootHandle;

    fn remove_field(&mut self, field: Box<dyn super::Field<Handle = Self::Handle>>) {
        todo!()
    }

    fn append_field(&mut self, field: Self::Handle) {
        let uuid = field.0;

        let field = self.fields.get(&uuid).unwrap();

        let element = field.container_element();

        if !self.container.contains(Some(element)) {
            self.container.append_child(element).unwrap();
        }
    }

    fn append_field_after(&mut self, field: Self::Handle, after: Self::Handle) {
        let uuid = field.0;

        let field = self.fields.get(&uuid).unwrap();

        let uuid = after.0;

        let after = self.fields.get(&uuid).unwrap();

        let element = field.container_element();

        let after = after.container_element();

        if !self.container.contains(Some(element)) {
            after.after_with_node_1(element).unwrap();
        }
    }

    fn remove(&mut self) {
        self.container.remove();
        if self.is_root {
            *self.needs_remove.borrow_mut() = true;
        }
    }
}

#[derive(Clone)]
pub struct Root {
    context: Rc<RefCell<RootContext>>,
    implementation: Rc<RefCell<dyn AbstractDynamic<RootContext>>>,
}

impl DynamicTerm<()> for Root {
    fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        todo!()
    }

    fn index(&self) -> u8 {
        // if/when I change the surrounding logic, this is 'i' for invocation historically
        'i' as u8
    }

    fn encode(self: Box<Self>) -> Vec<u8> {
        encode(self.implementation.borrow().encode()).unwrap()
    }

    fn expand(self: Box<Self>) -> Term<()> {
        todo!()
    }

    fn box_clone(&self) -> Box<dyn DynamicTerm<()>> {
        Box::new(self.clone())
    }

    fn add_ui(
        mut self: Box<Self>,
        sender: &Sender<()>,
    ) -> (UiSection, Box<dyn DynamicTerm<UiSection>>) {
        self.context
            .borrow_mut()
            .sender
            .get_or_insert_with(|| sender.clone());

        self.implementation
            .borrow_mut()
            .render(&mut *self.context.borrow_mut());

        let focused = self.context.borrow().focused.clone();
        let needs_remove = self.context.borrow().needs_remove.clone();
        let container = self.context.borrow().container.clone();

        (
            UiSection::new(UiSectionVariance::Dynamic(Box::new(RootVariance {
                sender: sender.clone(),
                needs_remove,
                focused,
                container,
            }))),
            Box::new(self),
        )
    }

    fn apply_mutations(
        self: Box<Self>,
        up: Path<UiSection>,
        annotation: Box<dyn DynamicVariance>,
        focused: &mut Option<Cursor<UiSection>>,
        sender: &Sender<()>,
    ) -> Result<Cursor<UiSection>, JsValue> {
        todo!()
    }

    fn render_to(
        &self,
        up: &Path<UiSection>,
        annotation: &dyn DynamicVariance,
        node: &Node,
    ) -> Result<(), JsValue> {
        todo!()
    }

    fn clear_annotation(self: Box<Self>) -> Box<dyn DynamicTerm<()>> {
        Box::new(*self)
    }
}

impl DynamicTerm<UiSection> for Root {
    fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        todo!()
    }

    fn index(&self) -> u8 {
        todo!()
    }

    fn encode(self: Box<Self>) -> Vec<u8> {
        todo!()
    }

    fn expand(self: Box<Self>) -> Term<()> {
        self.implementation.borrow().expand()
    }

    fn box_clone(&self) -> Box<dyn DynamicTerm<UiSection>> {
        Box::new(self.clone())
    }

    fn add_ui(
        self: Box<Self>,
        sender: &Sender<()>,
    ) -> (UiSection, Box<dyn DynamicTerm<UiSection>>) {
        todo!()
    }

    fn apply_mutations(
        self: Box<Self>,
        up: Path<UiSection>,
        annotation: Box<dyn DynamicVariance>,
        focused: &mut Option<Cursor<UiSection>>,
        sender: &Sender<()>,
    ) -> Result<Cursor<UiSection>, JsValue>
    where
        Term<UiSection>: Into<Term<UiSection>>,
    {
        let context = self.context.clone();

        self.implementation
            .borrow_mut()
            .render(&mut *self.context.borrow_mut());

        let mut cursor = Cursor::Dynamic(DynamicCursor {
            up: up.clone(),
            term: Box::new(*self),
            annotation: UiSection::new(UiSectionVariance::Dynamic(annotation)),
        });

        if context.borrow().needs_remove.replace(false) {
            cursor = Cursor::Hole(match cursor {
                Cursor::Dynamic(_) => {
                    HoleCursor::new(up.clone(), ui_section(Term::Hole(()), sender))
                }
                _ => todo!(),
            });
        }

        if context.borrow().needs_focus.replace(false) {
            *focused = Some(cursor.clone());
        }

        Ok(cursor)
    }

    fn render_to(
        &self,
        up: &Path<UiSection>,
        annotation: &dyn DynamicVariance,
        node: &Node,
    ) -> Result<(), JsValue> {
        self.implementation
            .borrow_mut()
            .render(&mut *self.context.borrow_mut());

        let mut last_el: Option<&Element> = None;

        let container = &self.context.borrow().container;

        if !node.contains(Some(container)) {
            node.append_child(container).unwrap();
        }

        Ok(())
    }

    fn clear_annotation(self: Box<Self>) -> Box<dyn DynamicTerm<()>> {
        Box::new(*self)
    }
}

impl Root {
    pub fn new<T: AbstractDynamic<RootContext> + 'static>(item: T) -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let container = document.create_element("div").unwrap();

        container.class_list().add_1("abst").unwrap();

        Self {
            context: Rc::new(RefCell::new(RootContext {
                focused: Rc::new(RefCell::new(HasFocus::None)),
                fields: HashMap::new(),
                is_root: true,
                sender: None,
                needs_focus: Rc::new(RefCell::new(false)),
                container,
                needs_remove: Rc::new(RefCell::new(false)),
            })),
            implementation: Rc::new(RefCell::new(item)),
        }
    }
}
