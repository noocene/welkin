mod evaluator;
mod interface;

use std::{cell::RefCell, error::Error, panic, rc::Rc};

use bincode::deserialize;
use evaluator::Evaluator;
use futures::{channel::mpsc::unbounded, SinkExt, Stream, StreamExt};
use interface::{
    whelk::{Request, Whelk},
    FromWelkin, Io, Unit,
};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, KeyboardEvent};
use welkin_core::term::Term;

use async_recursion::async_recursion;

use crate::{
    evaluator::Substitution,
    interface::{whelk, ToWelkin, WSized, WString},
};

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

        let push_paragraph = move |data: &str| {
            let paragraph = document.create_element("p").unwrap();
            paragraph.set_text_content(Some(data));
            container.append_child(&paragraph).unwrap();
            window.scroll_to_with_x_and_y(0., body.scroll_height() as f64);
        };

        let (send, mut receive) = unbounded();

        let term: Term<String> = deserialize(&term).unwrap();

        let io = (Whelk::from_welkin(term).unwrap().0).0;

        let evaluator = Substitution;

        *input_submit_callback.borrow_mut() = Box::new(move |data| {
            let mut send = send.clone();
            spawn_local(async move {
                send.send(data).await.unwrap();
            });
        });

        run_io(io, &push_paragraph, &mut receive, &evaluator)
            .await
            .unwrap();
    });

    Ok(())
}

#[async_recursion(?Send)]
async fn run_io<
    F: Fn(&str),
    R: Stream<Item = String> + Unpin,
    E: Evaluator + 'static,
    D: FromWelkin + 'static,
>(
    io: Io<Request, D>,
    push_paragraph: &F,
    receive: &mut R,
    evaluator: &E,
) -> Result<D, anyhow::Error>
where
    E::Error: Error + Send + Sync,
    <D as FromWelkin>::Error: Send + Sync + Error,
{
    match io {
        Io::Data(data) => Ok(data),
        Io::Request(request) => {
            let req = request.request();
            push_paragraph(&format!(
                "REQUEST) \n{}",
                match req {
                    whelk::Request::Loop(_) => "Loop",
                    whelk::Request::Prompt => "Prompt",
                    whelk::Request::Print(_) => "Print",
                }
            ));

            match req {
                whelk::Request::Loop(request) => {
                    let mut request = request.clone();
                    loop {
                        request
                            .step(&*evaluator, |io| async {
                                run_io(io, &*push_paragraph, &mut *receive, &*evaluator).await
                            })
                            .await?;
                        if request.proceed(&*evaluator)? {
                            continue;
                        } else {
                            break Ok(FromWelkin::from_welkin(request.into_state())?);
                        }
                    }
                }
                whelk::Request::Prompt => {
                    push_paragraph(&format!("PROMPT)"));
                    let message = receive.next().await.unwrap();
                    push_paragraph(&format!("FULFILLED) {:?}", message));
                    let io = request
                        .fulfill(WSized(WString(message)).to_welkin().unwrap(), &*evaluator)?;
                    run_io(io, &*push_paragraph, &mut *receive, &*evaluator).await
                }
                whelk::Request::Print(data) => {
                    push_paragraph(&format!("PRINT) \n{:?}", (data.0).0));
                    let io = request.fulfill(Unit.to_welkin().unwrap(), &*evaluator)?;
                    run_io(io, &*push_paragraph, &mut *receive, &*evaluator).await
                }
            }
        }
    }
}
