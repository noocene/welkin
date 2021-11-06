use std::{borrow::Cow, cell::RefCell, collections::HashMap, fmt, rc::Rc};

use futures::channel::mpsc::Sender;
use ui_section::add_ui;
use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::{Element, Node};

mod fields;
use fields::*;

use crate::edit::{
    focus_contenteditable, focus_element, ui_section,
    zipper::{dynamic::DynamicTerm, encode, Cursor, DynamicCursor, Path, Term},
    DynamicVariance, UiSection, UiSectionVariance,
};

use super::{AbstractDynamic, Color, DynamicContext, Replace};

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
    needs_remove: Rc<RefCell<Option<Term<()>>>>,
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

#[derive(Clone)]
pub struct RootHandle(Uuid);

#[derive(Clone)]
pub struct RootVariance {
    focused: Rc<RefCell<HasFocus>>,
    needs_remove: Rc<RefCell<Option<Term<()>>>>,
    container: Element,
    sender: Sender<()>,
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
        *self.needs_remove.borrow_mut() = Some(Term::Hole(()));
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

impl Replace for RootContext {
    fn replace(&mut self, term: Term<()>) {
        self.container.remove();
        if self.is_root {
            *self.needs_remove.borrow_mut() = Some(term);
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
            *self.needs_remove.borrow_mut() = Some(Term::Hole(()));
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
        self.implementation.borrow().expand()
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

        if let Some(replace) = context.borrow().needs_remove.replace(None) {
            cursor = match cursor {
                Cursor::Dynamic(cursor) => {
                    Cursor::from_term_and_path(add_ui(replace, sender, true), cursor.up)
                }
                _ => todo!(),
            };
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
                needs_remove: Rc::new(RefCell::new(None)),
            })),
            implementation: Rc::new(RefCell::new(item)),
        }
    }
}
