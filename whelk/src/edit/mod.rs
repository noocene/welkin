use std::{cell::RefCell, convert::Infallible, rc::Rc};

use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    task::{noop_waker, Context},
    Future, StreamExt,
};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{ClipboardEvent, Element, FocusEvent, HtmlElement, KeyboardEvent, Node};
use zipper::{Cursor, Term};

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
}
pub use mutations::*;

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
}

#[derive(Clone, Debug)]
pub struct UiSection {
    variant: UiSectionVariance,
}

impl UiSection {
    pub fn trigger_remove(&self, sender: &Sender<()>) {
        match &self.variant {
            UiSectionVariance::Lambda {
                p,
                span,
                container,
                closures,
                mutations,
            } => {
                container.dyn_ref::<Element>().unwrap().remove();
                mutations.borrow_mut().push(LambdaMutation::Remove)
            }
            UiSectionVariance::Application {
                container,
                closures,
                mutations,
            } => {
                container.remove();
                mutations.borrow_mut().push(ApplicationMutation::Remove)
            }
            UiSectionVariance::Reference {
                p,
                mutations,
                closures,
            } => {
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

#[derive(Debug)]
pub struct Scratchpad {
    data: Rc<RefCell<Cursor<UiSection>>>,
    needs_update: Receiver<()>,
    sender: Sender<()>,
    clipboard_event_handler: Closure<dyn FnMut(JsValue)>,
}

fn add_ui(term: Term, sender: &Sender<()>) -> Term<UiSection> {
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
                        e.stop_propagation();
                        if (e.code() == "Backspace" || e.code() == "Delete")
                            && span.text_content().unwrap_or("".into()).len() == 0
                        {
                            mutations.borrow_mut().push(LambdaMutation::Remove);
                            container.remove();
                            let _ = sender.borrow_mut().try_send(());
                        } else if e.code() == "Backslash" {
                            e.prevent_default();
                            mutations.borrow_mut().push(LambdaMutation::ToggleErased);
                            let _ = sender.borrow_mut().try_send(());
                        }
                    }
                }) as Box<dyn FnMut(JsValue)>);

                span.add_event_listener_with_callback(
                    "keydown",
                    keydown_closure.as_ref().unchecked_ref(),
                )
                .unwrap();

                UiSection {
                    variant: UiSectionVariance::Lambda {
                        p,
                        span,
                        mutations,
                        container: container.into(),
                        closures: Rc::new(vec![closure, focus_closure, keydown_closure]),
                    },
                }
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

