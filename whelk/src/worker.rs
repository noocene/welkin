use std::{cell::RefCell, collections::HashMap, panic, rc::Rc};

use futures::channel::oneshot::{channel, Sender};
use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent, Worker};
use welkin_core::term::{MapCache, Term};

use crate::{
    edit::{
        dynamic::abst::controls::Zero,
        zipper::analysis::{
            AnalysisError, AnalysisTerm, DefinitionResult, StratificationError, TypedDefinitions,
        },
    },
    evaluator::{CoreEvaluator, Inet},
    DefWrapper, DefWrapperData,
};

thread_local! {
    pub static DEFS: RefCell<Option<DefWrapper>> = RefCell::new(None);
    pub static CACHE: RefCell<MapCache> = RefCell::new(MapCache::new());
    pub static INITIALIZED: RefCell<bool> = RefCell::new(false);
    pub static TEMP_DEFS: RefCell<HashMap<Uuid, Vec<(String, Term<String>, Term<String>)>>> = RefCell::new(HashMap::new());
}

#[derive(Clone)]
pub struct WorkerWrapper {
    worker: Worker,
    channels: Rc<RefCell<HashMap<Uuid, Sender<WorkerResponse>>>>,
    on_message: Rc<Closure<dyn FnMut(JsValue)>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CheckError<T> {
    Analysis(AnalysisError<T>),
    Stratification(StratificationError),
    Recursive,
}

impl<T> CheckError<T> {
    pub fn map_annotation<U, F: FnMut(T) -> U>(self, call: &mut F) -> CheckError<U> {
        match self {
            CheckError::Analysis(e) => CheckError::Analysis(e.map_annotation(call)),
            CheckError::Stratification(e) => CheckError::Stratification(e),
            CheckError::Recursive => CheckError::Recursive,
        }
    }
}

impl<T> From<AnalysisError<T>> for CheckError<T> {
    fn from(e: AnalysisError<T>) -> Self {
        CheckError::Analysis(e)
    }
}

impl<T> From<StratificationError> for CheckError<T> {
    fn from(e: StratificationError) -> Self {
        CheckError::Stratification(e)
    }
}

pub struct TempDefs(Uuid);

pub struct MergeDefs<'a>(
    &'a DefWrapper,
    &'a Option<Vec<(String, Term<String>, Term<String>)>>,
);

impl<'a, T: Zero> TypedDefinitions<T> for MergeDefs<'a> {
    fn get_typed(
        &self,
        name: &str,
    ) -> Option<DefinitionResult<(AnalysisTerm<T>, AnalysisTerm<T>)>> {
        self.1
            .as_ref()
            .and_then(|defs| defs.iter().find(|(n, _, _)| n.as_str() == name).cloned())
            .map(|(_, ty, term)| DefinitionResult::Owned((ty.into(), term.into())))
            .or_else(|| self.0.get_typed(name))
    }
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
        temp_defs: Option<&TempDefs>,
    ) -> Result<(), CheckError<Option<T>>> {
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
                temp_defs.map(|a| a.0.clone()),
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

    pub async fn register(&self, name: String, ty: Term<String>, term: Term<String>) {
        self.make_request(WorkerRequestVariant::Register(name, ty, term))
            .await
            .data
            .unwrap()
    }

    pub async fn evaluate(&self, term: Term<String>) -> Term<String> {
        let data = self
            .make_request(WorkerRequestVariant::Evaluate(term))
            .await;
        data.data.unwrap();
        data.evaluated.unwrap()
    }

    pub async fn expand_evaluate(&self, term: AnalysisTerm<()>) -> Term<String> {
        let data = self
            .make_request(WorkerRequestVariant::ExpandEvaluate(term))
            .await;
        data.data.unwrap();
        data.evaluated.unwrap()
    }

    pub async fn register_temp(&self, defs: Vec<(String, Term<String>, Term<String>)>) -> TempDefs {
        let data = self
            .make_request(WorkerRequestVariant::TempDefs(defs))
            .await;
        data.data.unwrap();
        TempDefs(data.id.unwrap())
    }

    pub async fn clear_temp(&self, defs: TempDefs) {
        self.make_request(WorkerRequestVariant::ClearTempDefs(defs.0))
            .await
            .data
            .unwrap();
    }
}

