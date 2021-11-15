use core_futures_io::FuturesCompat;
use downcast_rs::{impl_downcast, Downcast};
use futures::{task::noop_waker, Future};
use mincodec::{
    AsyncReader, AsyncReaderError, AsyncWriter, AsyncWriterError, MinCodec, MinCodecRead,
    MinCodecWrite,
};
use std::{
    any::TypeId,
    cell::RefCell,
    collections::hash_map::DefaultHasher,
    convert::TryInto,
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    rc::Rc,
};
pub mod analysis;
pub mod dynamic;
use serde::{Deserialize, Serialize};
use welkin_core::term::{self, Index};

use self::dynamic::{Dynamic, DynamicTerm};

use super::dynamic::abst::controls::{
    CompressedChar, CompressedSize, CompressedString, CompressedWord, Zero,
};

#[derive(Debug, Clone, MinCodec, Serialize, Deserialize)]
#[bounds()]
pub enum TermData {
    Lambda {
        erased: bool,
        name: Option<String>,
        body: Box<TermData>,
    },
    Application {
        erased: bool,
        function: Box<TermData>,
        argument: Box<TermData>,
    },
    Put(Box<TermData>),
    Duplication {
        binder: Option<String>,
        expression: Box<TermData>,
        body: Box<TermData>,
    },
    Reference(String),

    Universe,
    Function {
        erased: bool,
        name: Option<String>,
        self_name: Option<String>,
        argument_type: Box<TermData>,
        return_type: Box<TermData>,
    },
    Wrap(Box<TermData>),

    Hole,

    Dynamic(Dynamic<()>),
}

impl From<Term<()>> for TermData {
    fn from(term: Term<()>) -> Self {
        match term {
            Term::Lambda {
                erased, name, body, ..
            } => TermData::Lambda {
                erased,
                name,
                body: Box::new((*body).into()),
            },
            Term::Application {
                erased,
                function,
                argument,
                ..
            } => TermData::Application {
                erased,
                function: Box::new((*function).into()),
                argument: Box::new((*argument).into()),
            },
            Term::Put(term, _) => TermData::Put(Box::new((*term).into())),
            Term::Duplication {
                binder,
                expression,
                body,
                ..
            } => TermData::Duplication {
                binder,
                expression: Box::new((*expression).into()),
                body: Box::new((*body).into()),
            },
            Term::Reference(name, _) => TermData::Reference(name),
            Term::Universe(_) => TermData::Universe,
            Term::Function {
                erased,
                name,
                self_name,
                argument_type,
                return_type,
                ..
            } => TermData::Function {
                erased,
                name,
                self_name,
                argument_type: Box::new((*argument_type).into()),
                return_type: Box::new((*return_type).into()),
            },
            Term::Wrap(term, _) => TermData::Wrap(Box::new((*term).into())),
            Term::Hole(_) => TermData::Hole,
            Term::Dynamic(data) => TermData::Dynamic(data),

            Term::Compressed(_) => todo!(),
        }
    }
}

impl From<TermData> for Term<()> {
    fn from(term: TermData) -> Self {
        match term {
            TermData::Lambda { erased, name, body } => Term::Lambda {
                erased,
                name,
                body: Box::new((*body).into()),
                annotation: (),
            },
            TermData::Application {
                erased,
                function,
                argument,
            } => Term::Application {
                erased,
                function: Box::new((*function).into()),
                argument: Box::new((*argument).into()),
                annotation: (),
            },
            TermData::Put(term) => Term::Put(Box::new((*term).into()), ()),
            TermData::Duplication {
                binder,
                expression,
                body,
            } => Term::Duplication {
                binder,
                expression: Box::new((*expression).into()),
                body: Box::new((*body).into()),
                annotation: (),
            },
            TermData::Reference(name) => Term::Reference(name, ()),
            TermData::Universe => Term::Universe(()),
            TermData::Function {
                erased,
                name,
                self_name,
                argument_type,
                return_type,
            } => Term::Function {
                erased,
                name,
                self_name,
                argument_type: Box::new((*argument_type).into()),
                return_type: Box::new((*return_type).into()),
                annotation: (),
            },
            TermData::Wrap(term) => Term::Wrap(Box::new((*term).into()), ()),

            TermData::Hole => Term::Hole(()),

            TermData::Dynamic(term) => Term::Dynamic(term),
        }
    }
}

impl<T: Clone> From<Term<T, RefCount>> for Term<T, System> {
    fn from(term: Term<T, RefCount>) -> Self {
        match term {
            Term::Lambda {
                erased,
                name,
                body,
                annotation,
            } => Term::Lambda {
                erased,
                name,
                body: Box::new(body.borrow().clone().into()),
                annotation,
            },
            Term::Application {
                erased,
                function,
                argument,
                annotation,
            } => Term::Application {
                erased,
                function: Box::new(function.borrow().clone().into()),
                argument: Box::new(argument.borrow().clone().into()),
                annotation,
            },
            Term::Put(term, annotation) => {
                Term::Put(Box::new(term.borrow().clone().into()), annotation)
            }
            Term::Duplication {
                binder,
                expression,
                body,
                annotation,
            } => Term::Duplication {
                binder,
                expression: Box::new(expression.borrow().clone().into()),
                body: Box::new(body.borrow().clone().into()),
                annotation,
            },
            Term::Reference(name, annotation) => Term::Reference(name, annotation),
            Term::Universe(annotation) => Term::Universe(annotation),
            Term::Function {
                erased,
                name,
                self_name,
                argument_type,
                return_type,
                annotation,
            } => Term::Function {
                erased,
                name,
                self_name,
                argument_type: Box::new(argument_type.borrow().clone().into()),
                return_type: Box::new(return_type.borrow().clone().into()),
                annotation,
            },
            Term::Wrap(term, annotation) => {
                Term::Wrap(Box::new(term.borrow().clone().into()), annotation)
            }
            Term::Hole(annotation) => Term::Hole(annotation),

            Term::Dynamic(dynamic) => Term::Dynamic(dynamic),

            Term::Compressed(_) => todo!(),
        }
    }
}

