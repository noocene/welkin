use std::{cell::RefCell, fmt, rc::Rc};

use downcast_rs::{impl_downcast, Downcast};
use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    task::{noop_waker, Context},
    Future, StreamExt,
};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{ClipboardEvent, Element, FocusEvent, HtmlElement, KeyboardEvent, Node};
use zipper::{Cursor, Term};

pub mod dynamic;
pub mod zipper;

mod mutations {
    use super::{Term, UiSection};

    #[derive(Clone, Debug)]
    pub enum ReferenceMutation {
        Update(String),
        Focus,
        Remove,
    }

    #[derive(Clone, Debug)]
    pub enum LambdaMutation {
        Update(String),
        Focus,
        Remove,
        ToggleErased,
    }

    #[derive(Clone, Debug)]
    pub enum HoleMutation {
        Focus,
        Replace(Term<UiSection>),
        ToParent,
    }

    #[derive(Clone, Debug)]
    pub enum ApplicationMutation {
        Focus,
        Remove,
        ToggleErased,
    }

    #[derive(Clone, Debug)]
    pub enum UniverseMutation {
        Focus,
        Remove,
    }

    #[derive(Clone, Debug)]
    pub enum WrapMutation {
        Focus,
        Remove,
    }

    #[derive(Clone, Debug)]
    pub enum PutMutation {
        Focus,
        Remove,
    }

    #[derive(Clone, Debug)]
    pub enum DuplicationMutation {
        Focus,
        Remove,
        Update(String),
    }

    #[derive(Clone, Debug)]
    pub enum FunctionMutation {
        Focus,
        FocusSelf,
        Remove,
        Update(String),
        UpdateSelf(String),
        ToggleErased,
    }
}
pub use mutations::*;

use crate::edit::{dynamic::Def, zipper::TermData};

use self::zipper::{dynamic::Dynamic, RefCount};

#[allow(dead_code)]
fn focus_contenteditable(p: &Element, always: bool) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    if document.active_element() == Some(p.clone()) && !always {
        return;
    }

    let selection = window.get_selection().unwrap().unwrap();
    let range = document.create_range().unwrap();
    range.set_start(p, 0).unwrap();
    range
        .set_end(p, Node::from(p.clone()).child_nodes().length())
        .unwrap();
    selection.remove_all_ranges().unwrap();
    selection.add_range(&range).unwrap();
}

#[allow(dead_code)]
fn focus_element(p: &Element, always: bool) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let selection = window.get_selection().unwrap().unwrap();
    selection.remove_all_ranges().unwrap();

    if document.active_element() == Some(p.clone()) && !always {
        return;
    }

    p.dyn_ref::<HtmlElement>().unwrap().focus().unwrap();
}

#[derive(Clone, Debug)]
pub enum UiSectionVariance {
    Lambda {
        p: Element,
        span: Element,
        container: Node,
        closures: Rc<Vec<Closure<dyn FnMut(JsValue)>>>,
        mutations: Rc<RefCell<Vec<LambdaMutation>>>,
    },
    Function {
        container: Element,
        span: Element,
        self_span: Element,
        closures: Rc<Vec<Closure<dyn FnMut(JsValue)>>>,
        self_focused: Rc<RefCell<bool>>,
        mutations: Rc<RefCell<Vec<FunctionMutation>>>,
    },
    Application {
        container: Element,
        closures: Rc<Vec<Closure<dyn FnMut(JsValue)>>>,
        mutations: Rc<RefCell<Vec<ApplicationMutation>>>,
    },
    Reference {
        p: Element,
        mutations: Rc<RefCell<Vec<ReferenceMutation>>>,
        closures: Rc<Vec<Closure<dyn FnMut(JsValue)>>>,
    },
    Hole {
        p: Element,
        mutations: Rc<RefCell<Vec<HoleMutation>>>,
        closures: Rc<Vec<Closure<dyn FnMut(JsValue)>>>,
    },
    Universe {
        p: Element,
        mutations: Rc<RefCell<Vec<UniverseMutation>>>,
        closures: Rc<Vec<Closure<dyn FnMut(JsValue)>>>,
    },
    Wrap {
        container: Element,
        content: Element,
        mutations: Rc<RefCell<Vec<WrapMutation>>>,
        closures: Rc<Vec<Closure<dyn FnMut(JsValue)>>>,
    },
    Put {
        container: Element,
        content: Element,
        mutations: Rc<RefCell<Vec<PutMutation>>>,
        closures: Rc<Vec<Closure<dyn FnMut(JsValue)>>>,
    },
    Duplication {
        container: Element,
        span: Element,
        closures: Rc<Vec<Closure<dyn FnMut(JsValue)>>>,
        mutations: Rc<RefCell<Vec<DuplicationMutation>>>,
    },
    Dynamic(Box<dyn DynamicVariance>),
}

impl Clone for Box<dyn DynamicVariance> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

impl fmt::Debug for Box<dyn DynamicVariance> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug(f)
    }
}

pub trait DynamicVariance: Downcast {
    fn box_clone(&self) -> Box<dyn DynamicVariance>;
    fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result;
    fn focus(&self);
    fn remove(&self);
    fn focused_el(&self) -> &Element;
}

impl_downcast!(DynamicVariance);

#[derive(Debug, Clone)]
pub struct UiSection {
    variant: UiSectionVariance,
    annotation: Rc<RefCell<Term<(), RefCount>>>,
}

impl UiSection {
    fn new(variant: UiSectionVariance) -> Self {
        UiSection {
            variant,
            annotation: Rc::new(RefCell::new(Term::Hole(()))),
        }
    }
}

impl UiSection {
    pub fn trigger_remove(&self, sender: &Sender<()>) {
        match &self.variant {
            UiSectionVariance::Lambda {
                container,
                mutations,
                ..
            } => {
                container.dyn_ref::<Element>().unwrap().remove();
                mutations.borrow_mut().push(LambdaMutation::Remove)
            }
            UiSectionVariance::Application {
                container,

                mutations,
                ..
            } => {
                container.remove();
                mutations.borrow_mut().push(ApplicationMutation::Remove)
            }
            UiSectionVariance::Reference { p, mutations, .. } => {
                p.remove();
                mutations.borrow_mut().push(ReferenceMutation::Remove)
            }
            UiSectionVariance::Hole { .. } => {}
            UiSectionVariance::Universe { p, mutations, .. } => {
                p.remove();
                mutations.borrow_mut().push(UniverseMutation::Remove)
            }
            UiSectionVariance::Wrap {
                mutations,
                container,
                ..
            } => {
                container.remove();
                mutations.borrow_mut().push(WrapMutation::Remove);
            }
            UiSectionVariance::Put {
                mutations,
                container,
                ..
            } => {
                container.remove();
                mutations.borrow_mut().push(PutMutation::Remove);
            }
            UiSectionVariance::Duplication {
                mutations,
                container,
                ..
            } => {
                container.remove();
                mutations.borrow_mut().push(DuplicationMutation::Remove);
            }
            UiSectionVariance::Function {
                mutations,
                container,
                ..
            } => {
                container.remove();
                mutations.borrow_mut().push(FunctionMutation::Remove);
            }
            UiSectionVariance::Dynamic(variance) => {
                variance.remove();
            }
        }
        let _ = sender.clone().try_send(());
    }

    pub fn focus(&self) {
        match &self.variant {
            UiSectionVariance::Lambda { span, .. } => {
                focus_contenteditable(span, false);
            }
            UiSectionVariance::Reference { p, .. } => {
                focus_contenteditable(p, false);
            }
            UiSectionVariance::Hole { p, .. } => {
                focus_contenteditable(p, false);
            }
            UiSectionVariance::Application { container, .. } => {
                focus_element(container, false);
            }
            UiSectionVariance::Universe { p, .. } => {
                focus_element(p, false);
            }
            UiSectionVariance::Wrap { container, .. } => {
                focus_element(container, false);
            }
            UiSectionVariance::Put { container, .. } => {
                focus_element(container, false);
            }
            UiSectionVariance::Duplication { span, .. } => {
                focus_contenteditable(span, false);
            }
            UiSectionVariance::Function {
                span,
                self_span,
                self_focused,
                ..
            } => {
                focus_contenteditable(
                    if *self_focused.borrow() {
                        self_span
                    } else {
                        span
                    },
                    false,
                );
            }
            UiSectionVariance::Dynamic(variance) => {
                variance.focus();
            }
        }
    }

