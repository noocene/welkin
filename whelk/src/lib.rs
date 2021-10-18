#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (#[allow(unused_unsafe)] unsafe { web_sys::console::log_1(&format_args!($($t)*).to_string().into()) })
}

use std::{cell::RefCell, panic, rc::Rc};

use edit::{
    zipper::{self, Cursor, TermData},
    Scratchpad, UiSection,
};
use futures::{
    channel::mpsc::{channel, Receiver},
    stream, StreamExt,
};
use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsCast, JsValue,
};
use wasm_bindgen_futures::spawn_local;
use web_sys::{Element, KeyboardEvent, Node};
use welkin_core::term::{DefinitionResult, MapCache, Term, TypedDefinitions};

mod bindings;
mod edit;
mod evaluator;

thread_local! {
    pub static CACHE: RefCell<MapCache> = RefCell::new(MapCache::new());
}

#[wasm_bindgen]
pub fn entry(_: Vec<u8>) -> Result<(), JsValue> {
    spawn_local(async move {
        main().await.unwrap();
    });
    Ok(())
}

async fn main() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let container = document.get_element_by_id("container").unwrap();

    let pads = Rc::new(RefCell::new(vec![]));

    let storage = window.local_storage().unwrap().unwrap();
    let data = storage.get_item("data").unwrap().unwrap_or_else(|| {
        let data = String::from_utf8_lossy(&base91::slice_encode(
            &bincode::serialize(&Vec::<TermData>::new()).unwrap(),
        ))
        .into_owned();

        storage.set_item("data", &data).unwrap();

        data
    });

    let data = base91::slice_decode(data.as_bytes());

    let mut data: Vec<TermData> = bincode::deserialize(&data).unwrap();

    if data.is_empty() {
        data.push(TermData::Hole);
    }

    for term in data {
        let pad = add_scratchpad(term.into(), pads.clone()).await?;

        container.append_child(&pad.wrapper)?;

        pads.borrow_mut().push(pad);
    }

    let closure = Closure::wrap(Box::new({
        let pads = pads.clone();
        move |e: JsValue| {
            let e: KeyboardEvent = e.dyn_into().unwrap();
            match e.code().as_str() {
                "KeyS" if e.ctrl_key() => {
                    save(pads.clone());
                    e.prevent_default();
                }
                _ => {}
            }
        }
    }) as Box<dyn FnMut(JsValue)>);

    window.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;

    closure.forget();

    Box::leak(Box::new(pads));

    Ok(())
}

fn save(pads: Rc<RefCell<Vec<ScratchpadContainer>>>) {
    let term_data: Vec<TermData> = pads
        .borrow()
        .iter()
        .map(|pad| {
            zipper::Term::<_>::from({
                let mut data = pad.data.borrow().clone();
                while !data.is_top() {
                    data = data.ascend()
                }
                data
            })
            .clear_annotation()
            .into()
        })
        .collect();
    let window = web_sys::window().unwrap();
    let storage = window.local_storage().unwrap().unwrap();
    let data = bincode::serialize(&term_data).unwrap();
    storage
        .set_item(
            "data",
            String::from_utf8_lossy(base91::slice_encode(&data).as_ref()).as_ref(),
        )
        .unwrap();
}

async fn add_scratchpad(
    term: zipper::Term,
    pads: Rc<RefCell<Vec<ScratchpadContainer>>>,
) -> Result<ScratchpadContainer, JsValue> {
    let call: Rc<RefCell<Box<dyn FnMut(JsValue)>>> = Rc::new(RefCell::new(Box::new(|_| {})));

    let mut pad = make_scratchpad(term, {
        let call = call.clone();
        move |e| {
            (&mut *call.borrow_mut())(e);
        }
    })
    .await?;

    let receiver = pad.receiver.take().unwrap();

    let wrapper = pad.wrapper.clone();

    *call.borrow_mut() = Box::new({
        let wrapper = wrapper.clone();
        let pads = pads.clone();
        move |e| {
            if e.dyn_ref::<KeyboardEvent>().is_none() {
                save(pads.clone());
                return;
            }
            let e: KeyboardEvent = e.dyn_into().unwrap();
            let code = e.code();
            match code.as_str() {
                "Enter" => {
                    e.prevent_default();
                    let wrapper = wrapper.clone();
                    let pads = pads.clone();
                    spawn_local(async move {
                        let pad = add_scratchpad(zipper::Term::Hole(()), pads.clone())
                            .await
                            .unwrap();
                        let idx = pads
                            .borrow()
                            .iter()
                            .position(|pad| pad.wrapper == wrapper)
                            .unwrap();
                        wrapper.after_with_node_1(&pad.wrapper).unwrap();
                        pads.borrow_mut().insert(idx + 1, pad);
                    });
                }
                "Delete" | "Backspace" => {
                    if !e
                        .target()
                        .unwrap()
                        .dyn_ref::<Element>()
                        .unwrap()
                        .class_list()
                        .contains("hole")
                    {
                        return;
                    }
                    let pads = pads.clone();
                    let wrapper = wrapper.clone();
                    spawn_local(async move {
                        let idx = pads
                            .borrow()
                            .iter()
                            .position(|pad| pad.wrapper == wrapper)
                            .unwrap();
                        e.stop_propagation();
                        e.prevent_default();
                        if pads.borrow().len() > 1 {
                            let pad = pads.borrow_mut().remove(idx);
                            pad.wrapper.remove();
                            Box::leak(Box::new(pad));
                        }
                    });
                }
                _ => {}
            }
        }
    });

    spawn_local({
        let wrapper = wrapper.clone();
        let status = pad.status.clone();
        let data = pad.data.clone();
        let pads = pads.clone();
        async move {
            let mut receiver = Box::pin(stream::once(async { () }).chain(receiver));

            while let Some(()) = receiver.next().await {
                status.remove_attribute("class").unwrap();
                if let Some(term) = {
                    let mut data = data.borrow().clone();
                    while !data.is_top() {
                        data = data.ascend();
                    }
                    data.into_term()
                } {
                    if let Some((name, ty, term)) = read_definition(&term) {
                        CACHE.with(|cache| {
                            let defs = DefWrapper(pads.clone());
                            let cache = &mut *cache.borrow_mut();
                            if let Ok(()) = ty.check(&Term::Universe, &defs, cache) {
                                if let Ok(()) = term.check(&ty, &defs, cache) {
                                    status
                                        .class_list()
                                        .add_3("scratchpad", "status", "def-ok")
                                        .unwrap();
                                    return;
                                }
                            }
                            status
                                .class_list()
                                .add_3("scratchpad", "status", "def-err")
                                .unwrap();
                        });
                        continue;
                    }
                }
                status.class_list().add_2("scratchpad", "status").unwrap();
            }
        }
    });

    Ok(pad)
}

