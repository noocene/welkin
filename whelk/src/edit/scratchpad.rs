use std::{cell::RefCell, rc::Rc, task::Context};

use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    task::noop_waker,
    Future, StreamExt,
};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{ClipboardEvent, Element, FocusEvent, KeyboardEvent, Node};

use crate::edit::{apply_mutations, mutations::HoleMutation, render_to, zipper::TermData};

use super::{
    add_ui,
    zipper::{Cursor, Term},
    OnChangeWrapper, UiSection, UiSectionVariance,
};

#[derive(Debug)]
pub struct Scratchpad {
    data: Rc<RefCell<Cursor<UiSection>>>,
    needs_update: Receiver<()>,
    sender: Sender<()>,
    clipboard_event_handler: Closure<dyn FnMut(JsValue)>,
    focus_event_handler: Closure<dyn FnMut(JsValue)>,
    has_focus: Rc<RefCell<bool>>,
    target_node: Node,
    on_change: OnChangeWrapper,
}

impl Scratchpad {
    pub fn new(term: Term, target_node: Node) -> Self {
        let (mut sender, receiver) = channel(0);
        let data = Rc::new(RefCell::new(add_ui(term, &sender).into()));

        let has_focus = Rc::new(RefCell::new(false));

        let focus_event_handler = Closure::wrap(Box::new({
            let has_focus = has_focus.clone();
            move |e: JsValue| {
                let e: FocusEvent = e.dyn_into().unwrap();

                match e.type_().as_str() {
                    "focusin" => {
                        *has_focus.borrow_mut() = true;
                    }
                    "focusout" => {
                        if e.related_target().is_some() {
                            *has_focus.borrow_mut() = false;
                        }
                    }
                    _ => panic!(),
                }
            }
        }) as Box<dyn FnMut(JsValue)>);

        target_node
            .add_event_listener_with_callback(
                "focusin",
                focus_event_handler.as_ref().unchecked_ref(),
            )
            .unwrap();
        target_node
            .add_event_listener_with_callback(
                "focusout",
                focus_event_handler.as_ref().unchecked_ref(),
            )
            .unwrap();

        let scratchpad = Scratchpad {
            data: data.clone(),
            needs_update: receiver,
            target_node,
            has_focus,
            sender: sender.clone(),
            on_change: OnChangeWrapper::new(),
            focus_event_handler,
            clipboard_event_handler: Closure::wrap(Box::new(move |e: JsValue| {
                let copy = |f: Box<dyn Fn(String)>| {
                    let data = &mut *data.borrow_mut();

                    let term: Term<()> = Term::<UiSection>::from(data.clone()).clear_annotation();

                    let waker = noop_waker();
                    let mut context = Context::from_waker(&waker);

                    let mut fut = Box::pin(TermData::from(term.clear_annotation()).encode());

                    let data = loop {
                        match fut.as_mut().poll(&mut context) {
                            std::task::Poll::Ready(data) => break data,
                            std::task::Poll::Pending => {}
                        }
                    };

                    if let Ok(data) = data {
                        f(data);
                    }
                };

                if let Some(e) = e.dyn_ref::<KeyboardEvent>() {
                    if e.ctrl_key() {
                        if e.key() == "c" {
                            copy(Box::new(|data| {
                                spawn_local(async move {
                                    let navigator = web_sys::window().unwrap().navigator();
                                    let clipboard = navigator.clipboard().unwrap();
                                    JsFuture::from(clipboard.write_text(&data)).await.unwrap();
                                });
                            }));
                            e.prevent_default();
                            e.stop_propagation();
                        } else if e.key() == "x" {
                            copy(Box::new(|data| {
                                spawn_local(async move {
                                    let navigator = web_sys::window().unwrap().navigator();
                                    let clipboard = navigator.clipboard().unwrap();
                                    JsFuture::from(clipboard.write_text(&data)).await.unwrap();
                                });
                            }));
                            data.borrow().annotation().trigger_remove(&sender);
                            e.prevent_default();
                            e.stop_propagation();
                        }
                    }

                    return;
                }
                let e: ClipboardEvent = e.dyn_into().unwrap();
                let c_data = e.clipboard_data().unwrap();

                match e.type_().as_str() {
                    "cut" => {
                        copy(Box::new(|data| {
                            c_data.set_data("text/plain", &data).unwrap();
                        }));
                        data.borrow().annotation().trigger_remove(&sender);
                        e.prevent_default();
                    }
                    "copy" => {
                        copy(Box::new(|data| {
                            c_data.set_data("text/plain", &data).unwrap();
                        }));
                        e.prevent_default();
                    }
                    "paste" => {
                        let data = &mut *data.borrow_mut();

                        if let Cursor::Hole(cursor) = data {
                            e.prevent_default();
                            if let Ok(data) = c_data.get_data("text/plain") {
                                let waker = noop_waker();
                                let mut context = Context::from_waker(&waker);

                                let mut fut = Box::pin(TermData::decode(data));

                                let data = loop {
                                    match fut.as_mut().poll(&mut context) {
                                        std::task::Poll::Ready(data) => break data,
                                        std::task::Poll::Pending => {}
                                    }
                                };

                                if let Ok(Some(data)) = data {
                                    match &mut cursor.annotation_mut().variant {
                                        UiSectionVariance::Hole { p, mutations, .. } => {
                                            p.remove();
                                            mutations.borrow_mut().push(HoleMutation::Replace(
                                                add_ui(data.into(), &sender),
                                            ));
                                        }
                                        _ => panic!(),
                                    }
                                    let _ = sender.try_send(());
                                }
                            }
                        }
                    }
                    _ => panic!(),
                }
            }) as Box<dyn FnMut(JsValue)>),
        };

        scratchpad.add_copy_listener();

        scratchpad
    }

