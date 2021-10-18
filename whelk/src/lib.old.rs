#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (#[allow(unused_unsafe)] unsafe { web_sys::console::log_1(&format_args!($($t)*).to_string().into()) })
}

mod bindings;
mod edit;
mod evaluator;

use std::{
    cell::RefCell, collections::HashMap, error::Error, mem::replace, panic, pin::Pin, rc::Rc,
};

use bincode::deserialize;
use bindings::w;
use edit::{
    zipper::{self, Cursor},
    Scratchpad, UiSection,
};
use evaluator::Evaluator;
use futures::{
    channel::mpsc::{channel, unbounded, Sender},
    future::{select, Either},
    stream, SinkExt, Stream, StreamExt,
};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::{Element, HtmlInputElement, KeyboardEvent};
use welkin_binding::{FromAnalogue, FromWelkin, ToWelkin};
use welkin_core::term::{DefinitionResult, MapCache, Term, TypedDefinitions};

use async_recursion::async_recursion;

use crate::{bindings::io::iter::LoopRequest, evaluator::Substitution};

enum Block {
    Info {
        header: String,
        content: String,
    },
    Printed {
        data: String,
    },
    Scratchpad {
        data: zipper::Term,
        on_update: Box<dyn FnMut(&Cursor<UiSection>)>,
        after: Option<Element>,
        event_listener: Box<dyn FnMut(JsValue, &Element)>,
        force_update:
            Pin<Box<dyn Stream<Item = Box<dyn FnOnce(&mut Scratchpad) -> Option<zipper::Term>>>>>,
    },
}