#[derive(Serialize, Deserialize)]
pub enum WorkerRequestVariant {
    Check(
        AnalysisTerm<Option<u64>>,
        AnalysisTerm<Option<u64>>,
        Option<Uuid>,
    ),
    Register(String, Term<String>, Term<String>),
    Initialize(DefWrapperData),
    Evaluate(Term<String>),
    ExpandEvaluate(AnalysisTerm<()>),
    TempDefs(Vec<(String, Term<String>, Term<String>)>),
    ClearTempDefs(Uuid),
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
    evaluated: Option<Term<String>>,
    data: Result<(), CheckError<Option<u64>>>,
    id: Option<Uuid>,
}

pub async fn worker(event: MessageEvent) -> Result<(), JsValue> {
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

    let mut evaluated = None;
    let mut id = None;

    let response = match data.variant {
        WorkerRequestVariant::Check(term, ty, temp_defs) => {
            let temp_id = temp_defs.clone();
            let temp_defs =
                temp_defs.map(|id| TEMP_DEFS.with(|defs| defs.borrow_mut().remove(&id).unwrap()));

            let res = DEFS.with(|defs| {
                CACHE.with(|cache| {
                    let defs = defs.borrow();
                    let defs = defs.as_ref().unwrap();
                    let cache = &mut *cache.borrow_mut();
                    let defs = MergeDefs(defs, &temp_defs);
                    term.check_in(
                        &ty,
                        &defs,
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
                    )?;
                    term.is_stratified()?;
                    if term.is_recursive_in(&defs) {
                        Err(CheckError::Recursive)
                    } else {
                        Ok(())
                    }
                })
            });

            if let Some(temp_id) = temp_id {
                TEMP_DEFS.with(move |defs| defs.borrow_mut().insert(temp_id, temp_defs.unwrap()));
            }

            res
        }
        WorkerRequestVariant::Initialize(data) => {
            DEFS.with(|defs| {
                *defs.borrow_mut() = Some(data.into());
            });
            Ok(())
        }
        WorkerRequestVariant::Register(name, ty, term) => {
            DEFS.with(|defs| {
                defs.borrow()
                    .as_ref()
                    .unwrap()
                    .0
                    .borrow_mut()
                    .insert(name, (ty, term));
            });
            Ok(())
        }
        WorkerRequestVariant::Evaluate(term) => {
            let e = DEFS.with(|defs| {
                let defs = defs.borrow();
                let defs = defs.as_ref().unwrap();
                let evaluator = Inet(defs.clone());
                evaluator.evaluate(term)
            });
            evaluated = Some(e.await.unwrap());
            Ok(())
        }
        WorkerRequestVariant::ExpandEvaluate(term) => {
            let term: Term<String> = term.into();
            let e = DEFS.with(|defs| {
                let defs = defs.borrow();
                let defs = defs.as_ref().unwrap();
                let evaluator = Inet(defs.clone());
                evaluator.evaluate(term)
            });
            evaluated = Some(e.await.unwrap());
            Ok(())
        }
        WorkerRequestVariant::TempDefs(temp_defs) => {
            let uuid = Uuid::new_v4();
            id = Some(uuid.clone());
            TEMP_DEFS.with(|defs| {
                defs.borrow_mut().insert(uuid, temp_defs);
            });
            Ok(())
        }
        WorkerRequestVariant::ClearTempDefs(uuid) => {
            TEMP_DEFS.with(|defs| {
                defs.borrow_mut().remove(&uuid);
            });
            Ok(())
        }
    };

    let response = WorkerResponse {
        data: response,
        idx: data.idx,
        evaluated,
        inferred,
        filled,
        id,
    };

    let data = bincode::serialize(&response).unwrap();

    worker
        .post_message(&Uint8Array::from(data.as_slice()).into())
        .unwrap();

    Ok(())
}
