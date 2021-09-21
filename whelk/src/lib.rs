mod evaluator;
mod interface;

use std::{cell::RefCell, panic, rc::Rc};

use bincode::deserialize;
use futures::{channel::mpsc::unbounded, SinkExt, StreamExt};
use interface::{whelk::Whelk, FromWelkin, Io, Unit};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, KeyboardEvent};
use welkin_core::term::Term;

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
        };

        let (send, mut receive) = unbounded();

        let term: Term<String> = deserialize(&term).unwrap();

        let mut whelk = (Whelk::from_welkin(term).unwrap().0).0;

        let evaluator = Substitution;

        let mut idx = 0;

        *input_submit_callback.borrow_mut() = Box::new(move |data| {
            let mut send = send.clone();
            spawn_local(async move {
                send.send(data).await.unwrap();
            });
        });

        loop {
            idx += 1;
            if idx > 20 {
                break;
            }

            match whelk {
                Io::Data(data) => {
                    push_paragraph(&format!("PURE) \n{:?}", data));
                    break;
                }
                Io::Request(request) => {
                    let req = request.request();
                    push_paragraph(&format!(
                        "REQUEST) \n{}",
                        match req {
                            whelk::Request::Prompt => "Prompt",
                            whelk::Request::Print(_) => "Print",
                        }
                    ));

                    match req {
                        whelk::Request::Prompt => {
                            push_paragraph(&format!("PROMPT)"));
                            let message = receive.next().await.unwrap();
                            push_paragraph(&format!("FULFILLED) {:?}", message));
                            let io = request
                                .fulfill(WSized(WString(message)).to_welkin().unwrap(), &evaluator)
                                .unwrap();
                            whelk = io;
                        }
                        whelk::Request::Print(data) => {
                            push_paragraph(&format!("PRINT) \n{:?}", (data.0).0));
                            let io = request
                                .fulfill(Unit.to_welkin().unwrap(), &evaluator)
                                .unwrap();
                            whelk = io
                        }
                    }
                }
            }
        }
    });

    Ok(())
}