impl<T> From<Term<T>> for Term<T, RefCount> {
    fn from(term: Term<T>) -> Self {
        match term {
            Term::Lambda {
                erased,
                name,
                body,
                annotation,
            } => Term::Lambda {
                erased,
                name,
                body: Rc::new(RefCell::new((*body).into())),
                annotation,
            },
            Term::Application {
                erased,
                function,
                argument,
                annotation,
            } => Term::Application {
                erased,
                function: Rc::new(RefCell::new((*function).into())),
                argument: Rc::new(RefCell::new((*argument).into())),
                annotation,
            },
            Term::Put(term, annotation) => {
                Term::Put(Rc::new(RefCell::new((*term).into())), annotation)
            }
            Term::Duplication {
                binder,
                expression,
                body,
                annotation,
            } => Term::Duplication {
                binder,
                expression: Rc::new(RefCell::new((*expression).into())),
                body: Rc::new(RefCell::new((*body).into())),
                annotation,
            },
            Term::Reference(name, annotation) => Term::Reference(name, annotation),

            Term::Universe(annotation) => Term::Universe(annotation),
            Term::Function {
                erased,
                name,
                self_name,
                argument_type,
                return_type,
                annotation,
            } => Term::Function {
                erased,
                name,
                self_name,
                argument_type: Rc::new(RefCell::new((*argument_type).into())),
                return_type: Rc::new(RefCell::new((*return_type).into())),
                annotation,
            },
            Term::Wrap(term, annotation) => {
                Term::Wrap(Rc::new(RefCell::new((*term).into())), annotation)
            }

            Term::Hole(annotation) => Term::Hole(annotation),

            Term::Dynamic(dynamic) => Term::Dynamic(dynamic),

            Term::Compressed(_) => todo!(),
        }
    }
}

pub trait Allocator<T> {
    type Box;

    fn clone(data: &Self::Box) -> Self::Box
    where
        T: Clone;

    fn debug(data: &Self::Box, f: &mut fmt::Formatter) -> fmt::Result
    where
        T: Debug;
}

pub trait AClone<T> {
    fn clone(&self) -> Self;
}

#[derive(Debug, Clone)]
pub struct System;

impl<T> Allocator<T> for System {
    type Box = Box<Term<T, System>>;

    fn clone(data: &Self::Box) -> Self::Box
    where
        T: Clone,
    {
        data.clone()
    }

    fn debug(data: &Self::Box, f: &mut fmt::Formatter) -> fmt::Result
    where
        T: Debug,
    {
        <Self::Box as Debug>::fmt(data, f)
    }
}

#[derive(Debug, Clone)]
pub struct RefCount;

impl<T> Allocator<T> for RefCount {
    type Box = Rc<RefCell<Term<T, RefCount>>>;

    fn clone(data: &Self::Box) -> Self::Box
    where
        T: Clone,
    {
        data.clone()
    }

    fn debug(data: &Self::Box, f: &mut fmt::Formatter) -> fmt::Result
    where
        T: Debug,
    {
        <Term<T, RefCount> as Debug>::fmt(&*data.borrow(), f)
    }
}

struct DebugWrapper<'a, T: Debug, A: Allocator<T>>(&'a A::Box);

impl<'a, T: Debug, A: Allocator<T>> Debug for DebugWrapper<'a, T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        A::debug(self.0, f)
    }
}

pub enum Term<T = (), A: Allocator<T> = System> {
    Lambda {
        erased: bool,
        name: Option<String>,
        body: A::Box,
        annotation: T,
    },
    Application {
        erased: bool,
        function: A::Box,
        argument: A::Box,
        annotation: T,
    },
    Put(A::Box, T),
    Duplication {
        binder: Option<String>,
        expression: A::Box,
        body: A::Box,
        annotation: T,
    },
    Reference(String, T),

    Universe(T),
    Function {
        erased: bool,
        name: Option<String>,
        self_name: Option<String>,
        argument_type: A::Box,
        return_type: A::Box,
        annotation: T,
    },
    Wrap(A::Box, T),

    Hole(T),

    Dynamic(Dynamic<T>),

    Compressed(Box<dyn CompressedTerm<T>>),
}

impl<T: Debug, A: Allocator<T>> Term<T, A> {
    fn write(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Term::*;

        match &self {
            Lambda {
                body, name, erased, ..
            } => write!(
                f,
                "{}{} {:?}",
                if *erased { "/" } else { "\\" },
                name.as_ref().map(String::as_str).unwrap_or(""),
                DebugWrapper::<T, A>(body)
            ),
            Application {
                function,
                argument,
                erased,
                ..
            } => write!(
                f,
                "{}{:?} {:?}{}",
                if *erased { "[" } else { "(" },
                DebugWrapper::<T, A>(function),
                DebugWrapper::<T, A>(argument),
                if *erased { "]" } else { ")" }
            ),
            Put(term, _) => write!(f, ". {:?}", DebugWrapper::<T, A>(term)),
            Reference(name, _) => name.fmt(f),
            Duplication {
                expression,
                body,
                binder,
                ..
            } => write!(
                f,
                ": {} = {:?} {:?}",
                binder.as_ref().map(String::as_str).unwrap_or(""),
                DebugWrapper::<T, A>(expression),
                DebugWrapper::<T, A>(body)
            ),

            Universe(_) => write!(f, "*"),
            Wrap(term, _) => write!(f, "!{:?}", DebugWrapper::<T, A>(term)),
            Function {
                argument_type,
                return_type,
                erased,
                name,
                self_name,
                ..
            } => write!(
                f,
                "{}{},{}:{:?} {:?}",
                if *erased { "_" } else { "+" },
                self_name.as_ref().map(String::as_str).unwrap_or(""),
                name.as_ref().map(String::as_str).unwrap_or(""),
                DebugWrapper::<T, A>(argument_type),
                DebugWrapper::<T, A>(return_type)
            ),

            Hole(_) => write!(f, "?"),
            Dynamic(_) => write!(f, "DYNAMIC"),
            Compressed(_) => write!(f, "COMPRESSED"),
        }
    }
}

impl<T: Debug, A: Allocator<T>> Debug for Term<T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write(f)
    }
}

pub trait CompressedTerm<T>: Downcast {
    fn expand(&self) -> Term<T>;
    fn box_clone(&self) -> Box<dyn CompressedTerm<T>>;
    fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result;
    fn to_vec(&self) -> Vec<u8>;
    fn concrete_ty(&self) -> Option<Term<T>>;
    fn annotation(&self) -> T;
    fn hash(&self) -> u64;
}

impl_downcast!(CompressedTerm<T>);

impl<T> Serialize for Box<dyn CompressedTerm<T>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Serialize::serialize(&self.to_vec(), serializer)
    }
}

impl<T> Hash for Box<dyn CompressedTerm<T>> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        CompressedTerm::hash(&**self).hash(state)
    }
}

