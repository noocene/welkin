use std::{cell::RefCell, rc::Rc};

use super::mutations::*;
use futures::channel::mpsc::Sender;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{HtmlElement, KeyboardEvent};

use crate::edit::{
    configure_contenteditable,
    dynamic::Def,
    focus_contenteditable, focus_element,
    zipper::{dynamic::Dynamic, Term},
    UiSectionVariance,
};

use super::UiSection;

pub fn add_ui<T>(term: Term<T>, sender: &Sender<()>) -> Term<UiSection> {
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

pub fn ui_section(term: Term, sender: &Sender<()>) -> UiSection {
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