    pub fn render(&self, into: &Node, cursor: &Cursor<UiSection>) -> Result<Option<Node>, JsValue> {
        Ok(match &self.variant {
            UiSectionVariance::Lambda {
                container, span, ..
            } => match cursor {
                Cursor::Lambda(cursor) => {
                    if let Some(name) = cursor.name() {
                        span.set_text_content(Some(&name));
                    } else {
                        span.set_text_content(Some(""));
                    }

                    if cursor.erased() {
                        container
                            .clone()
                            .dyn_into::<Element>()?
                            .class_list()
                            .add_1("erased")?;
                    } else {
                        container
                            .clone()
                            .dyn_into::<Element>()?
                            .class_list()
                            .remove_1("erased")?;
                    }

                    if !into.contains(Some(&container)) {
                        into.append_child(&container)?;
                    }

                    Some(container.clone().into())
                }
                _ => panic!(),
            },
            UiSectionVariance::Application { container, .. } => match cursor {
                Cursor::Application(cursor) => {
                    if !into.contains(Some(&container)) {
                        into.append_child(&container)?;
                    }

                    if cursor.erased() {
                        container
                            .clone()
                            .dyn_into::<Element>()?
                            .class_list()
                            .add_1("erased")?;
                    } else {
                        container
                            .clone()
                            .dyn_into::<Element>()?
                            .class_list()
                            .remove_1("erased")?;
                    }

                    Some(container.clone().into())
                }
                _ => panic!(),
            },
            UiSectionVariance::Reference { p, .. } => match cursor {
                Cursor::Reference(c) => {
                    let name = c.name();

                    p.set_text_content(Some(name));

                    if let Some(_) = cursor.context().position(|a| {
                        if let Some(a) = a {
                            return name == &a;
                        }
                        false
                    }) {
                        p.class_list().add_1("var")?;
                        p.class_list().remove_1("ref")?;
                    } else {
                        p.class_list().add_1("ref")?;
                        p.class_list().remove_1("var")?;
                    }

                    if !into.contains(Some(&p)) {
                        into.append_child(&p)?;
                    }

                    None
                }
                _ => panic!(),
            },
            UiSectionVariance::Hole { p, .. } => match cursor {
                Cursor::Hole(_) => {
                    if !into.contains(Some(&p)) {
                        into.append_child(&p)?;
                    }

                    None
                }
                _ => panic!(),
            },
            UiSectionVariance::Universe { p, .. } => match cursor {
                Cursor::Universe(_) => {
                    if !into.contains(Some(&p)) {
                        into.append_child(&p)?;
                    }

                    None
                }
                _ => panic!(),
            },
            UiSectionVariance::Wrap {
                container, content, ..
            } => match cursor {
                Cursor::Wrap(_) => {
                    if !into.contains(Some(&container)) {
                        into.append_child(&container)?;
                    }

                    Some(content.clone().into())
                }
                _ => panic!(),
            },
            UiSectionVariance::Put {
                container, content, ..
            } => match cursor {
                Cursor::Put(_) => {
                    if !into.contains(Some(&container)) {
                        into.append_child(&container)?;
                    }

                    Some(content.clone().into())
                }
                _ => panic!(),
            },
            UiSectionVariance::Duplication {
                container, span, ..
            } => match cursor {
                Cursor::Duplication(cursor) => {
                    if !into.contains(Some(&container)) {
                        into.append_child(&container)?;
                    }

                    if let Some(binder) = cursor.binder() {
                        span.set_text_content(Some(binder));
                    } else {
                        span.set_text_content(Some(""));
                    }

                    Some(container.clone().into())
                }
                _ => panic!(),
            },
            UiSectionVariance::Function {
                container,
                span,
                self_span,
                ..
            } => match cursor {
                Cursor::Function(cursor) => {
                    if let Some(name) = cursor.binder() {
                        span.set_text_content(Some(&name));
                    } else {
                        span.set_text_content(Some(""));
                    }

                    if let Some(name) = cursor.self_binder() {
                        self_span.set_text_content(Some(&name));
                    } else {
                        self_span.set_text_content(Some(""));
                    }

                    if cursor.erased() {
                        container
                            .clone()
                            .dyn_into::<Element>()?
                            .class_list()
                            .add_1("erased")?;
                    } else {
                        container
                            .clone()
                            .dyn_into::<Element>()?
                            .class_list()
                            .remove_1("erased")?;
                    }

                    if !into.contains(Some(&container)) {
                        into.append_child(&container)?;
                    }

                    Some(container.clone().into())
                }
                _ => panic!(),
            },
            UiSectionVariance::Dynamic(_) => match cursor {
                Cursor::Dynamic(_) => {
                    todo!()
                }
                _ => panic!(),
            },
        })
    }
}

fn configure_contenteditable(el: &Element) {
    el.set_attribute("contenteditable", "true").unwrap();
    el.set_attribute("tabindex", "0").unwrap();
    el.set_attribute("spellcheck", "false").unwrap();
    el.set_attribute("autocorrect", "off").unwrap();
    el.set_attribute("autocapitalize", "off").unwrap();
}

struct OnChangeWrapper {
    to_call: Vec<Box<dyn FnMut(&Cursor<UiSection>)>>,
}

impl OnChangeWrapper {
    fn new() -> Self {
        OnChangeWrapper { to_call: vec![] }
    }

    fn call(&mut self, data: &Cursor<UiSection>) {
        for call in &mut self.to_call {
            call(data);
        }
    }
}

impl std::fmt::Debug for OnChangeWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OnChangeWrapper").finish()
    }
}

struct OnRemoveWrapper {
    to_call: Vec<Box<dyn FnMut()>>,
}

impl OnRemoveWrapper {
    fn new() -> Self {
        OnRemoveWrapper { to_call: vec![] }
    }

    fn call(&mut self) {
        for call in &mut self.to_call {
            call();
        }
    }
}

impl std::fmt::Debug for OnRemoveWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OnRemoveWrapper").finish()
    }
}

#[derive(Debug)]
pub struct Scratchpad {
    data: Rc<RefCell<Cursor<UiSection>>>,
    needs_update: Receiver<()>,
    sender: Sender<()>,
    clipboard_event_handler: Closure<dyn FnMut(JsValue)>,
    focus_event_handler: Closure<dyn FnMut(JsValue)>,
    has_focus: Rc<RefCell<bool>>,
    target_node: Node,
    on_change: OnChangeWrapper,
}