fn ty_hash<T: 'static>() -> u64 {
    let mut hasher = DefaultHasher::new();
    TypeId::of::<T>().hash(&mut hasher);
    hasher.finish()
}

impl<'de, T: Zero + Clone> Deserialize<'de> for Box<dyn CompressedTerm<T>> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let buf = <Vec<u8> as Deserialize<'_>>::deserialize(deserializer)?;

        let data = u64::from_be_bytes(buf[..8].try_into().unwrap());

        if data == ty_hash::<CompressedSize>() {
            Ok(Box::new(
                bincode::deserialize::<CompressedSize>(&buf[8..]).unwrap(),
            ))
        } else if data == ty_hash::<CompressedWord>() {
            Ok(Box::new(
                bincode::deserialize::<CompressedWord>(&buf[8..]).unwrap(),
            ))
        } else if data == ty_hash::<CompressedChar>() {
            Ok(Box::new(
                bincode::deserialize::<CompressedChar>(&buf[8..]).unwrap(),
            ))
        } else if data == ty_hash::<CompressedString>() {
            Ok(Box::new(
                bincode::deserialize::<CompressedString>(&buf[8..]).unwrap(),
            ))
        } else {
            panic!()
        }
    }
}

impl<T> Clone for Box<dyn CompressedTerm<T>> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

impl<T> Debug for Box<dyn CompressedTerm<T>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug(f)
    }
}

impl<T: Clone, A: Allocator<T>> Clone for Term<T, A> {
    fn clone(&self) -> Self {
        match self {
            Self::Lambda {
                erased,
                name,
                body,
                annotation,
            } => Self::Lambda {
                erased: erased.clone(),
                name: name.clone(),
                body: A::clone(body),
                annotation: annotation.clone(),
            },
            Self::Application {
                erased,
                function,
                argument,
                annotation,
            } => Self::Application {
                erased: erased.clone(),
                function: A::clone(function),
                argument: A::clone(argument),
                annotation: annotation.clone(),
            },
            Self::Put(arg0, arg1) => Self::Put(A::clone(arg0), arg1.clone()),
            Self::Duplication {
                binder,
                expression,
                body,
                annotation,
            } => Self::Duplication {
                binder: binder.clone(),
                expression: A::clone(expression),
                body: A::clone(body),
                annotation: annotation.clone(),
            },
            Self::Reference(arg0, arg1) => Self::Reference(arg0.clone(), arg1.clone()),
            Self::Universe(arg0) => Self::Universe(arg0.clone()),
            Self::Function {
                erased,
                name,
                self_name,
                argument_type,
                return_type,
                annotation,
            } => Self::Function {
                erased: erased.clone(),
                name: name.clone(),
                self_name: self_name.clone(),
                argument_type: A::clone(argument_type),
                return_type: A::clone(return_type),
                annotation: annotation.clone(),
            },
            Self::Wrap(arg0, arg1) => Self::Wrap(A::clone(arg0), arg1.clone()),
            Self::Hole(arg0) => Self::Hole(arg0.clone()),
            Self::Dynamic(arg0) => Self::Dynamic(arg0.clone()),
            Self::Compressed(arg0) => Self::Compressed(arg0.clone()),
        }
    }
}

impl<T> Term<T> {
    pub fn clear_annotation(self) -> Term<()> {
        match self {
            Term::Lambda {
                erased, name, body, ..
            } => Term::Lambda {
                erased,
                name,
                body: Box::new(body.clear_annotation()),
                annotation: (),
            },
            Term::Application {
                erased,
                function,
                argument,
                ..
            } => Term::Application {
                erased,
                function: Box::new(function.clear_annotation()),
                argument: Box::new(argument.clear_annotation()),
                annotation: (),
            },
            Term::Put(term, _) => Term::Put(Box::new(term.clear_annotation()), ()),
            Term::Duplication {
                binder,
                expression,
                body,
                ..
            } => Term::Duplication {
                binder,
                expression: Box::new(expression.clear_annotation()),
                body: Box::new(body.clear_annotation()),
                annotation: (),
            },
            Term::Reference(name, _) => Term::Reference(name, ()),

            Term::Universe(_) => Term::Universe(()),
            Term::Function {
                erased,
                name,
                self_name,
                argument_type,
                return_type,
                ..
            } => Term::Function {
                erased,
                name,
                self_name,
                argument_type: Box::new(argument_type.clear_annotation()),
                return_type: Box::new(return_type.clear_annotation()),
                annotation: (),
            },
            Term::Wrap(term, _) => Term::Wrap(Box::new(term.clear_annotation()), ()),

            Term::Hole(_) => Term::Hole(()),

            Term::Dynamic(Dynamic { term, .. }) => Term::Dynamic(Dynamic {
                annotation: (),
                term: term.clear_annotation(),
            }),

            Term::Compressed(_) => todo!(),
        }
    }

    pub fn try_map_annotation<U, E, F: Fn(T) -> Result<U, E> + Clone>(
        self,
        f: F,
    ) -> Result<Term<U>, E> {
        Ok(match self {
            Term::Lambda {
                erased,
                name,
                body,
                annotation,
            } => Term::Lambda {
                erased,
                name,
                body: Box::new(body.try_map_annotation(f.clone())?),
                annotation: f(annotation)?,
            },
            Term::Application {
                erased,
                function,
                argument,
                annotation,
            } => Term::Application {
                erased,
                function: Box::new(function.try_map_annotation(f.clone())?),
                argument: Box::new(argument.try_map_annotation(f.clone())?),
                annotation: f(annotation)?,
            },
            Term::Put(term, annotation) => Term::Put(
                Box::new(term.try_map_annotation(f.clone())?),
                f(annotation)?,
            ),
            Term::Duplication {
                binder,
                expression,
                body,
                annotation,
            } => Term::Duplication {
                binder,
                expression: Box::new(expression.try_map_annotation(f.clone())?),
                body: Box::new(body.try_map_annotation(f.clone())?),
                annotation: f(annotation)?,
            },
            Term::Reference(name, annotation) => Term::Reference(name, f(annotation)?),

            Term::Universe(annotation) => Term::Universe(f(annotation)?),
            Term::Function {
                erased,
                name,
                self_name,
                argument_type,
                return_type,
                annotation,
            } => Term::Function {
                erased,
                name,
                self_name,
                argument_type: Box::new(argument_type.try_map_annotation(f.clone())?),
                return_type: Box::new(return_type.try_map_annotation(f.clone())?),
                annotation: f(annotation)?,
            },
            Term::Wrap(term, annotation) => Term::Wrap(
                Box::new(term.try_map_annotation(f.clone())?),
                f(annotation)?,
            ),

            Term::Hole(annotation) => Term::Hole(f(annotation)?),

            Term::Dynamic(Dynamic { .. }) => unimplemented!(),

            Term::Compressed(_) => todo!(),
        })
    }
}