    pub fn force_update(&mut self, data: Term) {
        *self.data.borrow_mut() = add_ui(data, &self.sender).into();
        for i in 0..self.target_node.child_nodes().length() {
            if let Some(el) = self
                .target_node
                .child_nodes()
                .get(i)
                .unwrap()
                .dyn_ref::<Element>()
            {
                el.remove()
            }
        }
        let _ = self.sender.try_send(());
    }

    pub fn cursor(&self) -> Cursor<UiSection> {
        self.data.borrow().clone()
    }

    pub fn data(&self) -> Rc<RefCell<Cursor<UiSection>>> {
        self.data.clone()
    }

    pub async fn needs_update(&mut self) {
        self.needs_update.next().await.unwrap()
    }

    fn root_el(&self) -> Element {
        let annotation = self.data.borrow();
        let annotation = annotation.annotation();

        match &annotation.variant {
            UiSectionVariance::Lambda { span, .. } => span.clone(),
            UiSectionVariance::Application { container, .. } => container.clone(),
            UiSectionVariance::Reference { p, .. } => p.clone(),
            UiSectionVariance::Hole { p, .. } => p.clone(),
            UiSectionVariance::Universe { p, .. } => p.clone(),
            UiSectionVariance::Wrap { container, .. } => container.clone(),
            UiSectionVariance::Put { container, .. } => container.clone(),
            UiSectionVariance::Duplication { span, .. } => span.clone(),
            UiSectionVariance::Function {
                self_span,
                span,
                self_focused,
                ..
            } => if *self_focused.borrow() {
                self_span
            } else {
                span
            }
            .clone(),
            UiSectionVariance::Dynamic(variance) => variance.focused_el().into_owned(),
        }
    }

    fn add_copy_listener(&self) {
        let el = self.root_el();
        el.add_event_listener_with_callback(
            "cut",
            self.clipboard_event_handler.as_ref().unchecked_ref(),
        )
        .unwrap();
        el.add_event_listener_with_callback(
            "copy",
            self.clipboard_event_handler.as_ref().unchecked_ref(),
        )
        .unwrap();
        el.add_event_listener_with_callback(
            "paste",
            self.clipboard_event_handler.as_ref().unchecked_ref(),
        )
        .unwrap();
        el.add_event_listener_with_callback(
            "keydown",
            self.clipboard_event_handler.as_ref().unchecked_ref(),
        )
        .unwrap();
    }

    pub fn apply_mutations(mut self) -> Result<Self, JsValue> {
        let perf = web_sys::window().unwrap().performance().unwrap();
        let time = perf.now();

        let mut focused = None;
        let mut data = self.data.borrow().clone();

        let el = self.root_el();
        el.remove_event_listener_with_callback(
            "cut",
            self.clipboard_event_handler.as_ref().unchecked_ref(),
        )?;
        el.remove_event_listener_with_callback(
            "copy",
            self.clipboard_event_handler.as_ref().unchecked_ref(),
        )?;
        el.remove_event_listener_with_callback(
            "paste",
            self.clipboard_event_handler.as_ref().unchecked_ref(),
        )?;
        el.remove_event_listener_with_callback(
            "keydown",
            self.clipboard_event_handler.as_ref().unchecked_ref(),
        )?;

        data = apply_mutations(data, &mut focused, &self.sender)?;

        {
            let mut data = data.clone();
            while !data.is_top() {
                data = data.ascend();
            }
            apply_mutations(data, &mut focused, &self.sender)?;
        }

        if let Some(focused) = focused {
            data = focused;
        }

        self.add_copy_listener();

        *self.data.borrow_mut() = data;

        self.on_change.call(&*self.data.borrow());

        console_log!("update took {:.1}ms", perf.now() - time);

        Ok(self)
    }

    pub fn on_change(&mut self, closure: Box<dyn FnMut(&Cursor<UiSection>)>) {
        self.on_change.to_call.push(closure);
    }

    pub fn render(&self) -> Result<(), JsValue> {
        let perf = web_sys::window().unwrap().performance().unwrap();
        let time = perf.now();

        let mut data = self.data.borrow().clone();

        while !data.is_top() {
            data = data.ascend();
        }

        render_to(&data, &self.target_node)?;

        spawn_local({
            let data = self.data.clone();
            let has_focus = *self.has_focus.borrow();
            async move {
                if has_focus {
                    data.borrow().annotation().focus();
                }
            }
        });

        console_log!("render took {:.1}ms", perf.now() - time);

        Ok(())
    }

    pub fn focus(&self) {
        let data = self.data.clone();
        spawn_local(async move {
            data.borrow().annotation().focus();
        });
    }
}
