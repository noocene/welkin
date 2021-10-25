use std::fmt;

use downcast_rs::{impl_downcast, Downcast};

use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, Node};
use zipper::Cursor;

mod scratchpad;
#[doc(inline)]
pub use scratchpad::*;

mod ui_section;
#[doc(inline)]
pub use ui_section::*;

mod apply_mutations;
use apply_mutations::apply_mutations;

mod render_to;
use render_to::render_to;

pub mod dynamic;
pub mod zipper;

#[allow(dead_code)]
fn focus_contenteditable(p: &Element, always: bool) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    if document.active_element() == Some(p.clone()) && !always {
        return;
    }

    let selection = window.get_selection().unwrap().unwrap();
    let range = document.create_range().unwrap();
    range.set_start(p, 0).unwrap();
    range
        .set_end(p, Node::from(p.clone()).child_nodes().length())
        .unwrap();
    selection.remove_all_ranges().unwrap();
    selection.add_range(&range).unwrap();
}

#[allow(dead_code)]
fn focus_element(p: &Element, always: bool) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let selection = window.get_selection().unwrap().unwrap();
    selection.remove_all_ranges().unwrap();

    if document.active_element() == Some(p.clone()) && !always {
        return;
    }

    p.dyn_ref::<HtmlElement>().unwrap().focus().unwrap();
}

impl Clone for Box<dyn DynamicVariance> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

impl fmt::Debug for Box<dyn DynamicVariance> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug(f)
    }
}

pub trait DynamicVariance: Downcast {
    fn box_clone(&self) -> Box<dyn DynamicVariance>;
    fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result;
    fn focus(&self);
    fn remove(&self);
    fn focused_el(&self) -> &Element;
}

impl_downcast!(DynamicVariance);

fn configure_contenteditable(el: &Element) {
    el.set_attribute("contenteditable", "true").unwrap();
    el.set_attribute("tabindex", "0").unwrap();
    el.set_attribute("spellcheck", "false").unwrap();
    el.set_attribute("autocorrect", "off").unwrap();
    el.set_attribute("autocapitalize", "off").unwrap();
}

struct OnChangeWrapper {
    to_call: Vec<Box<dyn FnMut(&Cursor<UiSection>)>>,
}

impl OnChangeWrapper {
    fn new() -> Self {
        OnChangeWrapper { to_call: vec![] }
    }

    fn call(&mut self, data: &Cursor<UiSection>) {
        for call in &mut self.to_call {
            call(data);
        }
    }
}

impl std::fmt::Debug for OnChangeWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OnChangeWrapper").finish()
    }
}

struct OnRemoveWrapper {
    to_call: Vec<Box<dyn FnMut()>>,
}

impl OnRemoveWrapper {
    fn new() -> Self {
        OnRemoveWrapper { to_call: vec![] }
    }

    fn call(&mut self) {
        for call in &mut self.to_call {
            call();
        }
    }
}

impl std::fmt::Debug for OnRemoveWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OnRemoveWrapper").finish()
    }
}