pub fn encode<T: MinCodecWrite>(
    data: T,
) -> Result<Vec<u8>, AsyncWriterError<std::io::Error, <T::Serialize as mincodec::Serialize>::Error>>
where
    T::Serialize: Unpin,
{
    let mut buffer = vec![];

    let fut = async {
        AsyncWriter::new(FuturesCompat::new(&mut buffer), data).await?;

        Ok(buffer)
    };

    let waker = noop_waker();
    let mut context = futures::task::Context::from_waker(&waker);

    let mut fut = Box::pin(fut);

    let data = loop {
        match fut.as_mut().poll(&mut context) {
            std::task::Poll::Ready(data) => break data,
            std::task::Poll::Pending => {}
        }
    };

    data
}

pub fn decode<T: MinCodecRead>(
    buffer: &[u8],
) -> Result<T, AsyncReaderError<std::io::Error, <T::Deserialize as mincodec::Deserialize>::Error>> {
    let fut = async { AsyncReader::<_, T>::new(FuturesCompat::new(buffer)).await };

    let waker = noop_waker();
    let mut context = futures::task::Context::from_waker(&waker);

    let mut fut = Box::pin(fut);

    let data = loop {
        match fut.as_mut().poll(&mut context) {
            std::task::Poll::Ready(data) => break data,
            std::task::Poll::Pending => {}
        }
    };

    data
}

impl TermData {
    pub async fn encode(
        self,
    ) -> Result<
        String,
        AsyncWriterError<
            std::io::Error,
            <<TermData as MinCodecWrite>::Serialize as mincodec::Serialize>::Error,
        >,
    > {
        let mut buffer = vec![];

        AsyncWriter::new(FuturesCompat::new(&mut buffer), self).await?;

        let buffer = base91::slice_encode(&buffer);

        Ok(format!(
            "welkin:{}",
            String::from_utf8_lossy(&buffer).as_ref()
        ))
    }

    pub async fn decode(
        data: String,
    ) -> Result<
        Option<Self>,
        AsyncReaderError<
            std::io::Error,
            <<TermData as MinCodecRead>::Deserialize as mincodec::Deserialize>::Error,
        >,
    > {
        let data = data.trim();

        if !data.starts_with("welkin:") {
            return Ok(None);
        }

        let data: String = data.chars().skip("welkin:".len()).collect();

        let buffer = base91::slice_decode(data.as_bytes());

        AsyncReader::new(FuturesCompat::new(buffer.as_slice()))
            .await
            .map(Some)
    }
}

#[derive(Debug, Clone)]
pub enum Path<T = ()> {
    Top,
    Lambda {
        erased: bool,
        name: Option<String>,
        up: Box<Path<T>>,
        annotation: T,
    },
    ApplicationFunction {
        erased: bool,
        argument: Term<T>,
        up: Box<Path<T>>,
        annotation: T,
    },
    ApplicationArgument {
        erased: bool,
        function: Term<T>,
        up: Box<Path<T>>,
        annotation: T,
    },
    Put {
        up: Box<Path<T>>,
        annotation: T,
    },
    Reference {
        name: String,
        up: Box<Path<T>>,
        annotation: T,
    },
    DuplicationExpression {
        binder: Option<String>,
        body: Term<T>,
        up: Box<Path<T>>,
        annotation: T,
    },
    DuplicationBody {
        binder: Option<String>,
        expression: Term<T>,
        up: Box<Path<T>>,
        annotation: T,
    },

    Universe {
        up: Box<Path<T>>,
        annotation: T,
    },
    FunctionArgumentType {
        erased: bool,
        name: Option<String>,
        self_name: Option<String>,
        return_type: Term<T>,
        up: Box<Path<T>>,
        annotation: T,
    },
    FunctionReturnType {
        erased: bool,
        name: Option<String>,
        self_name: Option<String>,
        argument_type: Term<T>,
        up: Box<Path<T>>,
        annotation: T,
    },
    Wrap {
        up: Box<Path<T>>,
        annotation: T,
    },

    Hole {
        up: Box<Path<T>>,
        annotation: T,
    },

    Dynamic {
        up: Box<Path<T>>,
        branch: Box<dyn BranchWrapper<T>>,
        annotation: T,
    },
}

pub trait BranchWrapper<T> {
    fn reconstruct(self: Box<Self>, term: Term<T>) -> Box<dyn DynamicTerm<T>>;
    fn box_clone(&self) -> Box<dyn BranchWrapper<T>>;
    fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

impl<T: fmt::Debug> Debug for Box<dyn BranchWrapper<T>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug(f)
    }
}

impl<T: Clone> Clone for Box<dyn BranchWrapper<T>> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

impl<T> Path<T> {
    fn is_top(&self) -> bool {
        matches!(self, Path::Top)
    }
}

#[derive(Debug, Clone)]
pub struct LambdaCursor<T> {
    erased: bool,
    name: Option<String>,
    body: Term<T>,
    up: Path<T>,
    annotation: T,
}

impl<T> LambdaCursor<T> {
    pub fn with_name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }

    pub fn with_body(mut self, body: Term<T>) -> Self {
        self.body = body;
        self
    }

    pub fn annotation(&self) -> &T {
        &self.annotation
    }

    pub fn annotation_mut(&mut self) -> &mut T {
        &mut self.annotation
    }

    pub fn erased(&self) -> bool {
        self.erased
    }

    pub fn erased_mut(&mut self) -> &mut bool {
        &mut self.erased
    }

    pub fn into_hole(self, annotation: T) -> HoleCursor<T> {
        HoleCursor {
            up: self.up,
            annotation,
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|a| a.as_str())
    }

    pub fn body(self) -> Cursor<T>
    where
        T: 'static,
    {
        Cursor::from_term_and_path(
            self.body,
            Path::Lambda {
                erased: self.erased,
                name: self.name,
                annotation: self.annotation,
                up: Box::new(self.up),
            },
        )
    }

    pub fn ascend(self) -> Cursor<T>
    where
        T: 'static,
    {
        Cursor::ascend_helper(
            self.up,
            Term::Lambda {
                erased: self.erased,
                name: self.name,
                annotation: self.annotation,
                body: Box::new(self.body),
            },
        )
        .unwrap_or_else(|(path, term)| Cursor::from_term_and_path(term, path))
    }
}

