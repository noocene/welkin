#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (#[allow(unused_unsafe)] unsafe { web_sys::console::log_1(&format_args!($($t)*).to_string().into()) })
}

mod bindings;
mod edit;
mod evaluator;

use std::{cell::RefCell, error::Error, mem::replace, panic, rc::Rc};

use bincode::deserialize;
use bindings::w;
use edit::Scratchpad;
use evaluator::Evaluator;
use futures::{channel::mpsc::unbounded, SinkExt, Stream, StreamExt};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, KeyboardEvent};
use welkin_binding::{FromAnalogue, FromWelkin, ToWelkin};
use welkin_core::term::Term;

use async_recursion::async_recursion;

use crate::{bindings::io::iter::LoopRequest, evaluator::Substitution};

enum Block {
    Info { header: String, content: String },
    Printed { data: String },
    Scratchpad { data: Scratchpad },
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

        let push_paragraph = move |data: Block| {
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
                Block::Scratchpad { mut data } => {
                    let wrapper = document.create_element("div").unwrap();
                    wrapper.class_list().add_2("scratchpad", "wrapper").unwrap();
                    container.append_child(&wrapper).unwrap();
                    spawn_local(async move {
                        loop {
                            data.render_to(&wrapper).unwrap();
                            data.needs_update().await;
                            data = data.apply_mutations().unwrap();
                        }
                    });
                }
            };
            window.scroll_to_with_x_and_y(0., body.scroll_height() as f64);
        };

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

        let term = zipper::Term::Lambda {
            erased: false,
            annotation: (),
            name: Some("x".into()),
            body: Box::new(zipper::Term::Lambda {
                erased: false,
                annotation: (),
                name: Some("y".into()),
                body: Box::new(zipper::Term::Reference("x".into(), ())),
            }),
        };

        let scratchpad = Scratchpad::new(term);

        push_paragraph(Block::Scratchpad { data: scratchpad });

        run_io(io, &push_paragraph, &mut receive, &evaluator)
            .await
            .unwrap();
    });

    Ok(())
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