fn add_ui<T>(term: Term<T>, sender: &Sender<()>) -> Term<UiSection> {
    let document = web_sys::window().unwrap().document().unwrap();

    match term {
        Term::Lambda {
            erased, name, body, ..
        } => Term::Lambda {
            erased,
            name,
            body: Box::new(add_ui(*body, sender)),
            annotation: {
                let mutations = Rc::new(RefCell::new(vec![]));

                let container = document.create_element("div").unwrap();
                container.class_list().add_1("lambdawrapper").unwrap();
                let p = document.create_element("p").unwrap();
                p.class_list().add_1("lambda").unwrap();
                container.append_child(&p).unwrap();

                let span = document.create_element("span").unwrap();
                span.class_list().add_2("lambda", "arg").unwrap();
                span.set_attribute("contenteditable", "true").unwrap();
                span.set_attribute("tabindex", "0").unwrap();
                configure_contenteditable(&span);

                p.append_child(&span).unwrap();

                let closure = Closure::wrap(Box::new({
                    let p = p.clone();
                    let mutations = mutations.clone();
                    let sender = RefCell::new(sender.clone());
                    move |_| {
                        mutations.borrow_mut().push(LambdaMutation::Update(
                            p.text_content().unwrap_or("".to_owned()),
                        ));
                        let _ = sender.borrow_mut().try_send(());
                    }
                }) as Box<dyn FnMut(JsValue)>);

                span.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())
                    .unwrap();

                let focus_closure = Closure::wrap(Box::new({
                    let mutations = mutations.clone();
                    let span = span.clone();
                    let sender = RefCell::new(sender.clone());
                    move |_| {
                        mutations.borrow_mut().push(LambdaMutation::Focus);
                        focus_contenteditable(&span, true);
                        let _ = sender.borrow_mut().try_send(());
                    }
                }) as Box<dyn FnMut(JsValue)>);

                span.add_event_listener_with_callback(
                    "focus",
                    focus_closure.as_ref().unchecked_ref(),
                )
                .unwrap();

                let keydown_closure = Closure::wrap(Box::new({
                    let mutations = mutations.clone();
                    let span = span.clone();
                    let container = container.clone();
                    let sender = RefCell::new(sender.clone());
                    move |e: JsValue| {
                        let e: KeyboardEvent = e.dyn_into().unwrap();
                        if (e.code() == "Backspace" || e.code() == "Delete")
                            && span.text_content().unwrap_or("".into()).len() == 0
                        {
                            mutations.borrow_mut().push(LambdaMutation::Remove);
                            container.remove();
                            e.stop_propagation();
                            let _ = sender.borrow_mut().try_send(());
                        } else if e.code() == "Backslash" {
                            e.prevent_default();
                            mutations.borrow_mut().push(LambdaMutation::ToggleErased);
                            e.stop_propagation();
                            let _ = sender.borrow_mut().try_send(());
                        }
                    }
                }) as Box<dyn FnMut(JsValue)>);

                span.add_event_listener_with_callback(
                    "keydown",
                    keydown_closure.as_ref().unchecked_ref(),
                )
                .unwrap();

                UiSection::new(UiSectionVariance::Lambda {
                    p,
                    span,
                    mutations,
                    container: container.into(),
                    closures: Rc::new(vec![closure, focus_closure, keydown_closure]),
                })
            },
        },
        Term::Application {
            erased,
            function,
            argument,
            ..
        } => Term::Application {
            erased,
            function: Box::new(add_ui(*function, sender)),
            argument: Box::new(add_ui(*argument, sender)),
            annotation: {
                let mutations = Rc::new(RefCell::new(vec![]));
                let container = document.create_element("div").unwrap();

                container.class_list().add_1("application").unwrap();
                container.set_attribute("tabindex", "0").unwrap();

                let argument = document.create_element("span").unwrap();

                argument.class_list().add_1("application-argument").unwrap();

                let function = document.create_element("span").unwrap();

                function.class_list().add_1("application-function").unwrap();

                let spacer = document.create_element("span").unwrap();
                spacer.class_list().add_1("application-spacer").unwrap();

                container.append_child(&function).unwrap();
                container.append_child(&argument).unwrap();
                container.append_child(&spacer).unwrap();

                let focus_closure = Closure::wrap(Box::new({
                    let mutations = mutations.clone();
                    let container = container.clone();
                    let sender = RefCell::new(sender.clone());
                    move |_| {
                        mutations.borrow_mut().push(ApplicationMutation::Focus);
                        focus_element(&container, true);
                        let _ = sender.borrow_mut().try_send(());
                    }
                }) as Box<dyn FnMut(JsValue)>);

                container
                    .add_event_listener_with_callback(
                        "focus",
                        focus_closure.as_ref().unchecked_ref(),
                    )
                    .unwrap();

                let keydown_closure = Closure::wrap(Box::new({
                    let mutations = mutations.clone();
                    let container = container.clone();
                    let sender = RefCell::new(sender.clone());
                    move |e: JsValue| {
                        let e: KeyboardEvent = e.dyn_into().unwrap();
                        if document.active_element().unwrap() == container {
                            if e.code() == "Backspace" || e.code() == "Delete" {
                                mutations.borrow_mut().push(ApplicationMutation::Remove);
                                container.remove();
                                e.stop_propagation();
                                let _ = sender.borrow_mut().try_send(());
                            } else if e.code() == "Backslash" {
                                e.prevent_default();
                                mutations
                                    .borrow_mut()
                                    .push(ApplicationMutation::ToggleErased);
                                e.stop_propagation();
                                let _ = sender.borrow_mut().try_send(());
                            }
                        }
                    }
                }) as Box<dyn FnMut(JsValue)>);

                container
                    .add_event_listener_with_callback(
                        "keydown",
                        keydown_closure.as_ref().unchecked_ref(),
                    )
                    .unwrap();

                UiSection::new(UiSectionVariance::Application {
                    container,
                    closures: Rc::new(vec![focus_closure, keydown_closure]),
                    mutations,
                })
            },
        },
        Term::Put(term, _) => Term::Put(Box::new(add_ui(*term, &sender)), {
            let mutations = Rc::new(RefCell::new(vec![]));

            let container = document.create_element("div").unwrap();
            container.class_list().add_1("put").unwrap();
            container.set_attribute("tabindex", "0").unwrap();

            let span = document.create_element("span").unwrap();
            span.class_list().add_1("put-inner").unwrap();

            let content = document.create_element("span").unwrap();
            content.class_list().add_1("put-content").unwrap();

            container.append_child(&span).unwrap();
            container.append_child(&content).unwrap();

            let focus_closure = Closure::wrap(Box::new({
                let mutations = mutations.clone();
                let container = container.clone();
                let sender = RefCell::new(sender.clone());
                move |_| {
                    mutations.borrow_mut().push(PutMutation::Focus);
                    focus_element(&container, true);
                    let _ = sender.borrow_mut().try_send(());
                }
            }) as Box<dyn FnMut(JsValue)>);

            container
                .add_event_listener_with_callback("focus", focus_closure.as_ref().unchecked_ref())
                .unwrap();

            let keydown_closure = Closure::wrap(Box::new({
                let mutations = mutations.clone();
                let container = container.clone();
                let sender = RefCell::new(sender.clone());
                move |e: JsValue| {
                    let e: KeyboardEvent = e.dyn_into().unwrap();
                    if document.active_element().unwrap() == container {
                        if e.code() == "Backspace" || e.code() == "Delete" {
                            mutations.borrow_mut().push(PutMutation::Remove);
                            container.remove();
                            e.stop_propagation();
                            let _ = sender.borrow_mut().try_send(());
                        }
                    }
                }
            }) as Box<dyn FnMut(JsValue)>);

            container
                .add_event_listener_with_callback(
                    "keydown",
                    keydown_closure.as_ref().unchecked_ref(),
                )
                .unwrap();

            UiSection::new(UiSectionVariance::Put {
                mutations,
                closures: Rc::new(vec![keydown_closure, focus_closure]),
                container,
                content,
            })
        }),
        Term::Duplication {
            binder,
            expression,
            body,
            ..
        } => Term::Duplication {
            binder,
            expression: Box::new(add_ui(*expression, &sender)),
            body: Box::new(add_ui(*body, &sender)),
            annotation: {
                let mutations = Rc::new(RefCell::new(vec![]));

                let container = document.create_element("div").unwrap();
                container.class_list().add_1("duplication").unwrap();

                let span = document.create_element("span").unwrap();
                span.class_list().add_1("duplication-inner").unwrap();

                span.set_attribute("contenteditable", "true").unwrap();
                span.set_attribute("tabindex", "0").unwrap();
                configure_contenteditable(&span);

                let closure = Closure::wrap(Box::new({
                    let span = span.clone();
                    let mutations = mutations.clone();
                    let sender = RefCell::new(sender.clone());
                    move |_| {
                        mutations.borrow_mut().push(DuplicationMutation::Update(
                            span.text_content().unwrap_or("".to_owned()),
                        ));
                        let _ = sender.borrow_mut().try_send(());
                    }
                }) as Box<dyn FnMut(JsValue)>);

                span.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())
                    .unwrap();

                let focus_closure = Closure::wrap(Box::new({
                    let mutations = mutations.clone();
                    let span = span.clone();
                    let sender = RefCell::new(sender.clone());
                    move |_| {
                        mutations.borrow_mut().push(DuplicationMutation::Focus);
                        focus_contenteditable(&span, true);
                        let _ = sender.borrow_mut().try_send(());
                    }
                }) as Box<dyn FnMut(JsValue)>);

                span.add_event_listener_with_callback(
                    "focus",
                    focus_closure.as_ref().unchecked_ref(),
                )
                .unwrap();

                let keydown_closure = Closure::wrap(Box::new({
                    let mutations = mutations.clone();
                    let span = span.clone();
                    let container = container.clone();
                    let sender = RefCell::new(sender.clone());
                    move |e: JsValue| {
                        let e: KeyboardEvent = e.dyn_into().unwrap();
                        if (e.code() == "Backspace" || e.code() == "Delete")
                            && span.text_content().unwrap_or("".into()).len() == 0
                        {
                            mutations.borrow_mut().push(DuplicationMutation::Remove);
                            container.remove();
                            e.stop_propagation();
                            let _ = sender.borrow_mut().try_send(());
                        }
                    }
                }) as Box<dyn FnMut(JsValue)>);

                span.add_event_listener_with_callback(
                    "keydown",
                    keydown_closure.as_ref().unchecked_ref(),
                )
                .unwrap();

                let expression = document.create_element("span").unwrap();
                expression
                    .class_list()
                    .add_1("duplication-expression")
                    .unwrap();

                let body = document.create_element("span").unwrap();
                body.class_list().add_1("duplication-body").unwrap();

                container.append_child(&span).unwrap();
                container.append_child(&expression).unwrap();
                container.append_child(&body).unwrap();

                UiSection::new(UiSectionVariance::Duplication {
                    mutations,
                    closures: Rc::new(vec![closure, keydown_closure, focus_closure]),
                    container,
                    span,
                })
            },
        },
        Term::Reference(name, _) => Term::Reference(name, {
            let mutations = Rc::new(RefCell::new(vec![]));

            let p = document.create_element("p").unwrap();

            p.class_list().add_1("reference").unwrap();

            configure_contenteditable(&p);

            let closure = Closure::wrap(Box::new({
                let p = p.clone();
                let mutations = mutations.clone();
                let sender = RefCell::new(sender.clone());
                move |_| {
                    mutations.borrow_mut().push({
                        let content = p.text_content().unwrap_or("".to_owned());
                        if content.is_empty() {
                            p.remove();
                            ReferenceMutation::Remove
                        } else {
                            ReferenceMutation::Update(content)
                        }
                    });
                    let _ = sender.borrow_mut().try_send(());
                }
            }) as Box<dyn FnMut(JsValue)>);

            p.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())
                .unwrap();

            let focus_closure = Closure::wrap(Box::new({
                let mutations = mutations.clone();
                let sender = RefCell::new(sender.clone());
                let p = p.clone();
                move |_| {
                    mutations.borrow_mut().push(ReferenceMutation::Focus);
                    focus_contenteditable(&p, true);
                    let _ = sender.borrow_mut().try_send(());
                }
            }) as Box<dyn FnMut(JsValue)>);

            p.add_event_listener_with_callback("focus", focus_closure.as_ref().unchecked_ref())
                .unwrap();

            let blur_closure = Closure::wrap(Box::new({
                let mutations = mutations.clone();
                let sender = RefCell::new(sender.clone());
                let p = p.clone();
                move |_| {
                    if p.text_content().unwrap_or("".into()).is_empty() {
                        if let Ok(mut r) = mutations.try_borrow_mut() {
                            p.remove();
                            r.push(ReferenceMutation::Remove);
                            let _ = sender.borrow_mut().try_send(());
                        }
                    }
                }
            }) as Box<dyn FnMut(JsValue)>);

            p.add_event_listener_with_callback("blur", blur_closure.as_ref().unchecked_ref())
                .unwrap();

            let keydown_closure = Closure::wrap(Box::new({
                let p = p.clone();
                move |e: JsValue| {
                    let e: KeyboardEvent = e.dyn_into().unwrap();
                    if document.active_element().unwrap() == p {
                        if (e.code() == "Backspace" || e.code() == "Delete")
                            && p.text_content().unwrap_or("".into()).is_empty()
                        {
                            e.stop_propagation();
                            p.dyn_ref::<HtmlElement>().unwrap().blur().unwrap();
                        } else if e.code() == "Escape" {
                            e.stop_propagation();
                            p.dyn_ref::<HtmlElement>().unwrap().blur().unwrap();
                        }
                    }
                }
            }) as Box<dyn FnMut(JsValue)>);

            p.add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref())
                .unwrap();

            UiSection::new(UiSectionVariance::Reference {
                p,
                mutations,
                closures: Rc::new(vec![closure, focus_closure, blur_closure, keydown_closure]),
            })
        }),

        Term::Universe(_) => Term::Universe({
            let mutations = Rc::new(RefCell::new(vec![]));

            let p = document.create_element("p").unwrap();

            p.class_list().add_1("universe").unwrap();
            p.set_attribute("tabindex", "0").unwrap();

            let focus_closure = Closure::wrap(Box::new({
                let mutations = mutations.clone();
                let p = p.clone();
                let sender = RefCell::new(sender.clone());
                move |_| {
                    mutations.borrow_mut().push(UniverseMutation::Focus);
                    focus_element(&p, true);
                    let _ = sender.borrow_mut().try_send(());
                }
            }) as Box<dyn FnMut(JsValue)>);

            p.add_event_listener_with_callback("focus", focus_closure.as_ref().unchecked_ref())
                .unwrap();

            let keydown_closure = Closure::wrap(Box::new({
                let mutations = mutations.clone();
                let p = p.clone();
                let sender = RefCell::new(sender.clone());
                move |e: JsValue| {
                    let e: KeyboardEvent = e.dyn_into().unwrap();
                    if document.active_element().unwrap() == p {
                        if e.code() == "Backspace" || e.code() == "Delete" {
                            mutations.borrow_mut().push(UniverseMutation::Remove);
                            p.remove();
                            e.stop_propagation();
                            let _ = sender.borrow_mut().try_send(());
                        }
                    }
                }
            }) as Box<dyn FnMut(JsValue)>);

            p.add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref())
                .unwrap();

            UiSection::new(UiSectionVariance::Universe {
                mutations,
                closures: Rc::new(vec![focus_closure, keydown_closure]),
                p,
            })
        }),
        Term::Function {
            erased,
            name,
            argument_type,
            return_type,
            self_name,
            ..
        } => Term::Function {
            erased,
            name,
            argument_type: Box::new(add_ui(*argument_type, &sender)),
            return_type: Box::new(add_ui(*return_type, &sender)),
            self_name,
            annotation: {
                let mutations = Rc::new(RefCell::new(vec![]));

                let container = document.create_element("div").unwrap();
                container.class_list().add_1("function").unwrap();

                let span = document.create_element("span").unwrap();
                span.class_list().add_1("function-name").unwrap();

                let self_span = document.create_element("sub").unwrap();
                self_span.class_list().add_1("function-self-name").unwrap();

                span.set_attribute("contenteditable", "true").unwrap();
                span.set_attribute("tabindex", "0").unwrap();
                configure_contenteditable(&span);

                self_span.set_attribute("contenteditable", "true").unwrap();
                self_span.set_attribute("tabindex", "0").unwrap();
                configure_contenteditable(&self_span);

                let argument_type_span = document.create_element("span").unwrap();
                argument_type_span
                    .class_list()
                    .add_1("function-argument-type")
                    .unwrap();
                let return_type_span = document.create_element("span").unwrap();
                return_type_span
                    .class_list()
                    .add_1("function-return-type")
                    .unwrap();

                container.append_child(&self_span).unwrap();
                container.append_child(&span).unwrap();
                container.append_child(&argument_type_span).unwrap();
                container.append_child(&return_type_span).unwrap();

                let input_closure = Closure::wrap(Box::new({
                    let span = span.clone();
                    let mutations = mutations.clone();
                    let sender = RefCell::new(sender.clone());
                    move |_| {
                        mutations.borrow_mut().push(FunctionMutation::Update(
                            span.text_content().unwrap_or("".to_owned()),
                        ));
                        let _ = sender.borrow_mut().try_send(());
                    }
                }) as Box<dyn FnMut(JsValue)>);

                span.add_event_listener_with_callback(
                    "input",
                    input_closure.as_ref().unchecked_ref(),
                )
                .unwrap();

                let self_input_closure = Closure::wrap(Box::new({
                    let self_span = self_span.clone();
                    let mutations = mutations.clone();
                    let sender = RefCell::new(sender.clone());
                    move |_| {
                        mutations.borrow_mut().push(FunctionMutation::UpdateSelf(
                            self_span.text_content().unwrap_or("".to_owned()),
                        ));
                        let _ = sender.borrow_mut().try_send(());
                    }
                })
                    as Box<dyn FnMut(JsValue)>);

                self_span
                    .add_event_listener_with_callback(
                        "input",
                        self_input_closure.as_ref().unchecked_ref(),
                    )
                    .unwrap();

                let focus_closure = Closure::wrap(Box::new({
                    let mutations = mutations.clone();
                    let span = span.clone();
                    let sender = RefCell::new(sender.clone());
                    move |_| {
                        mutations.borrow_mut().push(FunctionMutation::Focus);
                        focus_contenteditable(&span, true);
                        let _ = sender.borrow_mut().try_send(());
                    }
                }) as Box<dyn FnMut(JsValue)>);

                span.add_event_listener_with_callback(
                    "focus",
                    focus_closure.as_ref().unchecked_ref(),
                )
                .unwrap();

                let self_focus_closure = Closure::wrap(Box::new({
                    let mutations = mutations.clone();
                    let self_span = self_span.clone();
                    let sender = RefCell::new(sender.clone());
                    move |_| {
                        mutations.borrow_mut().push(FunctionMutation::FocusSelf);
                        focus_contenteditable(&self_span, true);
                        let _ = sender.borrow_mut().try_send(());
                    }
                })
                    as Box<dyn FnMut(JsValue)>);

                self_span
                    .add_event_listener_with_callback(
                        "focus",
                        self_focus_closure.as_ref().unchecked_ref(),
                    )
                    .unwrap();

                let keydown_closure = Closure::wrap(Box::new({
                    let mutations = mutations.clone();
                    let span = span.clone();
                    let container = container.clone();
                    let sender = RefCell::new(sender.clone());
                    move |e: JsValue| {
                        let e: KeyboardEvent = e.dyn_into().unwrap();
                        if (e.code() == "Backspace" || e.code() == "Delete")
                            && span.text_content().unwrap_or("".into()).len() == 0
                        {
                            mutations.borrow_mut().push(FunctionMutation::Remove);
                            container.remove();
                            e.stop_propagation();
                            let _ = sender.borrow_mut().try_send(());
                        } else if e.code() == "Backslash" {
                            e.prevent_default();
                            mutations.borrow_mut().push(FunctionMutation::ToggleErased);
                            e.stop_propagation();
                            let _ = sender.borrow_mut().try_send(());
                        }
                    }
                }) as Box<dyn FnMut(JsValue)>);

                span.add_event_listener_with_callback(
                    "keydown",
                    keydown_closure.as_ref().unchecked_ref(),
                )
                .unwrap();

                let self_keydown_closure = Closure::wrap(Box::new({
                    let mutations = mutations.clone();
                    let self_span = self_span.clone();
                    let container = container.clone();
                    let sender = RefCell::new(sender.clone());
                    move |e: JsValue| {
                        let e: KeyboardEvent = e.dyn_into().unwrap();
                        if (e.code() == "Backspace" || e.code() == "Delete")
                            && self_span.text_content().unwrap_or("".into()).len() == 0
                        {
                            mutations.borrow_mut().push(FunctionMutation::Remove);
                            container.remove();
                            e.stop_propagation();
                            let _ = sender.borrow_mut().try_send(());
                        } else if e.code() == "Backslash" {
                            e.prevent_default();
                            mutations.borrow_mut().push(FunctionMutation::ToggleErased);
                            e.stop_propagation();
                            let _ = sender.borrow_mut().try_send(());
                        }
                    }
                })
                    as Box<dyn FnMut(JsValue)>);

                self_span
                    .add_event_listener_with_callback(
                        "keydown",
                        self_keydown_closure.as_ref().unchecked_ref(),
                    )
                    .unwrap();

                UiSection::new(UiSectionVariance::Function {
                    container,
                    self_span,
                    self_focused: Rc::new(RefCell::new(false)),
                    span,
                    mutations,
                    closures: Rc::new(vec![
                        input_closure,
                        self_input_closure,
                        focus_closure,
                        self_focus_closure,
                        keydown_closure,
                        self_keydown_closure,
                    ]),
                })
            },
        },
        Term::Wrap(term, _) => Term::Wrap(Box::new(add_ui(*term, &sender)), {
            let mutations = Rc::new(RefCell::new(vec![]));

            let container = document.create_element("div").unwrap();
            container.class_list().add_1("wrap").unwrap();
            container.set_attribute("tabindex", "0").unwrap();

            let span = document.create_element("span").unwrap();
            span.class_list().add_1("wrap-inner").unwrap();

            let content = document.create_element("span").unwrap();
            content.class_list().add_1("wrap-content").unwrap();

            container.append_child(&span).unwrap();
            container.append_child(&content).unwrap();

            let focus_closure = Closure::wrap(Box::new({
                let mutations = mutations.clone();
                let container = container.clone();
                let sender = RefCell::new(sender.clone());
                move |_| {
                    mutations.borrow_mut().push(WrapMutation::Focus);
                    focus_element(&container, true);
                    let _ = sender.borrow_mut().try_send(());
                }
            }) as Box<dyn FnMut(JsValue)>);

            container
                .add_event_listener_with_callback("focus", focus_closure.as_ref().unchecked_ref())
                .unwrap();

            let keydown_closure = Closure::wrap(Box::new({
                let mutations = mutations.clone();
                let container = container.clone();
                let sender = RefCell::new(sender.clone());
                move |e: JsValue| {
                    let e: KeyboardEvent = e.dyn_into().unwrap();
                    if document.active_element().unwrap() == container {
                        if e.code() == "Backspace" || e.code() == "Delete" {
                            mutations.borrow_mut().push(WrapMutation::Remove);
                            container.remove();
                            e.stop_propagation();
                            let _ = sender.borrow_mut().try_send(());
                        }
                    }
                }
            }) as Box<dyn FnMut(JsValue)>);

            container
                .add_event_listener_with_callback(
                    "keydown",
                    keydown_closure.as_ref().unchecked_ref(),
                )
                .unwrap();

            UiSection::new(UiSectionVariance::Wrap {
                mutations,
                closures: Rc::new(vec![keydown_closure, focus_closure]),
                container,
                content,
            })
        }),

        Term::Hole(_) => Term::Hole(ui_section(Term::Hole(()), sender)),

        Term::Dynamic(cursor) => {
            let (_, term) = cursor.into_inner();

            let (annotation, term) = term.add_ui(sender);

            Term::Dynamic(Dynamic::new(annotation, term))
        }
    }
}

