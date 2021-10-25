use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::{prelude::Closure, JsValue};
use web_sys::{Element, Node};

use crate::edit::DynamicVariance;

use super::mutations::*;

#[derive(Clone, Debug)]
pub enum UiSectionVariance {
    Lambda {
        p: Element,
        span: Element,
        container: Node,
        closures: Rc<Vec<Closure<dyn FnMut(JsValue)>>>,
        mutations: Rc<RefCell<Vec<LambdaMutation>>>,
    },
    Function {
        container: Element,
        span: Element,
        self_span: Element,
        closures: Rc<Vec<Closure<dyn FnMut(JsValue)>>>,
        self_focused: Rc<RefCell<bool>>,
        mutations: Rc<RefCell<Vec<FunctionMutation>>>,
    },
    Application {
        container: Element,
        closures: Rc<Vec<Closure<dyn FnMut(JsValue)>>>,
        mutations: Rc<RefCell<Vec<ApplicationMutation>>>,
    },
    Reference {
        p: Element,
        mutations: Rc<RefCell<Vec<ReferenceMutation>>>,
        closures: Rc<Vec<Closure<dyn FnMut(JsValue)>>>,
    },
    Hole {
        p: Element,
        mutations: Rc<RefCell<Vec<HoleMutation>>>,
        closures: Rc<Vec<Closure<dyn FnMut(JsValue)>>>,
    },
    Universe {
        p: Element,
        mutations: Rc<RefCell<Vec<UniverseMutation>>>,
        closures: Rc<Vec<Closure<dyn FnMut(JsValue)>>>,
    },
    Wrap {
        container: Element,
        content: Element,
        mutations: Rc<RefCell<Vec<WrapMutation>>>,
        closures: Rc<Vec<Closure<dyn FnMut(JsValue)>>>,
    },
    Put {
        container: Element,
        content: Element,
        mutations: Rc<RefCell<Vec<PutMutation>>>,
        closures: Rc<Vec<Closure<dyn FnMut(JsValue)>>>,
    },
    Duplication {
        container: Element,
        span: Element,
        closures: Rc<Vec<Closure<dyn FnMut(JsValue)>>>,
        mutations: Rc<RefCell<Vec<DuplicationMutation>>>,
    },
    Dynamic(Box<dyn DynamicVariance>),
}