#[wasm_bindgen]
pub fn entry(term: Vec<u8>) -> Result<(), JsValue> {
    spawn_local(async move {
        panic::set_hook(Box::new(console_error_panic_hook::hook));

        let window = web_sys::window().expect("no global window");

        let document = window
            .document()
            .expect("global window has no attached document");

        let body = document.body().expect("document has no body element");

        let container = document
            .get_element_by_id("container")
            .expect("body element missing");

        let input: HtmlInputElement = document
            .get_element_by_id("input")
            .expect("input element missing")
            .dyn_into()
            .unwrap();

        let input_submit_callback =
            Rc::new(RefCell::new(Box::new(|_: String| {}) as Box<dyn Fn(_)>));

        let input_submit_handler = Closure::wrap(Box::new({
            let callback = input_submit_callback.clone();
            let input = input.clone();
            move |e: KeyboardEvent| {
                if e.key_code() == 13 {
                    (callback.borrow())(input.value());
                    input.set_value("");
                }
            }
        }) as Box<dyn Fn(_)>);

        input
            .add_event_listener_with_callback(
                "keydown",
                input_submit_handler.as_ref().unchecked_ref(),
            )
            .unwrap();

        input_submit_handler.forget();

        let push_paragraph = Rc::new(move |data: Block| {
            match data {
                Block::Info { header, content } => {
                    let paragraph = document.create_element("p").unwrap();
                    let header_span = document.create_element("span").unwrap();
                    header_span.class_list().add_1("info-header").unwrap();
                    paragraph.class_list().add_1("info").unwrap();
                    header_span.set_text_content(Some(&header));
                    paragraph.append_child(&header_span).unwrap();
                    let text = document.create_text_node(&content);
                    paragraph.append_child(&text).unwrap();
                    container.append_child(&paragraph).unwrap();
                }
                Block::Printed { data } => {
                    let wrapper = document.create_element("div").unwrap();
                    wrapper.class_list().add_2("printed", "wrapper").unwrap();
                    let paragraph = document.create_element("p").unwrap();
                    paragraph.class_list().add_2("printed", "content").unwrap();
                    paragraph.set_text_content(Some(&data));
                    wrapper.append_child(&paragraph).unwrap();
                    container.append_child(&wrapper).unwrap();
                }
                Block::Scratchpad {
                    data,
                    on_update,
                    after,
                    mut event_listener,
                    mut force_update,
                } => {
                    let wrapper = document.create_element("div").unwrap();
                    wrapper.class_list().add_2("scratchpad", "wrapper").unwrap();

                    if let Some(after) = after {
                        after.after_with_node_1(&wrapper).unwrap();
                    } else {
                        container.append_child(&wrapper).unwrap();
                    }

                    let el = Closure::wrap(Box::new({
                        let wrapper = wrapper.clone();
                        move |e: JsValue| {
                            event_listener(e, &wrapper);
                        }
                    }) as Box<dyn FnMut(JsValue)>);

                    wrapper
                        .add_event_listener_with_callback("keydown", el.as_ref().unchecked_ref())
                        .unwrap();

                    el.forget();

                    let mut scratchpad = Scratchpad::new(data, wrapper.into());
                    scratchpad.on_change(on_update);

                    scratchpad.render().unwrap();

                    spawn_local(async move {
                        loop {
                            let data = match select(
                                Box::pin(scratchpad.needs_update()),
                                force_update.next(),
                            )
                            .await
                            {
                                Either::Left(_) => None,
                                Either::Right((data, _)) => {
                                    if let Some(data) = data {
                                        Some(data)
                                    } else {
                                        panic!()
                                    }
                                }
                            };

                            if let Some(data) = data {
                                if let Some(update) = data(&mut scratchpad) {
                                    scratchpad.force_update(update);
                                } else {
                                    continue;
                                }
                            }

                            scratchpad = scratchpad.apply_mutations().unwrap();

                            scratchpad.render().unwrap();
                        }
                    });
                }
            };
            window.scroll_to_with_x_and_y(0., body.scroll_height() as f64);
        });

        let (send, mut receive) = unbounded();

        let term: Term<String> = deserialize(&term).unwrap();

        let whelk = w::Whelk::from_welkin(term.clone()).unwrap();

        let io = match whelk {
            w::Whelk::new { data } => match data {
                w::BoxPoly::new { data } => data,
            },
        };

        let evaluator = Substitution;

        *input_submit_callback.borrow_mut() = Box::new(move |data| {
            let mut send = send.clone();
            spawn_local(async move {
                send.send(data).await.unwrap();
            });
        });

        let mut terms = Rc::new(RefCell::new(vec![]));

        let (sender, receiver) = channel(0);

        fn push_pad(
            push_paragraph: Rc<impl Fn(Block) + 'static>,
            after: Option<Element>,
            terms: Rc<RefCell<Vec<Rc<RefCell<Option<Definition>>>>>>,
            sender: &Sender<()>,
        ) {
            let mut data = Rc::new(RefCell::new(None));
            terms.borrow_mut().push(data.clone());
            push_paragraph(Block::Scratchpad {
                data: zipper::Term::Hole(()),
                after,
                event_listener: Box::new({
                    let push_paragraph = push_paragraph.clone();
                    let terms = terms.clone();
                    let sender = sender.clone();
                    move |ev, el| {
                        if let Some(ev) = ev.dyn_ref::<KeyboardEvent>() {
                            if ev.code() == "Enter" {
                                push_pad(
                                    push_paragraph.clone(),
                                    Some(el.clone()),
                                    terms.clone(),
                                    &sender,
                                )
                            }
                        }
                    }
                }),
                on_update: Box::new({
                    let mut sender = sender.clone();
                    move |cursor| {
                        let mut cursor = cursor.clone();

                        while !cursor.is_top() {
                            cursor = cursor.ascend();
                        }

                        let term = cursor.clone().into_term();

                        let def = term.and_then(|a| process_entry(a).ok());

                        if def.is_some() {
                            let _ = sender.try_send(());
                        }

                        *data.borrow_mut() = def;
                    }
                }),
                force_update: Box::pin(stream::pending()),
            });
        }

        push_pad(push_paragraph, None, terms.clone(), &sender);

        spawn_local(async move {
            let mut receiver = receiver;
            let defs = DefWrapper(terms.clone());
            let mut cache = MapCache::new();

            while let Some(()) = receiver.next().await {
                for term in &*terms.borrow() {
                    if let Some(term) = &*term.borrow() {
                        console_log!("{:?}", term.ty.check(&Term::Universe, &defs, &mut cache));
                        console_log!("{:?}", term.term.check(&term.ty, &defs, &mut cache));
                    }
                }
            }
        });

        run_io(io, &|_| {}, &mut receive, &evaluator).await.unwrap();
    });

    Ok(())
}