fn ui_section(term: Term, sender: &Sender<()>) -> UiSection {
    let document = web_sys::window().unwrap().document().unwrap();

    match term {
        Term::Hole(()) => {
            let mutations = Rc::new(RefCell::new(vec![]));

            let p = document.create_element("p").unwrap();

            p.class_list().add_1("hole").unwrap();

            configure_contenteditable(&p);

            let closure = Closure::wrap(Box::new({
                let p = p.clone();
                let mutations = mutations.clone();
                let sender = RefCell::new(sender.clone());
                move |_| {
                    let content = p.text_content().unwrap_or("".to_owned());
                    let letter = content.chars().next();

                    if let Some(c) = letter {
                        let mutation = match c {
                            'r' => Some(HoleMutation::Replace(add_ui(
                                Term::Reference("".into(), ()),
                                &sender.borrow().clone(),
                            ))),
                            'l' => Some(HoleMutation::Replace(add_ui(
                                Term::Lambda {
                                    erased: false,
                                    name: None,
                                    body: Box::new(Term::Hole(())),
                                    annotation: (),
                                },
                                &sender.borrow().clone(),
                            ))),
                            'L' => Some(HoleMutation::Replace(add_ui(
                                Term::Lambda {
                                    erased: true,
                                    name: None,
                                    body: Box::new(Term::Hole(())),
                                    annotation: (),
                                },
                                &sender.borrow().clone(),
                            ))),
                            'a' => Some(HoleMutation::Replace(add_ui(
                                Term::Application {
                                    erased: false,
                                    function: Box::new(Term::Hole(())),
                                    argument: Box::new(Term::Hole(())),
                                    annotation: (),
                                },
                                &sender.borrow().clone(),
                            ))),
                            'A' => Some(HoleMutation::Replace(add_ui(
                                Term::Application {
                                    erased: true,
                                    function: Box::new(Term::Hole(())),
                                    argument: Box::new(Term::Hole(())),
                                    annotation: (),
                                },
                                &sender.borrow().clone(),
                            ))),
                            'u' => Some(HoleMutation::Replace(add_ui(
                                Term::Universe(()),
                                &sender.borrow().clone(),
                            ))),
                            'w' => Some(HoleMutation::Replace(add_ui(
                                Term::Wrap(Box::new(Term::Hole(())), ()),
                                &sender.borrow().clone(),
                            ))),
                            'p' => Some(HoleMutation::Replace(add_ui(
                                Term::Put(Box::new(Term::Hole(())), ()),
                                &sender.borrow().clone(),
                            ))),
                            'd' => Some(HoleMutation::Replace(add_ui(
                                Term::Duplication {
                                    binder: None,
                                    expression: Box::new(Term::Hole(())),
                                    body: Box::new(Term::Hole(())),
                                    annotation: (),
                                },
                                &sender.borrow().clone(),
                            ))),
                            'f' => Some(HoleMutation::Replace(add_ui(
                                Term::Function {
                                    erased: false,
                                    name: None,
                                    self_name: None,
                                    annotation: (),
                                    argument_type: Box::new(Term::Hole(())),
                                    return_type: Box::new(Term::Hole(())),
                                },
                                &sender.borrow().clone(),
                            ))),
                            'F' => Some(HoleMutation::Replace(add_ui(
                                Term::Function {
                                    erased: true,
                                    name: None,
                                    self_name: None,
                                    annotation: (),
                                    argument_type: Box::new(Term::Hole(())),
                                    return_type: Box::new(Term::Hole(())),
                                },
                                &sender.borrow().clone(),
                            ))),
                            'D' => Some(HoleMutation::Replace(add_ui(
                                Term::Dynamic(Dynamic::new(
                                    (),
                                    Def::new(Term::Hole(()), Term::Hole(()), None),
                                )),
                                &sender.borrow().clone(),
                            ))),
                            _ => None,
                        };
                        if let Some(m) = mutation {
                            p.remove();
                            mutations.borrow_mut().push(m);
                            let _ = sender.borrow_mut().try_send(());
                        }
                    }
                }
            }) as Box<dyn FnMut(JsValue)>);

            p.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())
                .unwrap();

            let blur_closure = Closure::wrap(Box::new({
                let p = p.clone();
                move |_| {
                    p.set_text_content(Some(""));
                }
            }) as Box<dyn FnMut(JsValue)>);

            p.add_event_listener_with_callback("blur", blur_closure.as_ref().unchecked_ref())
                .unwrap();

            let focus_closure = Closure::wrap(Box::new({
                let mutations = mutations.clone();
                let sender = RefCell::new(sender.clone());
                let p = p.clone();
                move |_| {
                    mutations.borrow_mut().push(HoleMutation::Focus);
                    focus_contenteditable(&p, true);
                    let _ = sender.borrow_mut().try_send(());
                }
            }) as Box<dyn FnMut(JsValue)>);

            p.add_event_listener_with_callback("focus", focus_closure.as_ref().unchecked_ref())
                .unwrap();

            let keydown_closure = Closure::wrap(Box::new({
                let mutations = mutations.clone();
                let p = p.clone();
                let sender = RefCell::new(sender.clone());
                move |e: JsValue| {
                    let e: KeyboardEvent = e.dyn_into().unwrap();

                    let is_delete = e.code() == "Backspace" || e.code() == "Delete";

                    if !is_delete {
                        return;
                    }

                    let is_empty = p.text_content().unwrap_or("".into()).is_empty();
                    let is_top = p
                        .parent_element()
                        .unwrap()
                        .class_list()
                        .contains("scratchpad");

                    if is_empty {
                        if is_top {
                            return;
                        }

                        mutations.borrow_mut().push(HoleMutation::ToParent);
                        p.remove();
                        let _ = sender.borrow_mut().try_send(());
                    }

                    e.stop_propagation();
                }
            }) as Box<dyn FnMut(JsValue)>);

            p.add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref())
                .unwrap();

            UiSection::new(UiSectionVariance::Hole {
                mutations,
                p,
                closures: Rc::new(vec![closure, focus_closure, blur_closure, keydown_closure]),
            })
        }
        _ => todo!(),
    }
}