                container.append_child(&spacer).unwrap();
                container.append_child(&function).unwrap();
                container.append_child(&argument).unwrap();

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
                            e.stop_propagation();
                            if e.code() == "Backspace" || e.code() == "Delete" {
                                mutations.borrow_mut().push(ApplicationMutation::Remove);
                                container.remove();
                                let _ = sender.borrow_mut().try_send(());
                            } else if e.code() == "Backslash" {
                                e.prevent_default();
                                mutations
                                    .borrow_mut()
                                    .push(ApplicationMutation::ToggleErased);
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

                UiSection {
                    variant: UiSectionVariance::Application {
                        container,
                        closures: Rc::new(vec![focus_closure, keydown_closure]),
                        mutations,
                    },
                }
            },
        },
        Term::Put(term, ()) => Term::Put(Box::new(add_ui(*term, &sender)), {
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
                        e.stop_propagation();
                        if e.code() == "Backspace" || e.code() == "Delete" {
                            mutations.borrow_mut().push(PutMutation::Remove);
                            container.remove();
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

            UiSection {
                variant: UiSectionVariance::Put {
                    mutations,
                    closures: Rc::new(vec![keydown_closure, focus_closure]),
                    container,
                    content,
                },
            }
        }),
        Term::Duplication { .. } => todo!(),
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
                let mutations = mutations.clone();
                let p = p.clone();
                let sender = RefCell::new(sender.clone());
                move |e: JsValue| {
                    let e: KeyboardEvent = e.dyn_into().unwrap();
                    if document.active_element().unwrap() == p {
                        e.stop_propagation();
                        if (e.code() == "Backspace" || e.code() == "Delete")
                            && p.text_content().unwrap_or("".into()).is_empty()
                        {
                            p.dyn_ref::<HtmlElement>().unwrap().blur().unwrap();
                        } else if e.code() == "Escape" {
                            p.dyn_ref::<HtmlElement>().unwrap().blur().unwrap();
                        }
                    }
                }
            }) as Box<dyn FnMut(JsValue)>);

            p.add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref())
                .unwrap();

            UiSection {
                variant: UiSectionVariance::Reference {
                    p,
                    mutations,
                    closures: Rc::new(vec![closure, focus_closure, blur_closure, keydown_closure]),
                },
            }
        }),

        Term::Universe(()) => Term::Universe({
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
                        e.stop_propagation();
                        if e.code() == "Backspace" || e.code() == "Delete" {
                            mutations.borrow_mut().push(UniverseMutation::Remove);
                            p.remove();
                            let _ = sender.borrow_mut().try_send(());
                        }
                    }
                }
            }) as Box<dyn FnMut(JsValue)>);

            p.add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref())
                .unwrap();

            UiSection {
                variant: UiSectionVariance::Universe {
                    mutations,
                    closures: Rc::new(vec![focus_closure, keydown_closure]),
                    p,
                },
            }
        }),
        Term::Function { .. } => todo!(),
        Term::Wrap(term, ()) => Term::Wrap(Box::new(add_ui(*term, &sender)), {
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
                        e.stop_propagation();
                        if e.code() == "Backspace" || e.code() == "Delete" {
                            mutations.borrow_mut().push(WrapMutation::Remove);
                            container.remove();
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

            UiSection {
                variant: UiSectionVariance::Wrap {
                    mutations,
                    closures: Rc::new(vec![keydown_closure, focus_closure]),
                    container,
                    content,
                },
            }
        }),

        Term::Hole(()) => Term::Hole(ui_section(Term::Hole(()), sender)),
    }
}

fn ui_section(term: Term, sender: &Sender<()>) -> UiSection {
    let document = web_sys::window().unwrap().document().unwrap();

    match term {
        Term::Lambda { .. } => todo!(),
        Term::Application { .. } => todo!(),
        Term::Put(_, _) => todo!(),
        Term::Duplication { .. } => todo!(),
        Term::Reference(_, _) => todo!(),
        Term::Universe(_) => todo!(),
        Term::Function { .. } => todo!(),
        Term::Wrap(_, _) => todo!(),
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
                    e.stop_propagation();
                    if (e.code() == "Backspace" || e.code() == "Delete")
                        && p.text_content().unwrap_or("".into()).is_empty()
                    {
                        mutations.borrow_mut().push(HoleMutation::ToParent);
                        p.remove();
                        let _ = sender.borrow_mut().try_send(());
                    }
                }
            }) as Box<dyn FnMut(JsValue)>);

            p.add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref())
                .unwrap();

            UiSection {
                variant: UiSectionVariance::Hole {
                    mutations,
                    p,
                    closures: Rc::new(vec![closure, focus_closure, blur_closure, keydown_closure]),
                },
            }
        }
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

            let function_node = node.child_nodes().get(1).unwrap();
            let argument_node = node.child_nodes().get(2).unwrap();
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
        Cursor::Duplication(_) => todo!(),
        Cursor::Universe(cursor) => {
            let annotation = cursor.annotation();
            annotation.render(node, &Cursor::Universe(cursor.clone()))?;
        }
        Cursor::Function(_) => todo!(),
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
                        c = Cursor::Hole(match c {
                            Cursor::Lambda(cursor) => {
                                cursor.into_hole(ui_section(Term::Hole(()), sender))
                            }
                            _ => todo!(),
                        });
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
                        cursor = Cursor::Hole(match cursor {
                            Cursor::Application(cursor) => {
                                cursor.into_hole(ui_section(Term::Hole(()), sender))
                            }
                            _ => todo!(),
                        });
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
                        cursor = Cursor::Hole(match cursor {
                            Cursor::Put(cursor) => {
                                cursor.into_hole(ui_section(Term::Hole(()), sender))
                            }
                            _ => todo!(),
                        });
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
                        cursor = Cursor::Hole(match cursor {
                            Cursor::Reference(cursor) => {
                                cursor.into_hole(ui_section(Term::Hole(()), sender))
                            }
                            _ => todo!(),
                        });
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
        Cursor::Duplication(_) => todo!(),
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
                        cursor = Cursor::Hole(match cursor {
                            Cursor::Universe(cursor) => {
                                cursor.into_hole(ui_section(Term::Hole(()), sender))
                            }
                            _ => todo!(),
                        });
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
        Cursor::Function(_) => todo!(),
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
                        cursor = Cursor::Hole(match cursor {
                            Cursor::Wrap(cursor) => {
                                cursor.into_hole(ui_section(Term::Hole(()), sender))
                            }
                            _ => todo!(),
                        });
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
                        cursor = Cursor::from_term_and_path(term, cursor.path().clone());
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
    })
}