pub struct DefWrapper(Rc<RefCell<Vec<ScratchpadContainer>>>);

impl TypedDefinitions<String> for DefWrapper {
    fn get_typed(&self, n: &String) -> Option<DefinitionResult<(Term<String>, Term<String>)>> {
        self.0.borrow().iter().find_map(|a| {
            let a = {
                let mut data = a.data.borrow().clone();
                while !data.is_top() {
                    data = data.ascend();
                }
                data.into_term().as_ref().and_then(read_definition)
            };
            if let Some((name, ty, term)) = a {
                if &name == n {
                    return Some(DefinitionResult::Owned((ty, term)));
                }
            }
            None
        })
    }
}

fn read_definition(term: &Term<String>) -> Option<(String, Term<String>, Term<String>)> {
    if let Term::Apply {
        function,
        argument,
        erased: false,
    } = term
    {
        let term = &**argument;

        if let Term::Apply {
            function,
            argument,
            erased: false,
        } = &**function
        {
            let ty = &**argument;

            if let Term::Apply {
                function,
                argument,
                erased: false,
            } = &**function
            {
                if function.equals(&Term::Reference("define".into())) {
                    if let Term::Reference(name) = &**argument {
                        return Some((name.clone(), ty.clone(), term.clone()));
                    }
                }
            }
        }
    }

    None
}

struct ScratchpadContainer {
    wrapper: Element,
    data: Rc<RefCell<Cursor<UiSection>>>,
    receiver: Option<Receiver<()>>,
    status: Element,
    _closures: Vec<Closure<dyn FnMut(JsValue)>>,
}

impl ScratchpadContainer {
    fn add_to(&self, target: &Node) -> Result<(), JsValue> {
        if !target.contains(Some(&self.wrapper)) {
            target.append_child(&self.wrapper)?;
        }

        Ok(())
    }
}

async fn make_scratchpad(
    term: zipper::Term,
    mut listener: impl FnMut(JsValue) + 'static,
) -> Result<ScratchpadContainer, JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let wrapper = document.create_element("div")?;
    wrapper.class_list().add_2("scratchpad", "wrapper")?;

    let content = document.create_element("div")?;
    content.class_list().add_2("scratchpad", "content")?;

    let status = document.create_element("div")?;
    status.class_list().add_2("scratchpad", "status")?;

    wrapper.append_child(&content)?;
    wrapper.append_child(&status)?;

    let mut pad = Scratchpad::new(term, content.into());

    pad.render()?;

    pad.focus();

    let data = pad.data();

    let (mut sender, receiver) = channel(0);

    spawn_local(async move {
        loop {
            pad.needs_update().await;

            let _ = sender.try_send(());

            pad = pad.apply_mutations().unwrap();

            pad.render().unwrap();
        }
    });

    let keydown_listener =
        Closure::wrap(Box::new(move |e: JsValue| listener(e)) as Box<dyn FnMut(JsValue)>);

    wrapper
        .add_event_listener_with_callback("keydown", keydown_listener.as_ref().unchecked_ref())?;
    wrapper
        .add_event_listener_with_callback("focusout", keydown_listener.as_ref().unchecked_ref())?;

    Ok(ScratchpadContainer {
        wrapper,
        data,
        status,
        _closures: vec![keydown_listener],
        receiver: Some(receiver),
    })
}
