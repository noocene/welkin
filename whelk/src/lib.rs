mod interface;

use std::{cell::RefCell, panic, rc::Rc};

use bincode::deserialize;
use interface::{FromWelkin, Whelk};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{HtmlInputElement, KeyboardEvent};
use welkin_core::term::Term;

#[wasm_bindgen]
pub fn entry(term: Vec<u8>) -> Result<(), JsValue> {
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
        .dyn_into()?;

    let input_submit_callback = Rc::new(RefCell::new(Box::new(|_: String| {}) as Box<dyn Fn(_)>));

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

    input.add_event_listener_with_callback(
        "keydown",
        input_submit_handler.as_ref().unchecked_ref(),
    )?;

    input_submit_handler.forget();

    let push_paragraph = move |data: &str| {
        let paragraph = document.create_element("p").unwrap();
        paragraph.set_text_content(Some(data));
        container.append_child(&paragraph).unwrap();
    };

    let term: Term<String> = deserialize(&term).unwrap();

    let whelk = Whelk::from_welkin(term).unwrap();

    *input_submit_callback.borrow_mut() = Box::new(move |data| {
        push_paragraph(&whelk.call(data).unwrap());
    });

    Ok(())
}
