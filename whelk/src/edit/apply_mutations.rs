use futures::channel::mpsc::Sender;
use wasm_bindgen::JsValue;

use crate::edit::{mutations::*, ui_section, zipper::Term, UiSectionVariance};

use super::{zipper::Cursor, UiSection};

pub fn apply_mutations(
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
                match &cursor.annotation.variant {
                    UiSectionVariance::Dynamic(variance) => variance.clone(),
                    _ => panic!(),
                },
                focused,
                sender,
            )?
        }
    })
}