fn render_to(data: &Cursor<UiSection>, node: &Node) -> Result<(), JsValue> {
    match &data {
        Cursor::Lambda(cursor) => {
            let annotation = cursor.annotation();
            let node = annotation
                .render(node, &Cursor::Lambda(cursor.clone()))?
                .unwrap();

            render_to(&cursor.clone().body(), &node)?;
        }
        Cursor::Application(cursor) => {
            let annotation = cursor.annotation();
            let node = annotation
                .render(node, &Cursor::Application(cursor.clone()))?
                .unwrap();

            let function_node = node.child_nodes().get(0).unwrap();
            let argument_node = node.child_nodes().get(1).unwrap();
            render_to(&cursor.clone().function(), &function_node)?;
            render_to(&cursor.clone().argument(), &argument_node)?;
        }
        Cursor::Put(cursor) => {
            let annotation = cursor.annotation();
            let node = annotation
                .render(node, &Cursor::Put(cursor.clone()))?
                .unwrap();

            render_to(&cursor.clone().term(), &node)?;
        }
        Cursor::Reference(cursor) => {
            let annotation = cursor.annotation();
            annotation.render(node, &Cursor::Reference(cursor.clone()))?;
        }
        Cursor::Duplication(cursor) => {
            let annotation = cursor.annotation();

            let node = annotation
                .render(node, &Cursor::Duplication(cursor.clone()))?
                .unwrap();

            let expression_node = node.child_nodes().get(1).unwrap();
            let body_node = node.child_nodes().get(2).unwrap();
            render_to(&cursor.clone().expression(), &expression_node)?;
            render_to(&cursor.clone().body(), &body_node)?;
        }
        Cursor::Universe(cursor) => {
            let annotation = cursor.annotation();
            annotation.render(node, &Cursor::Universe(cursor.clone()))?;
        }
        Cursor::Function(cursor) => {
            let annotation = cursor.annotation();
            let node = annotation
                .render(node, &Cursor::Function(cursor.clone()))?
                .unwrap();

            let argument_type_node = node.child_nodes().get(2).unwrap();
            let return_type_node = node.child_nodes().get(3).unwrap();
            render_to(&cursor.clone().argument_type(), &argument_type_node)?;
            render_to(&cursor.clone().return_type(), &return_type_node)?;
        }
        Cursor::Wrap(cursor) => {
            let annotation = cursor.annotation();
            let node = annotation
                .render(node, &Cursor::Wrap(cursor.clone()))?
                .unwrap();

            render_to(&cursor.clone().term(), &node)?;
        }

        Cursor::Hole(cursor) => {
            let annotation = cursor.annotation();
            annotation.render(node, &Cursor::Hole(cursor.clone()))?;
        }

        Cursor::Dynamic(cursor) => {
            cursor.term.render_to(
                &cursor.up,
                match &cursor.annotation.variant {
                    UiSectionVariance::Dynamic(variance) => variance.as_ref(),
                    _ => panic!(),
                },
                node,
            )?;
        }
    }

    Ok(())
}