impl Scratchpad {
    pub fn new(term: Term) -> Self {
        let (mut sender, receiver) = channel(0);
        let data = Rc::new(RefCell::new(add_ui(term, &sender).into()));

        let scratchpad = Scratchpad {
            data: data.clone(),
            needs_update: receiver,
            sender: sender.clone(),
            clipboard_event_handler: Closure::wrap(Box::new(move |e: JsValue| {
                let copy = |f: Box<dyn Fn(String)>| {
                    let data = &mut *data.borrow_mut();

                    let term: Term<()> = Term::<UiSection>::from(data.clone())
                        .try_map_annotation(|_| Ok::<_, Infallible>(()))
                        .unwrap();

                    let waker = noop_waker();
                    let mut context = Context::from_waker(&waker);

                    let mut fut = Box::pin(term.encode());

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

                                let mut fut = Box::pin(Term::<()>::decode(data));

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
                                            mutations
                                                .borrow_mut()
                                                .push(HoleMutation::Replace(add_ui(data, &sender)));
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

    pub async fn needs_update(&mut self) {
        self.needs_update.next().await.unwrap()
    }

    fn add_copy_listener(&self) {
        let annotation = self.data.borrow();
        let annotation = annotation.annotation();

        let el = match &annotation.variant {
            UiSectionVariance::Lambda { span, .. } => span,
            UiSectionVariance::Application { container, .. } => container,
            UiSectionVariance::Reference { p, .. } => p,
            UiSectionVariance::Hole { p, .. } => p,
            UiSectionVariance::Universe { p, .. } => p,
            UiSectionVariance::Wrap { container, .. } => container,
            UiSectionVariance::Put { container, .. } => container,
        };
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

    pub fn apply_mutations(self) -> Result<Self, JsValue> {
        let perf = web_sys::window().unwrap().performance().unwrap();
        let time = perf.now();

        let mut focused = None;
        let mut data = self.data.borrow().clone();

        let annotation = data.annotation();

        let el = match &annotation.variant {
            UiSectionVariance::Lambda { span, .. } => span,
            UiSectionVariance::Application { container, .. } => container,
            UiSectionVariance::Reference { p, .. } => p,
            UiSectionVariance::Hole { p, .. } => p,
            UiSectionVariance::Universe { p, .. } => p,
            UiSectionVariance::Wrap { container, .. } => container,
            UiSectionVariance::Put { container, .. } => container,
        };
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

        console_log!("update took {:.1}ms", perf.now() - time);

        Ok(Scratchpad {
            data: self.data,
            needs_update: self.needs_update,
            sender: self.sender.clone(),
            clipboard_event_handler: self.clipboard_event_handler,
        })
    }

    pub fn render_to(&self, node: &Node) -> Result<(), JsValue> {
        let perf = web_sys::window().unwrap().performance().unwrap();
        let time = perf.now();

        let mut data = self.data.borrow().clone();

        while !data.is_top() {
            data = data.ascend();
        }

        render_to(&data, node)?;

        spawn_local({
            let data = self.data.clone();
            async move {
                data.borrow().annotation().focus();
            }
        });

        console_log!("render took {:.1}ms", perf.now() - time);

        Ok(())
    }
}
