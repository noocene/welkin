#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (#[allow(unused_unsafe)] unsafe { web_sys::console::log_1(&format_args!($($t)*).to_string().into()) })
}

use std::{cell::RefCell, collections::HashMap, error::Error, mem::replace, panic, rc::Rc};

use async_recursion::async_recursion;
use bindings::{io::iter::LoopRequest, w};
use edit::{
    zipper::{
        self,
        analysis::{self, AnalysisError, AnalysisTerm},
        Cursor, TermData,
    },
    Scratchpad, UiSection,
};
use evaluator::{Evaluator, Substitution};
use futures::{
    channel::{
        mpsc::{channel, Receiver, Sender},
        oneshot,
    },
    stream, Stream, StreamExt,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsCast, JsValue,
};
use wasm_bindgen_futures::spawn_local;
use web_sys::{Element, KeyboardEvent, MessageEvent, Node, Worker};
use welkin::Terms;
use welkin_binding::{FromAnalogue, FromWelkin, ToWelkin};
use welkin_core::term::{DefinitionResult, MapCache, Term, TypedDefinitions};

use crate::{
    edit::{add_ui, mutations::HoleMutation, UiSectionVariance},
    worker::WorkerWrapper,
};

mod bindings;
mod edit;
mod evaluator;
mod worker;

thread_local! {
    pub static CACHE: RefCell<MapCache> = RefCell::new(MapCache::new());
}

#[wasm_bindgen]
pub fn entry(terms: Vec<u8>, worker: Worker) -> Result<(), JsValue> {
    spawn_local(async move {
        main(terms, worker).await.unwrap();
    });
    Ok(())
}

#[wasm_bindgen]
pub fn worker(event: MessageEvent) -> Result<(), JsValue> {
    worker::worker(event)
}

#[derive(Debug)]
enum Block {
    Info {
        header: String,
        content: String,
    },
    Printed {
        data: String,
    },
    Error {
        data: AnalysisError<Option<UiSection>>,
    },
    Term {
        prefix: String,
        data: AnalysisTerm<()>,
    },
}

async fn main(terms: Vec<u8>, worker: Worker) -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let worker = WorkerWrapper::new(worker);

    let terms: Terms = bincode::deserialize(&terms).unwrap();

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

    let terms = Rc::new(terms);

    let defs = Rc::new(RefCell::new(HashMap::new()));

    worker
        .initialize(&DefWrapper(defs.clone(), terms.clone()))
        .await;

    for term in data {
        let pad = add_scratchpad(
            term.into(),
            pads.clone(),
            defs.clone(),
            terms.clone(),
            worker.clone(),
        )
        .await?;

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

fn push_paragraph(data: Block, container: &Element) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
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
        Block::Error { data } => {
            let wrapper = document.create_element("div").unwrap();
            wrapper.class_list().add_2("printed", "wrapper").unwrap();
            let paragraph = document.create_element("p").unwrap();
            paragraph
                .class_list()
                .add_3("printed", "error", "content")
                .unwrap();

            match data {
                AnalysisError::TypeError {
                    expected,
                    got,
                    annotation,
                } if annotation.is_some() => {
                    let annotation = annotation.unwrap();
                    let expected = if expected.is_complete() {
                        format!("{:?}", welkin_core::term::Term::from(expected))
                    } else {
                        format!("{:?}", expected)
                    };
                    let got = if got.is_complete() {
                        format!("{:?}", welkin_core::term::Term::from(got))
                    } else {
                        format!("{:?}", got)
                    };
                    paragraph.set_text_content(Some(&format!(
                        "type error:\nexpected\n\t{}\ngot\n\t{}",
                        expected, got
                    )));
                    annotation.show_error();
                }
                AnalysisError::ErasureMismatch {
                    lambda,
                    ty,
                    annotation,
                } if annotation.is_some() => {
                    let annotation = annotation.unwrap();
                    paragraph.set_text_content(Some("erasure mismatch"));
                    annotation.show_error();
                }
                AnalysisError::UnboundReference { annotation, .. } if annotation.is_some() => {
                    let annotation = annotation.unwrap();
                    paragraph.set_text_content(Some("unbound reference"));
                    annotation.show_error();
                }
                _ => {
                    paragraph.set_text_content(Some(&format!("{:?}", data)));
                }
            }
            wrapper.append_child(&paragraph).unwrap();
            container.append_child(&wrapper).unwrap();
        }
        Block::Term { data, prefix } => {
            let wrapper = document.create_element("div").unwrap();
            wrapper.class_list().add_2("printed", "wrapper").unwrap();
            let paragraph = document.create_element("p").unwrap();
            paragraph
                .class_list()
                .add_3("printed", "inference", "content")
                .unwrap();
            paragraph.set_attribute("data-prefix", &prefix).unwrap();
            if data.is_complete() {
                paragraph
                    .set_text_content(Some(&format!("{:?}", welkin_core::term::Term::from(data))));
            } else {
                paragraph.set_text_content(Some(&format!("{:?}", data)));
            }
            wrapper.append_child(&paragraph).unwrap();
            container.append_child(&wrapper).unwrap();
        }
    }
}