fn apply_mutations(
    data: Cursor<UiSection>,
    focused: &mut Option<Cursor<UiSection>>,
    sender: &Sender<()>,
) -> Result<Cursor<UiSection>, JsValue> {
    Ok(match data {
        Cursor::Lambda(c) => {
            let cursor = c.clone().body();
            let body: Term<_> = apply_mutations(cursor, focused, sender)?.into();

            let mutations: Vec<_> = match &c.annotation().variant {
                UiSectionVariance::Lambda { mutations, .. } => {
                    mutations.borrow_mut().drain(..).collect()
                }
                _ => panic!(),
            };

            let mut c = c.with_body(body);

            for mutation in &mutations {
                match mutation {
                    LambdaMutation::Update(name) => {
                        c = c.with_name(if name.is_empty() {
                            None
                        } else {
                            Some(name.clone())
                        });
                    }
                    LambdaMutation::ToggleErased => {
                        *c.erased_mut() = !c.erased();
                    }
                    _ => {}
                }
            }

            let mut c = Cursor::Lambda(c);

            for mutation in &mutations {
                match mutation {
                    LambdaMutation::Remove => {
                        let annotation = c.annotation().annotation.clone();
                        c = Cursor::Hole(match c {
                            Cursor::Lambda(cursor) => {
                                cursor.into_hole(ui_section(Term::Hole(()), sender))
                            }
                            _ => todo!(),
                        });
                        c.annotation_mut().annotation = annotation;
                        break;
                    }
                    _ => {}
                }
            }

            for mutation in mutations {
                match mutation {
                    LambdaMutation::Focus => {
                        *focused = Some(c.clone());
                    }
                    _ => {}
                }
            }

            c
        }
        Cursor::Application(mut cursor) => {
            let function: Term<_> =
                apply_mutations(cursor.clone().function(), focused, sender)?.into();
            let argument: Term<_> =
                apply_mutations(cursor.clone().argument(), focused, sender)?.into();

            cursor = cursor.with_function(function);
            cursor = cursor.with_argument(argument);

            let mutations: Vec<_> = match &cursor.annotation().variant {
                UiSectionVariance::Application { mutations, .. } => {
                    mutations.borrow_mut().drain(..).collect()
                }
                _ => panic!(),
            };

            for mutation in &mutations {
                match mutation {
                    ApplicationMutation::ToggleErased => {
                        *cursor.erased_mut() = !cursor.erased();
                    }
                    _ => {}
                }
            }

            let mut cursor = Cursor::Application(cursor);

            for mutation in &mutations {
                match mutation {
                    ApplicationMutation::Remove => {
                        let annotation = cursor.annotation().annotation.clone();
                        cursor = Cursor::Hole(match cursor {
                            Cursor::Application(cursor) => {
                                cursor.into_hole(ui_section(Term::Hole(()), sender))
                            }
                            _ => todo!(),
                        });
                        cursor.annotation_mut().annotation = annotation;
                        break;
                    }
                    _ => {}
                }
            }

            for mutation in mutations {
                match mutation {
                    ApplicationMutation::Focus => {
                        *focused = Some(cursor.clone());
                    }
                    _ => {}
                }
            }

            cursor
        }
        Cursor::Put(mut cursor) => {
            let term: Term<_> = apply_mutations(cursor.clone().term(), focused, sender)?.into();

            cursor = cursor.with_term(term);

            let mutations: Vec<_> = match &cursor.annotation().variant {
                UiSectionVariance::Put { mutations, .. } => {
                    mutations.borrow_mut().drain(..).collect()
                }
                _ => panic!(),
            };

            let mut cursor = Cursor::Put(cursor);

            for mutation in &mutations {
                match mutation {
                    PutMutation::Remove => {
                        let annotation = cursor.annotation().annotation.clone();
                        cursor = Cursor::Hole(match cursor {
                            Cursor::Put(cursor) => {
                                cursor.into_hole(ui_section(Term::Hole(()), sender))
                            }
                            _ => todo!(),
                        });
                        cursor.annotation_mut().annotation = annotation;
                        break;
                    }
                    _ => {}
                }
            }

            for mutation in mutations {
                match mutation {
                    PutMutation::Focus => {
                        *focused = Some(cursor.clone());
                    }
                    _ => {}
                }
            }

            cursor
        }
        Cursor::Reference(mut cursor) => {
            let mutations: Vec<_> = match &cursor.annotation().variant {
                UiSectionVariance::Reference { mutations, .. } => {
                    mutations.borrow_mut().drain(..).collect()
                }
                _ => panic!(),
            };

            for mutation in &mutations {
                match mutation {
                    ReferenceMutation::Update(name) => cursor = cursor.with_name(name.clone()),
                    _ => {}
                }
            }

            let mut cursor = Cursor::Reference(cursor);

            for mutation in &mutations {
                match mutation {
                    ReferenceMutation::Remove => {
                        let annotation = cursor.annotation().annotation.clone();
                        cursor = Cursor::Hole(match cursor {
                            Cursor::Reference(cursor) => {
                                cursor.into_hole(ui_section(Term::Hole(()), sender))
                            }
                            _ => todo!(),
                        });
                        cursor.annotation_mut().annotation = annotation;
                        break;
                    }
                    _ => {}
                }
            }

            for mutation in mutations {
                match mutation {
                    ReferenceMutation::Focus => {
                        *focused = Some(cursor.clone());
                    }
                    _ => {}
                }
            }

            cursor
        }
        Cursor::Duplication(c) => {
            let cursor = c.clone().body();
            let body: Term<_> = apply_mutations(cursor, focused, sender)?.into();

            let cursor = c.clone().expression();
            let expression: Term<_> = apply_mutations(cursor, focused, sender)?.into();

            let mutations: Vec<_> = match &c.annotation().variant {
                UiSectionVariance::Duplication { mutations, .. } => {
                    mutations.borrow_mut().drain(..).collect()
                }
                _ => panic!(),
            };

            let mut c = c.with_body(body);
            c = c.with_expression(expression);

            for mutation in &mutations {
                match mutation {
                    DuplicationMutation::Update(name) => {
                        c = c.with_binder(if name.is_empty() {
                            None
                        } else {
                            Some(name.clone())
                        });
                    }

                    _ => {}
                }
            }

            let mut c = Cursor::Duplication(c);

            for mutation in &mutations {
                match mutation {
                    DuplicationMutation::Remove => {
                        let annotation = c.annotation().annotation.clone();
                        c = Cursor::Hole(match c {
                            Cursor::Duplication(cursor) => {
                                cursor.into_hole(ui_section(Term::Hole(()), sender))
                            }
                            _ => todo!(),
                        });
                        c.annotation_mut().annotation = annotation;
                        break;
                    }
                    _ => {}
                }
            }

            for mutation in mutations {
                match mutation {
                    DuplicationMutation::Focus => {
                        *focused = Some(c.clone());
                    }
                    _ => {}
                }
            }

            c
        }
        Cursor::Universe(cursor) => {
            let mutations: Vec<_> = match &cursor.annotation().variant {
                UiSectionVariance::Universe { mutations, .. } => {
                    mutations.borrow_mut().drain(..).collect()
                }
                _ => panic!(),
            };

            let mut cursor = Cursor::Universe(cursor);

            for mutation in &mutations {
                match mutation {
                    UniverseMutation::Remove => {
                        let annotation = cursor.annotation().annotation.clone();
                        cursor = Cursor::Hole(match cursor {
                            Cursor::Universe(cursor) => {
                                cursor.into_hole(ui_section(Term::Hole(()), sender))
                            }
                            _ => todo!(),
                        });
                        cursor.annotation_mut().annotation = annotation;
                        break;
                    }
                    _ => {}
                }
            }

            for mutation in mutations {
                match mutation {
                    UniverseMutation::Focus => {
                        *focused = Some(cursor.clone());
                    }
                    _ => {}
                }
            }

            cursor
        }
        Cursor::Function(c) => {
            let cursor = c.clone().argument_type();
            let argument_type: Term<_> = apply_mutations(cursor, focused, sender)?.into();

            let cursor = c.clone().return_type();
            let return_type: Term<_> = apply_mutations(cursor, focused, sender)?.into();

            let (mutations, self_focused): (Vec<_>, _) = match &c.annotation().variant {
                UiSectionVariance::Function {
                    mutations,
                    self_focused,
                    ..
                } => (
                    mutations.borrow_mut().drain(..).collect(),
                    self_focused.clone(),
                ),
                _ => panic!(),
            };

            let mut c = c.with_argument_type(argument_type);
            c = c.with_return_type(return_type);

            for mutation in &mutations {
                match mutation {
                    FunctionMutation::Update(name) => {
                        c = c.with_name(if name.is_empty() {
                            None
                        } else {
                            Some(name.clone())
                        });
                    }
                    FunctionMutation::UpdateSelf(name) => {
                        c = c.with_self_name(if name.is_empty() {
                            None
                        } else {
                            Some(name.clone())
                        });
                    }
                    FunctionMutation::ToggleErased => {
                        *c.erased_mut() = !c.erased();
                    }
                    _ => {}
                }
            }

            let mut c = Cursor::Function(c);

            for mutation in &mutations {
                match mutation {
                    FunctionMutation::Remove => {
                        let annotation = c.annotation().annotation.clone();
                        c = Cursor::Hole(match c {
                            Cursor::Function(cursor) => {
                                cursor.into_hole(ui_section(Term::Hole(()), sender))
                            }
                            _ => todo!(),
                        });
                        c.annotation_mut().annotation = annotation;
                        break;
                    }
                    _ => {}
                }
            }

            for mutation in mutations {
                match mutation {
                    FunctionMutation::Focus => {
                        *self_focused.borrow_mut() = false;
                        *focused = Some(c.clone());
                    }
                    FunctionMutation::FocusSelf => {
                        *self_focused.borrow_mut() = true;
                        *focused = Some(c.clone());
                    }
                    _ => {}
                }
            }

            c
        }
        Cursor::Wrap(mut cursor) => {
            let term: Term<_> = apply_mutations(cursor.clone().term(), focused, sender)?.into();

            cursor = cursor.with_term(term);

            let mutations: Vec<_> = match &cursor.annotation().variant {
                UiSectionVariance::Wrap { mutations, .. } => {
                    mutations.borrow_mut().drain(..).collect()
                }
                _ => panic!(),
            };

            let mut cursor = Cursor::Wrap(cursor);

            for mutation in &mutations {
                match mutation {
                    WrapMutation::Remove => {
                        let annotation = cursor.annotation().annotation.clone();
                        cursor = Cursor::Hole(match cursor {
                            Cursor::Wrap(cursor) => {
                                cursor.into_hole(ui_section(Term::Hole(()), sender))
                            }
                            _ => todo!(),
                        });
                        cursor.annotation_mut().annotation = annotation;
                        break;
                    }
                    _ => {}
                }
            }

            for mutation in mutations {
                match mutation {
                    WrapMutation::Focus => {
                        *focused = Some(cursor.clone());
                    }
                    _ => {}
                }
            }

            cursor
        }

        Cursor::Hole(cursor) => {
            let mutations: Vec<_> = match &cursor.annotation().variant {
                UiSectionVariance::Hole { mutations, .. } => {
                    mutations.borrow_mut().drain(..).collect()
                }
                _ => panic!(),
            };

            let mut cursor = Cursor::Hole(cursor.clone());

            for mutation in &mutations {
                match mutation {
                    HoleMutation::Replace(term) => {
                        let term = term.clone();
                        let annotation = cursor.annotation().annotation.clone();
                        cursor = Cursor::from_term_and_path(term, cursor.path().clone());
                        cursor.annotation_mut().annotation = annotation;
                    }
                    HoleMutation::ToParent => {
                        cursor = cursor.ascend();
                    }
                    _ => {}
                }
            }

            for mutation in mutations {
                match mutation {
                    HoleMutation::Focus => {
                        *focused = Some(cursor.clone());
                    }
                    _ => {}
                }
            }

            cursor
        }

        Cursor::Dynamic(cursor) => {
            let term = cursor.term;

            term.apply_mutations(
                cursor.up,
                match cursor.annotation.variant {
                    UiSectionVariance::Dynamic(variance) => variance,
                    _ => panic!(),
                },
                focused,
                sender,
            )?
        }
    })
}