#[derive(Debug, Clone)]
pub struct ApplicationCursor<T> {
    erased: bool,
    function: Term<T>,
    argument: Term<T>,
    up: Path<T>,
    annotation: T,
}

impl<T> ApplicationCursor<T> {
    pub fn annotation(&self) -> &T {
        &self.annotation
    }

    pub fn annotation_mut(&mut self) -> &mut T {
        &mut self.annotation
    }

    pub fn erased(&self) -> bool {
        self.erased
    }

    pub fn erased_mut(&mut self) -> &mut bool {
        &mut self.erased
    }

    pub fn into_hole(self, annotation: T) -> HoleCursor<T> {
        HoleCursor {
            up: self.up,
            annotation,
        }
    }

    pub fn with_function(mut self, function: Term<T>) -> Self {
        self.function = function;
        self
    }

    pub fn with_argument(mut self, argument: Term<T>) -> Self {
        self.argument = argument;
        self
    }

    pub fn function(self) -> Cursor<T>
    where
        T: 'static,
    {
        Cursor::from_term_and_path(
            self.function,
            Path::ApplicationFunction {
                erased: self.erased,
                argument: self.argument,
                annotation: self.annotation,
                up: Box::new(self.up),
            },
        )
    }

    pub fn argument(self) -> Cursor<T>
    where
        T: 'static,
    {
        Cursor::from_term_and_path(
            self.argument,
            Path::ApplicationArgument {
                erased: self.erased,
                annotation: self.annotation,
                function: self.function,
                up: Box::new(self.up),
            },
        )
    }

    pub fn ascend(self) -> Cursor<T>
    where
        T: 'static,
    {
        Cursor::ascend_helper(
            self.up,
            Term::Application {
                erased: self.erased,
                annotation: self.annotation,
                function: Box::new(self.function),
                argument: Box::new(self.argument),
            },
        )
        .unwrap_or_else(|(path, term)| Cursor::from_term_and_path(term, path))
    }
}

#[derive(Debug, Clone)]
pub struct PutCursor<T> {
    term: Term<T>,
    up: Path<T>,
    annotation: T,
}

impl<T> PutCursor<T> {
    pub fn annotation(&self) -> &T {
        &self.annotation
    }

    pub fn annotation_mut(&mut self) -> &mut T {
        &mut self.annotation
    }

    pub fn term(self) -> Cursor<T>
    where
        T: 'static,
    {
        Cursor::from_term_and_path(
            self.term,
            Path::Put {
                annotation: self.annotation,
                up: Box::new(self.up),
            },
        )
    }

    pub fn with_term(mut self, term: Term<T>) -> Self {
        self.term = term;
        self
    }

    pub fn into_hole(self, annotation: T) -> HoleCursor<T> {
        HoleCursor {
            up: self.up,
            annotation,
        }
    }

    pub fn ascend(self) -> Cursor<T>
    where
        T: 'static,
    {
        Cursor::ascend_helper(self.up, Term::Put(Box::new(self.term), self.annotation))
            .unwrap_or_else(|(path, term)| Cursor::from_term_and_path(term, path))
    }
}

#[derive(Debug, Clone)]
pub struct ReferenceCursor<T> {
    name: String,
    up: Path<T>,
    annotation: T,
}

impl<T> ReferenceCursor<T> {
    pub fn annotation(&self) -> &T {
        &self.annotation
    }

    pub fn annotation_mut(&mut self) -> &mut T {
        &mut self.annotation
    }

    pub fn into_hole(self, annotation: T) -> HoleCursor<T> {
        HoleCursor {
            up: self.up,
            annotation,
        }
    }

