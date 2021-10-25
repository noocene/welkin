mod variance;
use std::{cell::RefCell, rc::Rc};

use futures::channel::mpsc::Sender;
#[doc(inline)]
pub use variance::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, Node};

use super::{
    focus_contenteditable, focus_element,
    zipper::{Cursor, RefCount, Term},
};

pub mod mutations;
use mutations::*;

mod add_ui;
#[doc(inline)]
pub use add_ui::*;

#[derive(Debug, Clone)]
pub struct UiSection {
    pub(crate) variant: UiSectionVariance,
    pub(crate) annotation: Rc<RefCell<Term<(), RefCount>>>,
}

impl UiSection {
    pub(crate) fn new(variant: UiSectionVariance) -> Self {
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