impl Scratchpad {
    pub fn new(term: Term, target_node: Node) -> Self {
        let (mut sender, receiver) = channel(0);
        let data = Rc::new(RefCell::new(add_ui(term, &sender).into()));

        let has_focus = Rc::new(RefCell::new(false));

        let focus_event_handler = Closure::wrap(Box::new({
            let has_focus = has_focus.clone();
            move |e: JsValue| {
                let e: FocusEvent = e.dyn_into().unwrap();

                match e.type_().as_str() {
                    "focusin" => {
                        *has_focus.borrow_mut() = true;
                    }
                    "focusout" => {
                        if e.related_target().is_some() {
                            *has_focus.borrow_mut() = false;
                        }
                    }
                    _ => panic!(),
                }
            }
        }) as Box<dyn FnMut(JsValue)>);

        target_node
            .add_event_listener_with_callback(
                "focusin",
                focus_event_handler.as_ref().unchecked_ref(),
            )
            .unwrap();
        target_node
            .add_event_listener_with_callback(
                "focusout",
                focus_event_handler.as_ref().unchecked_ref(),
            )
            .unwrap();

        let scratchpad = Scratchpad {
            data: data.clone(),
            needs_update: receiver,
            target_node,
            has_focus,
            sender: sender.clone(),
            on_change: OnChangeWrapper::new(),
            focus_event_handler,
            clipboard_event_handler: Closure::wrap(Box::new(move |e: JsValue| {
                let copy = |f: Box<dyn Fn(String)>| {
                    let data = &mut *data.borrow_mut();

                    let term: Term<()> = Term::<UiSection>::from(data.clone()).clear_annotation();

                    let waker = noop_waker();
                    let mut context = Context::from_waker(&waker);

                    let mut fut = Box::pin(TermData::from(term.clear_annotation()).encode());

                    let data = loop {
                        match fut.as_mut().poll(&mut context) {
                            std::task::Poll::Ready(data) => break data,
                            std::task::Poll::Pending => {}
                        }
                    };

                    if let Ok(data) = data {
                        f(data);
                    }
                };

                if let Some(e) = e.dyn_ref::<KeyboardEvent>() {
                    if e.ctrl_key() {
                        if e.key() == "c" {
                            copy(Box::new(|data| {
                                spawn_local(async move {
                                    let navigator = web_sys::window().unwrap().navigator();
                                    let clipboard = navigator.clipboard().unwrap();
                                    JsFuture::from(clipboard.write_text(&data)).await.unwrap();
                                });
                            }));
                            e.prevent_default();
                            e.stop_propagation();
                        } else if e.key() == "x" {
                            copy(Box::new(|data| {
                                spawn_local(async move {
                                    let navigator = web_sys::window().unwrap().navigator();
                                    let clipboard = navigator.clipboard().unwrap();
                                    JsFuture::from(clipboard.write_text(&data)).await.unwrap();
                                });
                            }));
                            data.borrow().annotation().trigger_remove(&sender);
                            e.prevent_default();
                            e.stop_propagation();
                        }
                    }

                    return;
                }
                let e: ClipboardEvent = e.dyn_into().unwrap();
                let c_data = e.clipboard_data().unwrap();

                match e.type_().as_str() {
                    "cut" => {
                        copy(Box::new(|data| {
                            c_data.set_data("text/plain", &data).unwrap();
                        }));
                        data.borrow().annotation().trigger_remove(&sender);
                        e.prevent_default();
                    }
                    "copy" => {
                        copy(Box::new(|data| {
                            c_data.set_data("text/plain", &data).unwrap();
                        }));
                        e.prevent_default();
                    }
                    "paste" => {
                        let data = &mut *data.borrow_mut();

                        if let Cursor::Hole(cursor) = data {
                            e.prevent_default();
                            if let Ok(data) = c_data.get_data("text/plain") {
                                let waker = noop_waker();
                                let mut context = Context::from_waker(&waker);

                                let mut fut = Box::pin(TermData::decode(data));

                                let data = loop {
                                    match fut.as_mut().poll(&mut context) {
                                        std::task::Poll::Ready(data) => break data,
                                        std::task::Poll::Pending => {}
                                    }
                                };

                                if let Ok(Some(data)) = data {
                                    match &mut cursor.annotation_mut().variant {
                                        UiSectionVariance::Hole { p, mutations, .. } => {
                                            p.remove();
                                            mutations.borrow_mut().push(HoleMutation::Replace(
                                                add_ui(data.into(), &sender),
                                            ));
                                        }
                                        _ => panic!(),
                                    }
                                    let _ = sender.try_send(());
                                }
                            }
                        }
                    }
                    _ => panic!(),
                }
            }) as Box<dyn FnMut(JsValue)>),
        };

        scratchpad.add_copy_listener();

        scratchpad
    }

