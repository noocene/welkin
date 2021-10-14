use std::{
    cell::RefCell,
    fmt::{self, Debug},
    rc::Rc,
};

use futures::channel::mpsc::Sender;
use mincodec::MinCodec;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{Element, KeyboardEvent};

use crate::{
    edit::{
        add_ui, apply_mutations, configure_contenteditable, focus_contenteditable, render_to,
        ui_section,
        zipper::{encode, BranchWrapper, Cursor, DynamicCursor, HoleCursor, Path, Term},
        DynamicVariance, UiSection, UiSectionVariance,
    },
    zipper::dynamic::DynamicTerm,
};

#[derive(Debug, Clone)]
pub struct Def<T> {
    expression: Term<T>,
    body: Term<T>,
    binder: Option<String>,
}

#[derive(Clone, Debug)]
pub enum DefMutation {
    Focus,
    Remove,
    Update(String),
}

#[derive(Clone, Debug)]
pub struct DefVariance {
    container: Element,
    span: Element,
    closures: Rc<Vec<Closure<dyn FnMut(JsValue)>>>,
    mutations: Rc<RefCell<Vec<DefMutation>>>,
}

impl DynamicVariance for DefVariance {
    fn box_clone(&self) -> Box<dyn DynamicVariance> {
        Box::new(self.clone())
    }

    fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <DefVariance as Debug>::fmt(self, f)
    }

    fn focus(&self) {
        focus_contenteditable(&self.span, false)
    }

    fn remove(&self) {
        self.mutations.borrow_mut().push(DefMutation::Remove);
        self.container.remove();
    }

    fn focused_el(&self) -> &Element {
        &self.span
    }
}

#[derive(MinCodec)]
#[bounds(Term<T>)]
pub struct DefData<T> {
    pub expression: Term<T>,
    pub body: Term<T>,
    pub binder: Option<String>,
}

impl DynamicTerm<()> for Def<()> {
    fn debug(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }

    fn box_clone(&self) -> Box<dyn DynamicTerm<()>> {
        Box::new(self.clone())
    }