async fn add_scratchpad(
    term: zipper::Term,
    pads: Rc<RefCell<Vec<ScratchpadContainer>>>,
    defs: Rc<RefCell<HashMap<String, (Term<String>, Term<String>)>>>,
    terms: Rc<Terms>,
    worker: WorkerWrapper,
) -> Result<ScratchpadContainer, JsValue> {
    let call: Rc<RefCell<Box<dyn FnMut(JsValue)>>> = Rc::new(RefCell::new(Box::new(|_| {})));

    let (sender, mut pad) = make_scratchpad(
        term,
        {
            let call = call.clone();
            move |e| {
                (&mut *call.borrow_mut())(e);
            }
        },
        {
            let pads = pads.clone();
            move |_| {
                save(pads.clone());
            }
        },
    )
    .await?;

    let output = pad.output.clone();

    let receiver = pad.receiver.take().unwrap();

    let wrapper = pad.wrapper.clone();

    *call.borrow_mut() = Box::new({
        let wrapper = wrapper.clone();
        let pads = pads.clone();
        let terms = terms.clone();
        let data = pad.data.clone();
        let defs = defs.clone();
        let worker = worker.clone();
        move |e| {
            let e: KeyboardEvent = e.dyn_into().unwrap();
            let code = e.code();
            match code.as_str() {
                "Enter" if e.shift_key() => {
                    e.prevent_default();
                    let wrapper = wrapper.clone();
                    let pads = pads.clone();
                    let terms = terms.clone();
                    let defs = defs.clone();
                    let worker = worker.clone();
                    spawn_local(async move {
                        let pad = add_scratchpad(
                            zipper::Term::Hole(()),
                            pads.clone(),
                            defs.clone(),
                            terms.clone(),
                            worker.clone(),
                        )
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
                "Period" => {
                    let defs = DefWrapper(defs.clone(), terms.clone());

                    CACHE.with(|cache| {
                        let ty = data.borrow().infer(
                            AnalysisTerm::Reference("Whelk".into(), ()),
                            &defs,
                            &mut *cache.borrow_mut(),
                        );

                        let mut applications = vec![];

                        if let Some(ty) = ty {
                            let mut ty = ty;

                            while let AnalysisTerm::Function {
                                return_type,
                                erased,
                                ..
                            } = ty
                            {
                                applications.push(erased);
                                ty = *return_type;
                            }
                        }

                        if applications.len() == 0 {
                            return;
                        }

                        let term: zipper::Term<_> = data.borrow().clone().into();
                        let mut term = term.clear_annotation();

                        for application in applications {
                            term = zipper::Term::Application {
                                erased: application,
                                annotation: (),
                                argument: Box::new(zipper::Term::Hole(())),
                                function: Box::new(term),
                            };
                        }

                        data.borrow().annotation().trigger_remove(&sender);

                        spawn_local({
                            let data = data.clone();
                            let mut sender = sender.clone();
                            async move {
                                if let Cursor::Hole(cursor) = &*data.borrow() {
                                    let annotation = cursor.annotation();
                                    if let UiSection {
                                        variant: UiSectionVariance::Hole { mutations, p, .. },
                                        ..
                                    } = annotation
                                    {
                                        p.remove();
                                        mutations
                                            .borrow_mut()
                                            .push(HoleMutation::Replace(add_ui(term, &sender)));
                                        let _ = sender.try_send(());
                                    }
                                }
                            }
                        });
                    });

                    e.prevent_default();
                    e.stop_propagation();
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
        let terms = terms.clone();
        let worker = worker.clone();
        async move {
            let mut receiver = Box::pin(stream::once(async { () }).chain(receiver));

            while let Some(()) = receiver.next().await {
                output.set_inner_html("");

                status.remove_attribute("class").unwrap();

                status
                    .class_list()
                    .add_3("scratchpad", "status", "pending")
                    .unwrap();

                let d = data.clone();

                let mut data = d.borrow().clone();
                while !data.is_top() {
                    data = data.ascend();
                }

                let term: AnalysisTerm<Option<UiSection>> = data.clone().into();
                let conv_term = data.into_term();

                {
                    let (sender, receiver) = oneshot::channel();

                    CACHE.with(|cache| {
                        let defs = DefWrapper(defs.clone(), terms.clone());
                        let cache = &mut *cache.borrow_mut();
                        let data = d;
                        let wrapper = wrapper.clone();
                        let output = output.clone();
                        let status = status.clone();
                        let worker = worker.clone();

                        spawn_local(async move {
                            let complete = term.is_complete();
                            let check = worker
                                .check(
                                    term,
                                    AnalysisTerm::Reference("Whelk".into(), None),
                                    |annotation, ty| {
                                        let annotation = &annotation.annotation;
                                        *annotation.borrow_mut() =
                                            Some(ty.clone().clear_annotation());
                                    },
                                    |annotation, ty| {
                                        if let UiSectionVariance::Hole {
                                            filled, mutations, ..
                                        } = &annotation.variant
                                        {
                                            *filled.borrow_mut() =
                                                Some(ty.clone().clear_annotation());
                                        }
                                    },
                                )
                                .await;
                            status.remove_attribute("class").unwrap();

                            let nodes = wrapper.query_selector_all(".error-span").unwrap();
                            for node in 0..nodes.length() {
                                let node = nodes.get(node).unwrap();
                                node.dyn_ref::<Element>()
                                    .unwrap()
                                    .class_list()
                                    .remove_1("error-span")
                                    .unwrap();
                            }
                            if let Cursor::Hole(cursor) = &*data.borrow() {
                                let annotation = cursor.annotation();
                                let mut f = false;
                                if let UiSectionVariance::Hole { filled, .. } = &annotation.variant
                                {
                                    if let Some(term) = &*filled.borrow() {
                                        f = true;
                                        push_paragraph(
                                            Block::Term {
                                                prefix: "filled".into(),
                                                data: term.clone(),
                                            },
                                            &output,
                                        );
                                    }
                                }
                                if !f {
                                    let annotation = &annotation.annotation;

                                    if let Some(ty) = &*annotation.borrow() {
                                        push_paragraph(
                                            Block::Term {
                                                prefix: "goal".into(),
                                                data: ty.clone(),
                                            },
                                            &output,
                                        );
                                    }
                                }
                            }
                            match check {
                                Ok(()) if complete => {
                                    status
                                        .class_list()
                                        .add_3("scratchpad", "status", "def-ok")
                                        .unwrap();

                                    let evaluator = Substitution(defs.clone());

                                    if let Some(term) = conv_term {
                                        let term = evaluator.evaluate(term).unwrap();

                                        let whelk = w::Whelk::from_welkin(term.clone()).unwrap();

                                        let io = match whelk {
                                            w::Whelk::new { data } => match data {
                                                w::BoxPoly::new { data } => data,
                                            },
                                        };

                                        let output = output.clone();

                                        output.set_inner_html("");

                                        run_io(
                                            io,
                                            &|block| push_paragraph(block, &output),
                                            &mut stream::pending(),
                                            &defs,
                                            &evaluator,
                                        )
                                        .await
                                        .unwrap();
                                    }
                                }
                                Err(AnalysisError::Impossible(AnalysisTerm::Hole(_))) => {
                                    status.class_list().add_2("scratchpad", "status").unwrap();
                                }
                                Err(e) => {
                                    output.set_inner_html("");

                                    status
                                        .class_list()
                                        .add_3("scratchpad", "status", "def-err")
                                        .unwrap();

                                    push_paragraph(Block::Error { data: e }, &output);
                                }
                                _ => {
                                    status.class_list().add_2("scratchpad", "status").unwrap();
                                }
                            }
                            sender.send(()).unwrap();
                        });
                    });
                    receiver.await.unwrap();
                }
            }
        }
    });

    Ok(pad)
}

#[derive(Clone)]
pub struct DefWrapper(
    Rc<RefCell<HashMap<String, (Term<String>, Term<String>)>>>,
    Rc<Terms>,
);

#[derive(Serialize, Deserialize)]
pub struct DefWrapperData(HashMap<String, (Term<String>, Term<String>)>, Terms);

impl From<DefWrapper> for DefWrapperData {
    fn from(data: DefWrapper) -> Self {
        DefWrapperData(data.0.borrow().clone(), (&*data.1).clone())
    }
}

impl From<DefWrapperData> for DefWrapper {
    fn from(data: DefWrapperData) -> Self {
        DefWrapper(Rc::new(RefCell::new(data.0)), Rc::new(data.1))
    }
}

impl<T> analysis::TypedDefinitions<Option<T>> for DefWrapper {
    fn get_typed(
        &self,
        name: &str,
    ) -> Option<analysis::DefinitionResult<(AnalysisTerm<Option<T>>, AnalysisTerm<Option<T>>)>>
    {
        TypedDefinitions::get_typed(self, &name.to_owned()).map(|defs| match defs {
            DefinitionResult::Borrowed((ty, term)) => {
                analysis::DefinitionResult::Owned((ty.clone().into(), term.clone().into()))
            }
            DefinitionResult::Owned((ty, term)) => {
                analysis::DefinitionResult::Owned((ty.into(), term.into()))
            }
        })
    }
}

impl TypedDefinitions<String> for DefWrapper {
    fn get_typed(&self, n: &String) -> Option<DefinitionResult<(Term<String>, Term<String>)>> {
        self.0
            .borrow()
            .get(n)
            .map(|(ty, term)| DefinitionResult::Owned((ty.clone(), term.clone())))
            .or_else(|| {
                self.1.data.iter().find_map(|(name, ty, term)| {
                    if &format!("{:?}", name) == n {
                        Some(DefinitionResult::Owned((
                            ty.clone()
                                .map_reference(|a| Term::Reference(format!("{:?}", a))),
                            term.clone()
                                .map_reference(|a| Term::Reference(format!("{:?}", a))),
                        )))
                    } else {
                        None
                    }
                })
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
    output: Element,
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
    mut focus_listener: impl FnMut(JsValue) + 'static,
) -> Result<(Sender<()>, ScratchpadContainer), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let wrapper = document.create_element("div")?;
    wrapper.class_list().add_2("scratchpad", "wrapper")?;

    let inner = document.create_element("div")?;
    inner.class_list().add_2("scratchpad", "inner")?;

    let content = document.create_element("div")?;
    content.class_list().add_2("scratchpad", "content")?;

    let status = document.create_element("div")?;
    status.class_list().add_2("scratchpad", "status")?;

    let output = document.create_element("div")?;
    output.class_list().add_2("scratchpad", "output")?;

    wrapper.append_child(&inner)?;
    inner.append_child(&content)?;
    inner.append_child(&status)?;
    wrapper.append_child(&output)?;

    let (s, mut pad) = Scratchpad::new(term, content.into());

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

    let focusout_listener =
        Closure::wrap(Box::new(move |e: JsValue| focus_listener(e)) as Box<dyn FnMut(JsValue)>);

    wrapper
        .add_event_listener_with_callback("keydown", keydown_listener.as_ref().unchecked_ref())?;
    wrapper
        .add_event_listener_with_callback("focusout", focusout_listener.as_ref().unchecked_ref())?;

    Ok((
        s,
        ScratchpadContainer {
            wrapper,
            data,
            output,
            status,
            _closures: vec![keydown_listener, focusout_listener],
            receiver: Some(receiver),
        },
    ))
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
    defs: &DefWrapper,
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
                    w::WhelkRequest::r#define { .. } => "define",
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
                                run_io(io, &*push_paragraph, &mut *receive, &*defs, &*evaluator)
                                    .await
                            })
                            .await?;
                        if {
                            // TODO allow loops later in background
                            request.proceed(&*evaluator)?;
                            false
                        } {
                            continue;
                        } else {
                            break Ok(FromAnalogue::from_analogue(FromWelkin::from_welkin(
                                request.into_state(),
                            )?));
                        }
                    }
                }
                w::WhelkRequest::prompt { .. } => {
                    // TODO reinstate actual message input
                    // let message = receive.next().await.unwrap();
                    let message: String = "placeholder".into();
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
                    run_io(io, &*push_paragraph, &mut *receive, &*defs, &*evaluator).await
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
                    run_io(io, &*push_paragraph, &mut *receive, &*defs, &*evaluator).await
                }
                w::WhelkRequest::define { name, term, r#type } => {
                    let name: String = match name {
                        w::Sized::new { data, .. } => data.clone().into(),
                    };

                    push_paragraph(Block::Info {
                        header: "DEF".into(),
                        content: format!("{:?}", name.clone()),
                    });

                    let term = term.0.clone();
                    let ty = r#type.0.clone();

                    defs.0.borrow_mut().insert(name, (ty, term));

                    let io = io
                        .into_request()
                        .unwrap()
                        .fulfill(w::Unit::new.to_welkin().unwrap(), &*evaluator)?;
                    run_io(io, &*push_paragraph, &mut *receive, &*defs, &*evaluator).await
                }
            }
        }
    }
}