    pub fn annotate(&self, data: Term) {
        self.data.borrow_mut().annotate(data);
    }

    pub fn force_update(&mut self, data: Term) {
        *self.data.borrow_mut() = add_ui(data, &self.sender).into();
        for i in 0..self.target_node.child_nodes().length() {
            if let Some(el) = self
                .target_node
                .child_nodes()
                .get(i)
                .unwrap()
                .dyn_ref::<Element>()
            {
                el.remove()
            }
        }
        let _ = self.sender.try_send(());
    }

    pub fn cursor(&self) -> Cursor<UiSection> {
        self.data.borrow().clone()
    }

    pub fn data(&self) -> Rc<RefCell<Cursor<UiSection>>> {
        self.data.clone()
    }

    pub async fn needs_update(&mut self) {
        self.needs_update.next().await.unwrap()
    }

    fn root_el(&self) -> Element {
        let annotation = self.data.borrow();
        let annotation = annotation.annotation();

        match &annotation.variant {
            UiSectionVariance::Lambda { span, .. } => span,
            UiSectionVariance::Application { container, .. } => container,
            UiSectionVariance::Reference { p, .. } => p,
            UiSectionVariance::Hole { p, .. } => p,
            UiSectionVariance::Universe { p, .. } => p,
            UiSectionVariance::Wrap { container, .. } => container,
            UiSectionVariance::Put { container, .. } => container,
            UiSectionVariance::Duplication { span, .. } => span,
            UiSectionVariance::Function {
                self_span,
                span,
                self_focused,
                ..
            } => {
                if *self_focused.borrow() {
                    self_span
                } else {
                    span
                }
            }
            UiSectionVariance::Dynamic(variance) => variance.focused_el(),
        }
        .clone()
    }

    fn add_copy_listener(&self) {
        let el = self.root_el();
        el.add_event_listener_with_callback(
            "cut",
            self.clipboard_event_handler.as_ref().unchecked_ref(),
        )
        .unwrap();
        el.add_event_listener_with_callback(
            "copy",
            self.clipboard_event_handler.as_ref().unchecked_ref(),
        )
        .unwrap();
        el.add_event_listener_with_callback(
            "paste",
            self.clipboard_event_handler.as_ref().unchecked_ref(),
        )
        .unwrap();
        el.add_event_listener_with_callback(
            "keydown",
            self.clipboard_event_handler.as_ref().unchecked_ref(),
        )
        .unwrap();
    }

    pub fn apply_mutations(mut self) -> Result<Self, JsValue> {
        let perf = web_sys::window().unwrap().performance().unwrap();
        let time = perf.now();

        let mut focused = None;
        let mut data = self.data.borrow().clone();

        let el = self.root_el();
        el.remove_event_listener_with_callback(
            "cut",
            self.clipboard_event_handler.as_ref().unchecked_ref(),
        )?;
        el.remove_event_listener_with_callback(
            "copy",
            self.clipboard_event_handler.as_ref().unchecked_ref(),
        )?;
        el.remove_event_listener_with_callback(
            "paste",
            self.clipboard_event_handler.as_ref().unchecked_ref(),
        )?;
        el.remove_event_listener_with_callback(
            "keydown",
            self.clipboard_event_handler.as_ref().unchecked_ref(),
        )?;

        data = apply_mutations(data, &mut focused, &self.sender)?;

        {
            let mut data = data.clone();
            while !data.is_top() {
                data = data.ascend();
            }
            apply_mutations(data, &mut focused, &self.sender)?;
        }

        if let Some(focused) = focused {
            data = focused;
        }

        self.add_copy_listener();

        *self.data.borrow_mut() = data;

        self.on_change.call(&*self.data.borrow());

        console_log!("update took {:.1}ms", perf.now() - time);

        Ok(self)
    }

    pub fn on_change(&mut self, closure: Box<dyn FnMut(&Cursor<UiSection>)>) {
        self.on_change.to_call.push(closure);
    }

    pub fn render(&self) -> Result<(), JsValue> {
        let perf = web_sys::window().unwrap().performance().unwrap();
        let time = perf.now();

        let mut data = self.data.borrow().clone();

        while !data.is_top() {
            data = data.ascend();
        }

        render_to(&data, &self.target_node)?;

        spawn_local({
            let data = self.data.clone();
            let has_focus = *self.has_focus.borrow();
            async move {
                if has_focus {
                    data.borrow().annotation().focus();
                }
            }
        });

        console_log!("render took {:.1}ms", perf.now() - time);

        Ok(())
    }

    pub fn focus(&self) {
        let data = self.data.clone();
        spawn_local(async move {
            data.borrow().annotation().focus();
        });
    }
}