    fn add_ui(
        self: Box<Self>,
        sender: &Sender<()>,
    ) -> (UiSection, Box<dyn DynamicTerm<UiSection>>) {
        let document = web_sys::window().unwrap().document().unwrap();

        let mutations = Rc::new(RefCell::new(vec![]));
        let sender = RefCell::new(sender.clone());

        let container = document.create_element("div").unwrap();
        container.class_list().add_1("def").unwrap();

        let span = document.create_element("span").unwrap();
        span.class_list().add_1("def-inner").unwrap();

        span.set_attribute("contenteditable", "true").unwrap();
        span.set_attribute("tabindex", "0").unwrap();
        configure_contenteditable(&span);

        let closure = Closure::wrap(Box::new({
            let span = span.clone();
            let mutations = mutations.clone();
            let sender = sender.clone();
            move |_| {
                mutations.borrow_mut().push(DefMutation::Update(
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
            let sender = sender.clone();
            move |_| {
                mutations.borrow_mut().push(DefMutation::Focus);
                focus_contenteditable(&span, true);
                let _ = sender.borrow_mut().try_send(());
            }
        }) as Box<dyn FnMut(JsValue)>);

        span.add_event_listener_with_callback("focus", focus_closure.as_ref().unchecked_ref())
            .unwrap();

        let keydown_closure = Closure::wrap(Box::new({
            let mutations = mutations.clone();
            let span = span.clone();
            let container = container.clone();
            let sender = sender.clone();
            move |e: JsValue| {
                let e: KeyboardEvent = e.dyn_into().unwrap();
                e.stop_propagation();
                if (e.code() == "Backspace" || e.code() == "Delete")
                    && span.text_content().unwrap_or("".into()).len() == 0
                {
                    mutations.borrow_mut().push(DefMutation::Remove);
                    container.remove();
                    let _ = sender.borrow_mut().try_send(());
                }
            }
        }) as Box<dyn FnMut(JsValue)>);

        span.add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref())
            .unwrap();

        let expression = document.create_element("span").unwrap();
        expression.class_list().add_1("def-expression").unwrap();

        let body = document.create_element("span").unwrap();
        body.class_list().add_1("def-body").unwrap();

        container.append_child(&span).unwrap();
        container.append_child(&expression).unwrap();
        container.append_child(&body).unwrap();

        let section = UiSection {
            variant: UiSectionVariance::Dynamic(Box::new(DefVariance {
                container,
                span,
                closures: Rc::new(vec![keydown_closure, focus_closure, closure]),
                mutations,
            })),
        };

        let sender = &sender.borrow().clone();

        (
            section,
            Box::new(Def {
                expression: add_ui(self.expression, sender),
                body: add_ui(self.body, sender),
                binder: self.binder,
            }),
        )
    }

    fn apply_mutations(
        self: Box<Self>,
        _: Path<UiSection>,
        _: Box<dyn DynamicVariance>,
        _: &mut Option<Cursor<UiSection>>,
        _: &Sender<()>,
    ) -> Result<Cursor<UiSection>, JsValue> {
        unimplemented!()
    }

    fn render_to(
        &self,
        _: &Path<UiSection>,
        _: &dyn DynamicVariance,
        _: &web_sys::Node,
    ) -> Result<(), JsValue> {
        unimplemented!()
    }

    fn clear_annotation(self: Box<Self>) -> Box<dyn DynamicTerm<()>> {
        self
    }

    fn index(&self) -> u8 {
        'D' as u8
    }

    fn encode(self: Box<Self>) -> Vec<u8> {
        if let Ok(data) = encode(DefData {
            expression: self.expression,
            body: self.body,
            binder: self.binder,
        }) {
            data
        } else {
            panic!()
        }
    }
}

#[derive(Debug, Clone)]
pub enum DefBranch<T> {
    Expression {
        body: Term<T>,
        binder: Option<String>,
    },
    Body {
        expression: Term<T>,
        binder: Option<String>,
    },
}

impl BranchWrapper<UiSection> for DefBranch<UiSection> {
    fn reconstruct(self: Box<Self>, term: Term<UiSection>) -> Box<dyn DynamicTerm<UiSection>> {
        Box::new(match *self {
            DefBranch::Expression { body, binder } => Def {
                body,
                expression: term,
                binder,
            },
            DefBranch::Body { expression, binder } => Def {
                body: term,
                expression,
                binder,
            },
        })
    }

    fn box_clone(&self) -> Box<dyn BranchWrapper<UiSection>> {
        Box::new(<Self as Clone>::clone(self))
    }

    fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl DynamicTerm<UiSection> for Def<UiSection> {
    fn debug(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }

    fn box_clone(&self) -> Box<dyn DynamicTerm<UiSection>> {
        Box::new(self.clone())
    }

    fn add_ui(self: Box<Self>, _: &Sender<()>) -> (UiSection, Box<dyn DynamicTerm<UiSection>>) {
        todo!()
    }

    fn apply_mutations(
        mut self: Box<Self>,
        up: Path<UiSection>,
        annotation: Box<dyn DynamicVariance>,
        focused: &mut Option<Cursor<UiSection>>,
        sender: &Sender<()>,
    ) -> Result<Cursor<UiSection>, JsValue> {
        let c_expression = self.expression;
        let body = self.body;

        let annotation_data: &DefVariance = annotation.downcast_ref().unwrap();
        let mutations = annotation_data
            .mutations
            .borrow_mut()
            .drain(..)
            .collect::<Vec<_>>();

        let expression = apply_mutations(
            Cursor::from_term_and_path(
                c_expression.clone(),
                Path::Dynamic {
                    up: Box::new(up.clone()),
                    branch: Box::new(DefBranch::Expression {
                        body: body.clone(),
                        binder: self.binder.clone(),
                    }),
                    annotation: UiSection {
                        variant: UiSectionVariance::Dynamic(annotation.clone()),
                    },
                },
            ),
            focused,
            sender,
        )?;

        let body = apply_mutations(
            Cursor::from_term_and_path(
                body,
                Path::Dynamic {
                    up: Box::new(up.clone()),
                    branch: Box::new(DefBranch::Body {
                        expression: c_expression,
                        binder: self.binder.clone(),
                    }),
                    annotation: UiSection {
                        variant: UiSectionVariance::Dynamic(annotation.clone()),
                    },
                },
            ),
            focused,
            sender,
        )?;

        for mutation in &mutations {
            match mutation {
                DefMutation::Update(name) => {
                    self.binder = if name.is_empty() {
                        None
                    } else {
                        Some(name.clone())
                    };
                }

                _ => {}
            }
        }

        let mut cursor = Cursor::Dynamic(DynamicCursor {
            up: up.clone(),
            term: Box::new(Def {
                expression: expression.into(),
                body: body.into(),
                binder: self.binder.clone(),
            }),
            annotation: UiSection {
                variant: UiSectionVariance::Dynamic(annotation),
            },
        });

        for mutation in &mutations {
            match mutation {
                DefMutation::Remove => {
                    cursor = Cursor::Hole(match cursor {
                        Cursor::Dynamic(_) => {
                            HoleCursor::new(up.clone(), ui_section(Term::Hole(()), sender))
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
                DefMutation::Focus => {
                    *focused = Some(cursor.clone());
                }
                _ => {}
            }
        }

        Ok(cursor)
    }

    fn render_to(
        &self,
        up: &Path<UiSection>,
        annotation: &dyn DynamicVariance,
        node: &web_sys::Node,
    ) -> Result<(), JsValue> {
        let annotation: &DefVariance = annotation.downcast_ref().unwrap();
        let container = &annotation.container;

        if !node.contains(Some(container)) {
            node.append_child(container)?;
        }

        if let Some(binder) = &self.binder {
            annotation.span.set_text_content(Some(binder));
        } else {
            annotation.span.set_text_content(Some(""));
        }

        let expression_node = container.child_nodes().get(1).unwrap();
        let body_node = container.child_nodes().get(2).unwrap();
        render_to(
            &Cursor::from_term_and_path(
                self.expression.clone(),
                Path::Dynamic {
                    up: Box::new(up.clone()),
                    branch: Box::new(DefBranch::Expression {
                        body: self.body.clone(),
                        binder: self.binder.clone(),
                    }),
                    annotation: UiSection {
                        variant: UiSectionVariance::Dynamic(Box::new(annotation.clone())),
                    },
                },
            ),
            &expression_node,
        )?;
        render_to(
            &Cursor::from_term_and_path(
                self.body.clone(),
                Path::Dynamic {
                    up: Box::new(up.clone()),
                    branch: Box::new(DefBranch::Body {
                        expression: self.expression.clone(),
                        binder: self.binder.clone(),
                    }),
                    annotation: UiSection {
                        variant: UiSectionVariance::Dynamic(Box::new(annotation.clone())),
                    },
                },
            ),
            &body_node,
        )?;

        Ok(())
    }

    fn clear_annotation(self: Box<Self>) -> Box<dyn DynamicTerm<()>> {
        Box::new(Def {
            expression: self.expression.clear_annotation(),
            body: self.body.clear_annotation(),
            binder: self.binder,
        })
    }

    fn index(&self) -> u8 {
        todo!()
    }

    fn encode(self: Box<Self>) -> Vec<u8> {
        todo!()
    }
}

impl<T> Def<T> {
    pub fn new(body: Term<T>, expression: Term<T>, binder: Option<String>) -> Self {
        Def {
            body,
            expression,
            binder,
        }
    }
}