    pub fn with_name(self, name: String) -> Self {
        ReferenceCursor {
            name,
            up: self.up,
            annotation: self.annotation,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ascend(self) -> Cursor<T>
    where
        T: 'static,
    {
        Cursor::ascend_helper(self.up, Term::Reference(self.name, self.annotation))
            .unwrap_or_else(|(path, term)| Cursor::from_term_and_path(term, path))
    }
}

#[derive(Debug, Clone)]
pub struct DuplicationCursor<T> {
    expression: Term<T>,
    binder: Option<String>,
    body: Term<T>,
    up: Path<T>,
    annotation: T,
}

impl<T> DuplicationCursor<T> {
    pub fn annotation(&self) -> &T {
        &self.annotation
    }

    pub fn annotation_mut(&mut self) -> &mut T {
        &mut self.annotation
    }

    pub fn binder(&self) -> Option<&str> {
        self.binder.as_ref().map(|a| a.as_str())
    }

    pub fn expression(self) -> Cursor<T>
    where
        T: 'static,
    {
        Cursor::from_term_and_path(
            self.expression,
            Path::DuplicationExpression {
                binder: self.binder,
                annotation: self.annotation,
                body: self.body,
                up: Box::new(self.up),
            },
        )
    }

    pub fn with_expression(mut self, term: Term<T>) -> Self {
        self.expression = term;
        self
    }

    pub fn with_body(mut self, term: Term<T>) -> Self {
        self.body = term;
        self
    }

    pub fn into_hole(self, annotation: T) -> HoleCursor<T> {
        HoleCursor {
            up: self.up,
            annotation,
        }
    }

    pub fn with_binder(mut self, binder: Option<String>) -> Self {
        self.binder = binder;
        self
    }

    pub fn body(self) -> Cursor<T>
    where
        T: 'static,
    {
        Cursor::from_term_and_path(
            self.body,
            Path::DuplicationBody {
                expression: self.expression,
                binder: self.binder,
                up: Box::new(self.up),
                annotation: self.annotation,
            },
        )
    }

    pub fn ascend(self) -> Cursor<T>
    where
        T: 'static,
    {
        Cursor::ascend_helper(
            self.up,
            Term::Duplication {
                binder: self.binder,
                expression: Box::new(self.expression),
                body: Box::new(self.body),
                annotation: self.annotation,
            },
        )
        .unwrap_or_else(|(path, term)| Cursor::from_term_and_path(term, path))
    }
}

#[derive(Debug, Clone)]
pub struct UniverseCursor<T> {
    path: Path<T>,
    annotation: T,
}

impl<T> UniverseCursor<T> {
    pub fn annotation(&self) -> &T {
        &self.annotation
    }

    pub fn annotation_mut(&mut self) -> &mut T {
        &mut self.annotation
    }

    pub fn ascend(self) -> Cursor<T>
    where
        T: 'static,
    {
        Cursor::ascend_helper(self.path, Term::Universe(self.annotation))
            .unwrap_or_else(|(path, term)| Cursor::from_term_and_path(term, path))
    }

    pub fn into_hole(self, annotation: T) -> HoleCursor<T> {
        HoleCursor {
            up: self.path,
            annotation,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionCursor<T> {
    argument_type: Term<T>,
    return_type: Term<T>,
    up: Path<T>,
    binder: Option<String>,
    annotation: T,
    self_binder: Option<String>,
    erased: bool,
}

impl<T> FunctionCursor<T> {
    pub fn annotation(&self) -> &T {
        &self.annotation
    }

    pub fn annotation_mut(&mut self) -> &mut T {
        &mut self.annotation
    }

    pub fn binder(&self) -> Option<&str> {
        self.binder.as_ref().map(|a| a.as_str())
    }

    pub fn self_binder(&self) -> Option<&str> {
        self.self_binder.as_ref().map(|a| a.as_str())
    }

    pub fn erased(&self) -> bool {
        self.erased
    }

    pub fn erased_mut(&mut self) -> &mut bool {
        &mut self.erased
    }

    pub fn argument_type(self) -> Cursor<T>
    where
        T: 'static,
    {
        Cursor::from_term_and_path(
            self.argument_type,
            Path::FunctionArgumentType {
                name: self.binder,
                self_name: self.self_binder,
                return_type: self.return_type,
                erased: self.erased,
                up: Box::new(self.up),
                annotation: self.annotation,
            },
        )
    }

    pub fn with_argument_type(mut self, argument_type: Term<T>) -> Self {
        self.argument_type = argument_type;
        self
    }

    pub fn with_return_type(mut self, return_type: Term<T>) -> Self {
        self.return_type = return_type;
        self
    }

    pub fn with_name(mut self, name: Option<String>) -> Self {
        self.binder = name;
        self
    }

    pub fn with_self_name(mut self, self_name: Option<String>) -> Self {
        self.self_binder = self_name;
        self
    }

    pub fn return_type(self) -> Cursor<T>
    where
        T: 'static,
    {
        Cursor::from_term_and_path(
            self.return_type,
            Path::FunctionReturnType {
                erased: self.erased,
                self_name: self.self_binder,
                argument_type: self.argument_type,
                name: self.binder,
                up: Box::new(self.up),
                annotation: self.annotation,
            },
        )
    }

    pub fn ascend(self) -> Cursor<T>
    where
        T: 'static,
    {
        Cursor::ascend_helper(
            self.up,
            Term::Function {
                erased: self.erased,
                annotation: self.annotation,
                argument_type: Box::new(self.argument_type),
                return_type: Box::new(self.return_type),
                name: self.binder,
                self_name: self.self_binder,
            },
        )
        .unwrap_or_else(|(path, term)| Cursor::from_term_and_path(term, path))
    }

    pub fn into_hole(self, annotation: T) -> HoleCursor<T> {
        HoleCursor {
            up: self.up,
            annotation,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WrapCursor<T> {
    term: Term<T>,
    up: Path<T>,
    annotation: T,
}

impl<T> WrapCursor<T> {
    pub fn annotation(&self) -> &T {
        &self.annotation
    }

    pub fn annotation_mut(&mut self) -> &mut T {
        &mut self.annotation
    }

    pub fn term(self) -> Cursor<T>
    where
        T: 'static,
    {
        Cursor::from_term_and_path(
            self.term,
            Path::Wrap {
                up: Box::new(self.up),
                annotation: self.annotation,
            },
        )
    }

    pub fn with_term(mut self, term: Term<T>) -> Self {
        self.term = term;
        self
    }

    pub fn into_hole(self, annotation: T) -> HoleCursor<T> {
        HoleCursor {
            annotation,
            up: self.up,
        }
    }

    pub fn ascend(self) -> Cursor<T>
    where
        T: 'static,
    {
        Cursor::ascend_helper(self.up, Term::Wrap(Box::new(self.term), self.annotation))
            .unwrap_or_else(|(path, term)| Cursor::from_term_and_path(term, path))
    }
}

#[derive(Debug, Clone)]
pub struct HoleCursor<T> {
    up: Path<T>,
    annotation: T,
}

impl<T> HoleCursor<T> {
    pub fn new(up: Path<T>, annotation: T) -> Self {
        HoleCursor { up, annotation }
    }

    pub fn annotation(&self) -> &T {
        &self.annotation
    }

    pub fn annotation_mut(&mut self) -> &mut T {
        &mut self.annotation
    }

    pub fn ascend(self) -> Cursor<T>
    where
        T: 'static,
    {
        Cursor::ascend_helper(self.up, Term::Hole(self.annotation))
            .unwrap_or_else(|(path, term)| Cursor::from_term_and_path(term, path))
    }
}

#[derive(Debug, Clone)]
pub struct DynamicCursor<T> {
    pub up: Path<T>,
    pub annotation: T,
    pub term: Box<dyn DynamicTerm<T>>,
}

#[derive(Debug, Clone)]
pub enum Cursor<T = ()> {
    Lambda(LambdaCursor<T>),
    Application(ApplicationCursor<T>),
    Put(PutCursor<T>),
    Reference(ReferenceCursor<T>),
    Duplication(DuplicationCursor<T>),

    Universe(UniverseCursor<T>),
    Function(FunctionCursor<T>),
    Wrap(WrapCursor<T>),

    Hole(HoleCursor<T>),

    Dynamic(DynamicCursor<T>),
}

impl<T: 'static> Cursor<T> {
    pub fn from_term_and_path(term: Term<T>, up: Path<T>) -> Self {
        match term {
            Term::Lambda {
                erased,
                annotation,
                name,
                body,
            } => Cursor::Lambda(LambdaCursor {
                erased,
                name,
                up,
                annotation,
                body: *body,
            }),
            Term::Application {
                erased,
                function,
                argument,
                annotation,
            } => Cursor::Application(ApplicationCursor {
                up,
                function: *function,
                annotation,
                erased,
                argument: *argument,
            }),
            Term::Put(term, annotation) => Cursor::Put(PutCursor {
                term: *term,
                annotation,
                up,
            }),
            Term::Duplication {
                binder,
                expression,
                body,
                annotation,
            } => Cursor::Duplication(DuplicationCursor {
                binder,
                expression: *expression,
                body: *body,
                annotation,
                up,
            }),
            Term::Reference(name, annotation) => Cursor::Reference(ReferenceCursor {
                name,
                up,
                annotation,
            }),

            Term::Universe(annotation) => Cursor::Universe(UniverseCursor {
                path: up,
                annotation,
            }),
            Term::Function {
                erased,
                name,
                argument_type,
                self_name,
                annotation,
                return_type,
            } => Cursor::Function(FunctionCursor {
                up,
                erased,
                annotation,
                binder: name,
                self_binder: self_name,
                argument_type: *argument_type,
                return_type: *return_type,
            }),
            Term::Wrap(term, annotation) => Cursor::Wrap(WrapCursor {
                term: *term,
                up,
                annotation,
            }),

            Term::Hole(annotation) => Cursor::Hole(HoleCursor { up, annotation }),

            Term::Dynamic(Dynamic { term, annotation }) => Cursor::Dynamic(DynamicCursor {
                term,
                annotation,
                up,
            }),

            Term::Compressed(data) => Cursor::from_term_and_path(data.expand(), up),
        }
    }

    fn ascend_helper(up: Path<T>, down: Term<T>) -> Result<Self, (Path<T>, Term<T>)> {
        Ok(match up {
            Path::Top => Err((up, down))?,
            Path::Lambda {
                erased,
                name,
                up,
                annotation,
            } => Cursor::Lambda(LambdaCursor {
                annotation,
                erased,
                name,
                body: down,
                up: *up,
            }),
            Path::ApplicationFunction {
                erased,
                argument,
                annotation,
                up,
            } => Cursor::Application(ApplicationCursor {
                erased,
                argument,
                annotation,
                up: *up,
                function: down,
            }),
            Path::ApplicationArgument {
                annotation,
                erased,
                function,
                up,
            } => Cursor::Application(ApplicationCursor {
                erased,
                annotation,
                function,
                up: *up,
                argument: down,
            }),
            Path::Put { up, annotation } => Cursor::Put(PutCursor {
                up: *up,
                term: down,
                annotation,
            }),
            Path::Reference {
                name,
                up,
                annotation,
            } => Cursor::Reference(ReferenceCursor {
                name,
                up: *up,
                annotation,
            }),
            Path::DuplicationExpression {
                binder,
                body,
                up,
                annotation,
            } => Cursor::Duplication(DuplicationCursor {
                binder,
                body,
                up: *up,
                expression: down,
                annotation,
            }),
            Path::DuplicationBody {
                binder,
                expression,
                annotation,
                up,
            } => Cursor::Duplication(DuplicationCursor {
                expression,
                binder,
                body: down,
                annotation,
                up: *up,
            }),

            Path::Universe { up, annotation } => Cursor::Universe(UniverseCursor {
                path: *up,
                annotation,
            }),
            Path::FunctionArgumentType {
                erased,
                name,
                annotation,
                self_name,
                return_type,
                up,
            } => Cursor::Function(FunctionCursor {
                up: *up,
                erased,
                binder: name,
                return_type,
                argument_type: down,
                self_binder: self_name,
                annotation,
            }),
            Path::FunctionReturnType {
                erased,
                name,
                self_name,
                argument_type,
                annotation,
                up,
            } => Cursor::Function(FunctionCursor {
                up: *up,
                erased,
                binder: name,
                self_binder: self_name,
                annotation,
                return_type: down,
                argument_type,
            }),
            Path::Wrap { up, annotation } => Cursor::Wrap(WrapCursor {
                term: down,
                up: *up,
                annotation,
            }),

            Path::Hole { up, annotation } => Cursor::Hole(HoleCursor {
                up: *up,
                annotation,
            }),

            Path::Dynamic {
                up,
                branch,
                annotation,
            } => Cursor::Dynamic(DynamicCursor {
                up: *up,
                annotation,
                term: branch.reconstruct(down),
            }),
        })
    }

    pub fn ascend(self) -> Self {
        match self {
            Cursor::Lambda(cursor) => cursor.ascend(),
            Cursor::Application(cursor) => cursor.ascend(),
            Cursor::Put(cursor) => cursor.ascend(),
            Cursor::Reference(cursor) => cursor.ascend(),
            Cursor::Duplication(cursor) => cursor.ascend(),

            Cursor::Universe(cursor) => cursor.ascend(),
            Cursor::Function(cursor) => cursor.ascend(),
            Cursor::Wrap(cursor) => cursor.ascend(),

            Cursor::Hole(cursor) => cursor.ascend(),

            Cursor::Dynamic(cursor) => cursor.ascend(),
        }
    }

    pub fn annotation(&self) -> &T {
        match self {
            Cursor::Lambda(cursor) => cursor.annotation(),
            Cursor::Application(cursor) => cursor.annotation(),
            Cursor::Put(cursor) => cursor.annotation(),
            Cursor::Reference(cursor) => cursor.annotation(),
            Cursor::Duplication(cursor) => cursor.annotation(),

            Cursor::Universe(cursor) => cursor.annotation(),
            Cursor::Function(cursor) => cursor.annotation(),
            Cursor::Wrap(cursor) => cursor.annotation(),

            Cursor::Hole(cursor) => cursor.annotation(),

            Cursor::Dynamic(cursor) => cursor.annotation(),
        }
    }

    pub fn annotation_mut(&mut self) -> &mut T {
        match self {
            Cursor::Lambda(cursor) => cursor.annotation_mut(),
            Cursor::Application(cursor) => cursor.annotation_mut(),
            Cursor::Put(cursor) => cursor.annotation_mut(),
            Cursor::Reference(cursor) => cursor.annotation_mut(),
            Cursor::Duplication(cursor) => cursor.annotation_mut(),

            Cursor::Universe(cursor) => cursor.annotation_mut(),
            Cursor::Function(cursor) => cursor.annotation_mut(),
            Cursor::Wrap(cursor) => cursor.annotation_mut(),

            Cursor::Hole(cursor) => cursor.annotation_mut(),

            Cursor::Dynamic(cursor) => cursor.annotation_mut(),
        }
    }

    pub fn is_top(&self) -> bool {
        self.path().is_top()
    }

    pub fn path(&self) -> &Path<T> {
        match self {
            Cursor::Lambda(cursor) => &cursor.up,
            Cursor::Application(cursor) => &cursor.up,
            Cursor::Put(cursor) => &cursor.up,
            Cursor::Reference(cursor) => &cursor.up,
            Cursor::Duplication(cursor) => &cursor.up,
            Cursor::Universe(cursor) => &cursor.path,
            Cursor::Function(cursor) => &cursor.up,
            Cursor::Wrap(cursor) => &cursor.up,
            Cursor::Hole(cursor) => &cursor.up,
            Cursor::Dynamic(cursor) => &cursor.up,
        }
    }

    pub fn path_mut(&mut self) -> &mut Path<T> {
        match self {
            Cursor::Lambda(cursor) => &mut cursor.up,
            Cursor::Application(cursor) => &mut cursor.up,
            Cursor::Put(cursor) => &mut cursor.up,
            Cursor::Reference(cursor) => &mut cursor.up,
            Cursor::Duplication(cursor) => &mut cursor.up,
            Cursor::Universe(cursor) => &mut cursor.path,
            Cursor::Function(cursor) => &mut cursor.up,
            Cursor::Wrap(cursor) => &mut cursor.up,
            Cursor::Hole(cursor) => &mut cursor.up,
            Cursor::Dynamic(cursor) => &mut cursor.up,
        }
    }

    pub fn context(&self) -> Context<T>
    where
        T: Clone,
    {
        let done = self.is_top();
        Context {
            cursor: self.clone(),
            done,
            next: None,
        }
    }
}

impl<T: 'static> From<Term<T>> for Cursor<T> {
    fn from(term: Term<T>) -> Self {
        Cursor::from_term_and_path(term, Path::Top)
    }
}

pub struct Context<T> {
    cursor: Cursor<T>,
    done: bool,
    next: Option<Option<String>>,
}

impl<T: Clone + 'static> Iterator for Context<T> {
    type Item = Option<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.next.take() {
            return Some(next);
        }

        if self.done {
            return None;
        }

        let mut c = self.cursor.clone();

        let r = loop {
            if let Some(binder) = {
                if match c.path() {
                    Path::DuplicationExpression { .. } => true,
                    Path::FunctionArgumentType { .. } => true,
                    _ => false,
                } && {
                    let mut c = c.clone();
                    c = c.ascend();
                    c.is_top()
                } {
                    self.done = true;
                    return None;
                }

                let binder = match &c {
                    Cursor::Lambda(cursor) => Some(cursor.name().map(|a| a.to_owned())),
                    Cursor::Duplication(cursor) => Some(cursor.binder().map(|a| a.to_owned())),
                    Cursor::Function(cursor) => {
                        self.next = Some(cursor.self_binder().map(|a| a.to_owned()));
                        Some(cursor.binder().map(|a| a.to_owned()))
                    }
                    _ => None,
                };
                self.done = c.is_top();

                loop {
                    match c.path() {
                        Path::DuplicationExpression { .. } => {
                            c = c.ascend();
                        }
                        Path::FunctionArgumentType { .. } => {
                            c = c.ascend();
                        }
                        _ => break,
                    }
                }

                c = c.ascend();
                binder
            } {
                break Some(binder);
            } else if self.done {
                break None;
            }
        };

        self.cursor = c;

        r
    }
}

impl<T> From<Cursor<T>> for Term<T> {
    fn from(cursor: Cursor<T>) -> Self {
        match cursor {
            Cursor::Lambda(cursor) => Term::Lambda {
                erased: cursor.erased,
                body: Box::new(cursor.body),
                name: cursor.name,
                annotation: cursor.annotation,
            },
            Cursor::Application(cursor) => Term::Application {
                erased: cursor.erased,
                function: Box::new(cursor.function),
                argument: Box::new(cursor.argument),
                annotation: cursor.annotation,
            },
            Cursor::Put(cursor) => Term::Put(Box::new(cursor.term), cursor.annotation),
            Cursor::Reference(cursor) => Term::Reference(cursor.name, cursor.annotation),
            Cursor::Duplication(cursor) => Term::Duplication {
                binder: cursor.binder,
                expression: Box::new(cursor.expression),
                body: Box::new(cursor.body),
                annotation: cursor.annotation,
            },

            Cursor::Universe(cursor) => Term::Universe(cursor.annotation),
            Cursor::Function(cursor) => Term::Function {
                erased: cursor.erased,
                self_name: cursor.self_binder,
                name: cursor.binder,
                argument_type: Box::new(cursor.argument_type),
                return_type: Box::new(cursor.return_type),
                annotation: cursor.annotation,
            },
            Cursor::Wrap(cursor) => Term::Wrap(Box::new(cursor.term), cursor.annotation),

            Cursor::Hole(cursor) => Term::Hole(cursor.annotation),

            Cursor::Dynamic(cursor) => Term::Dynamic(Dynamic {
                annotation: cursor.annotation,
                term: cursor.term,
            }),
        }
    }
}

impl<T: Clone + 'static> Cursor<T> {
    pub fn into_term(self) -> Option<term::Term<String>> {
        let cursor = self;
        Some(match cursor {
            Cursor::Lambda(cursor) => term::Term::Lambda {
                erased: cursor.erased(),
                body: Box::new(cursor.body().into_term()?),
            },
            Cursor::Application(cursor) => term::Term::Apply {
                erased: cursor.erased(),
                function: Box::new(cursor.clone().function().into_term()?),
                argument: Box::new(cursor.argument().into_term()?),
            },
            Cursor::Put(cursor) => term::Term::Put(Box::new(cursor.term().into_term()?)),
            Cursor::Reference(ref c) => {
                if let Some(idx) = cursor.context().position(|name| {
                    if let Some(name) = name {
                        if c.name() == &name {
                            return true;
                        }
                    }
                    false
                }) {
                    term::Term::Variable(Index(idx))
                } else {
                    term::Term::Reference(c.name().to_owned())
                }
            }
            Cursor::Duplication(cursor) => term::Term::Duplicate {
                expression: Box::new(cursor.clone().expression().into_term()?),
                body: Box::new(cursor.body().into_term()?),
            },

            Cursor::Universe(_) => term::Term::Universe,
            Cursor::Function(cursor) => term::Term::Function {
                erased: cursor.erased(),
                argument_type: Box::new(cursor.clone().argument_type().into_term()?),
                return_type: Box::new(cursor.return_type().into_term()?),
            },
            Cursor::Wrap(cursor) => term::Term::Wrap(Box::new(cursor.term().into_term()?)),

            Cursor::Hole(_) => None?,

            Cursor::Dynamic(cursor) => Cursor::from(cursor.expand()).into_term()?,
        })
    }
}
