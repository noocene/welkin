use std::{cell::RefCell, collections::HashMap, panic, rc::Rc};

use futures::channel::oneshot::{channel, Sender};
use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent, Worker};
use welkin_core::term::MapCache;

use crate::{
    edit::zipper::analysis::{AnalysisError, AnalysisTerm},
    DefWrapper, DefWrapperData,
};

thread_local! {
    pub static DEFS: RefCell<Option<DefWrapper>> = RefCell::new(None);
    pub static CACHE: RefCell<MapCache> = RefCell::new(MapCache::new());
    pub static INITIALIZED: RefCell<bool> = RefCell::new(false);
}

#[derive(Clone)]
pub struct WorkerWrapper {
    worker: Worker,
    channels: Rc<RefCell<HashMap<Uuid, Sender<WorkerResponse>>>>,
    on_message: Rc<Closure<dyn FnMut(JsValue)>>,
}

impl WorkerWrapper {
    pub fn new(worker: Worker) -> Self {
        let channels: Rc<RefCell<HashMap<Uuid, Sender<WorkerResponse>>>> =
            Rc::new(RefCell::new(HashMap::new()));

        let on_message = Closure::wrap(Box::new({
            let channels = channels.clone();
            move |e: JsValue| {
                let data: MessageEvent = e.dyn_into().unwrap();
                let data: Uint8Array = data.data().dyn_into().unwrap();
                let data = data.to_vec();
                let data: WorkerResponse = bincode::deserialize(data.as_slice()).unwrap();

                let sender = channels.borrow_mut().remove(&data.idx).unwrap();
                sender.send(data).unwrap();
            }
        }) as Box<dyn FnMut(JsValue)>);

        worker.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

        WorkerWrapper {
            worker,
            channels,
            on_message: Rc::new(on_message),
        }
    }

    async fn make_request(&self, variant: WorkerRequestVariant) -> WorkerResponse {
        let (sender, receiver) = channel();
        let uuid = Uuid::new_v4();
        self.channels.borrow_mut().insert(uuid.clone(), sender);

        let request = WorkerRequest { idx: uuid, variant };
        let request = bincode::serialize(&request).unwrap();
        self.worker
            .post_message(&Uint8Array::from(request.as_slice()).into())
            .unwrap();

        receiver.await.unwrap()
    }

    pub async fn check<T: Clone>(
        &self,
        term: AnalysisTerm<Option<T>>,
        ty: AnalysisTerm<Option<T>>,
        mut annotate: impl FnMut(T, AnalysisTerm<Option<T>>),
        mut fill_hole: impl FnMut(T, AnalysisTerm<Option<T>>),
    ) -> Result<(), AnalysisError<Option<T>>> {
        let mut annotations = HashMap::new();
        let resp = self
            .make_request(WorkerRequestVariant::Check(
                term.map_annotation(&mut |annotation| {
                    annotation.map(|annotation| {
                        let idx = annotations.len();
                        annotations.insert(idx, annotation);
                        idx as _
                    })
                }),
                ty.map_annotation(&mut |annotation| {
                    annotation.map(|annotation| {
                        let idx = annotations.len();
                        annotations.insert(idx, annotation);
                        idx as _
                    })
                }),
            ))
            .await;

        for (idx, ty) in resp.inferred {
            annotate(
                annotations.get(&(idx as _)).unwrap().clone(),
                ty.map_annotation(&mut |annotation| {
                    annotation.and_then(|idx| annotations.get(&(idx as _)).cloned())
                }),
            )
        }

        for (idx, ty) in resp.filled {
            fill_hole(
                annotations.get(&(idx as _)).unwrap().clone(),
                ty.map_annotation(&mut |annotation| {
                    annotation.and_then(|idx| annotations.get(&(idx as _)).cloned())
                }),
            )
        }

        resp.data.map_err(|e| {
            e.map_annotation(&mut |annotation| {
                annotation.and_then(|idx| annotations.remove(&(idx as _)))
            })
        })
    }

    pub async fn initialize(&self, defs: &DefWrapper) {
        self.make_request(WorkerRequestVariant::Initialize(defs.clone().into()))
            .await
            .data
            .unwrap();
    }
}

#[derive(Serialize, Deserialize)]
pub enum WorkerRequestVariant {
    Check(AnalysisTerm<Option<u64>>, AnalysisTerm<Option<u64>>),
    Initialize(DefWrapperData),
}

#[derive(Serialize, Deserialize)]
pub struct WorkerRequest {
    variant: WorkerRequestVariant,
    idx: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkerResponse {
    idx: Uuid,
    inferred: Vec<(u64, AnalysisTerm<Option<u64>>)>,
    filled: Vec<(u64, AnalysisTerm<Option<u64>>)>,
    data: Result<(), AnalysisError<Option<u64>>>,
}

pub fn worker(event: MessageEvent) -> Result<(), JsValue> {
    INITIALIZED.with(|initialized| {
        let initialized = &mut *initialized.borrow_mut();
        if !*initialized {
            panic::set_hook(Box::new(console_error_panic_hook::hook));
            *initialized = true;
        }
    });

    let data = event.data().dyn_into::<Uint8Array>().unwrap().to_vec();
    let data: WorkerRequest = bincode::deserialize(&data).unwrap();
    let mut inferred = vec![];
    let mut filled = vec![];
    let worker = js_sys::global()
        .dyn_into::<DedicatedWorkerGlobalScope>()
        .unwrap();

    let response = match data.variant {
        WorkerRequestVariant::Check(term, ty) => DEFS.with(|defs| {
            CACHE.with(|cache| {
                let defs = defs.borrow();
                let defs = defs.as_ref().unwrap();
                let cache = &mut *cache.borrow_mut();
                term.check_in(
                    &ty,
                    defs,
                    &mut |annotation, ty| {
                        if let Some(annotation) = annotation {
                            inferred.push((*annotation, ty.clone()))
                        }
                    },
                    &mut |annotation, ty| {
                        if let Some(annotation) = annotation {
                            filled.push((*annotation, ty.clone()))
                        }
                    },
                    cache,
                )
            })
        }),
        WorkerRequestVariant::Initialize(data) => {
            DEFS.with(|defs| {
                *defs.borrow_mut() = Some(data.into());
            });
            Ok(())
        }
    };

    let response = WorkerResponse {
        data: response,
        idx: data.idx,
        inferred,
        filled,
    };

    let data = bincode::serialize(&response).unwrap();

    worker
        .post_message(&Uint8Array::from(data.as_slice()).into())
        .unwrap();

    Ok(())
}