pub struct DefWrapper(Rc<RefCell<Vec<Rc<RefCell<Option<Definition>>>>>>);

impl TypedDefinitions<String> for DefWrapper {
    fn get_typed(&self, n: &String) -> Option<DefinitionResult<(Term<String>, Term<String>)>> {
        self.0.borrow().iter().find_map(|a| {
            let a = &*a.borrow();
            if let Some(Definition { name, ty, term }) = a {
                if name == n {
                    return Some(DefinitionResult::Owned((ty.clone(), term.clone())));
                }
            }
            None
        })
    }
}

#[derive(Debug, Clone)]
pub struct Definition {
    name: String,
    ty: Term<String>,
    term: Term<String>,
}

fn process_entry(term: Term<String>) -> Result<Definition, anyhow::Error> {
    if let Term::Apply {
        function,
        argument,
        erased: false,
    } = term
    {
        let term = *argument;

        if let Term::Apply {
            function,
            argument,
            erased: false,
        } = *function
        {
            let ty = *argument;

            if let Term::Apply {
                function,
                argument,
                erased: false,
            } = *function
            {
                if function.equals(&Term::Reference("define".into())) {
                    if let Term::Reference(name) = *argument {
                        return Ok(Definition { name, ty, term });
                    }
                }
            }
        }
    }

    Err(anyhow::anyhow!("invalid definition"))
}

#[async_recursion(?Send)]
async fn run_io<
    F: Fn(Block),
    R: Stream<Item = String> + Unpin,
    E: Evaluator + 'static,
    D: Clone + FromAnalogue + 'static,
>(
    mut io: w::WhelkIO<D>,
    push_paragraph: &F,
    receive: &mut R,
    evaluator: &E,
) -> Result<D, anyhow::Error>
where
    E::Error: Error + Send + Sync,
    <<D as FromAnalogue>::Analogue as FromWelkin>::Error: Send + Sync + Error,
{
    match io {
        w::IO::end { value } => Ok(value),
        w::IO::call {
            ref mut request, ..
        } => {
            push_paragraph(Block::Info {
                header: "REQ".into(),
                content: match request {
                    w::WhelkRequest::r#loop { .. } => "loop",
                    w::WhelkRequest::r#prompt { .. } => "prompt",
                    w::WhelkRequest::r#print { .. } => "print",
                }
                .into(),
            });

            match request {
                w::WhelkRequest::r#loop {
                    initial,
                    r#continue,
                    step,
                } => {
                    let mut request = LoopRequest::new(
                        replace(&mut initial.0, Term::Universe),
                        replace(&mut r#continue.0, Term::Universe),
                        replace(&mut step.0, Term::Universe),
                    );
                    loop {
                        request
                            .step(&*evaluator, |io| async {
                                run_io(io, &*push_paragraph, &mut *receive, &*evaluator).await
                            })
                            .await?;
                        if request.proceed(&*evaluator)? {
                            continue;
                        } else {
                            break Ok(FromAnalogue::from_analogue(FromWelkin::from_welkin(
                                request.into_state(),
                            )?));
                        }
                    }
                }
                w::WhelkRequest::prompt { .. } => {
                    let message = receive.next().await.unwrap();
                    push_paragraph(Block::Info {
                        header: "FUL".into(),
                        content: format!("{:?}", message.clone()),
                    });
                    let io = io.into_request().unwrap().fulfill(
                        w::Sized::<w::String>::new {
                            size: message.len().into(),
                            data: message.into(),
                        }
                        .to_welkin()
                        .unwrap(),
                        &*evaluator,
                    )?;
                    run_io(io, &*push_paragraph, &mut *receive, &*evaluator).await
                }
                w::WhelkRequest::print { data } => {
                    push_paragraph(Block::Printed {
                        data: match data {
                            w::Sized::new { data, .. } => data.clone().into(),
                        },
                    });
                    let io = io
                        .into_request()
                        .unwrap()
                        .fulfill(w::Unit::new.to_welkin().unwrap(), &*evaluator)?;
                    run_io(io, &*push_paragraph, &mut *receive, &*evaluator).await
                }
            }
        }
    }
}
