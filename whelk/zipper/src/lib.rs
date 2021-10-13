#![recursion_limit = "2048"]
use core_futures_io::FuturesCompat;
use mincodec::{
    AsyncReader, AsyncReaderError, AsyncWriter, AsyncWriterError, Deserialize, MinCodec,
    MinCodecRead, MinCodecWrite, Serialize,
};
use welkin_core::term::{self, Index};

#[derive(Debug, Clone, MinCodec)]
#[bounds(T)]
pub enum Term<T = ()> {
    Lambda {
        erased: bool,
        name: Option<String>,
        body: Box<Term<T>>,
        annotation: T,
    },
    Application {
        erased: bool,
        function: Box<Term<T>>,
        argument: Box<Term<T>>,
        annotation: T,
    },
    Put(Box<Term<T>>, T),
    Duplication {
        binder: Option<String>,
        expression: Box<Term<T>>,
        body: Box<Term<T>>,
        annotation: T,
    },
    Reference(String, T),

    Universe(T),
    Function {
        erased: bool,
        name: Option<String>,
        self_name: Option<String>,
        argument_type: Box<Term<T>>,
        return_type: Box<Term<T>>,
        annotation: T,
    },
    Wrap(Box<Term<T>>, T),

    Hole(T),
}

impl<T> Term<T> {
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
        })
    }
}

impl<T: MinCodecWrite + MinCodecRead + Unpin> Term<T>
where
    T::Serialize: Unpin,
{
    pub async fn encode(
        self,
    ) -> Result<
        String,
        AsyncWriterError<
            std::io::Error,
            <<Term<T> as MinCodecWrite>::Serialize as Serialize>::Error,
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
            <<Term<T> as MinCodecRead>::Deserialize as Deserialize>::Error,
        >,
    >
    where
        T::Deserialize: Unpin,
    {
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

    pub fn body(self) -> Cursor<T> {
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

    pub fn ascend(self) -> Cursor<T> {
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

    pub fn function(self) -> Cursor<T> {
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

    pub fn argument(self) -> Cursor<T> {
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

    pub fn ascend(self) -> Cursor<T> {
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

    pub fn term(self) -> Cursor<T> {
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

    pub fn ascend(self) -> Cursor<T> {
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

    pub fn into_hole(self, annotation: T) -> HoleCursor<T> {
        HoleCursor {
            up: self.up,
            annotation,
        }
    }

    pub fn annotation_mut(&mut self) -> &mut T {
        &mut self.annotation
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

    pub fn ascend(self) -> Cursor<T> {
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

    pub fn binder(&self) -> Option<&str> {
        self.binder.as_ref().map(|a| a.as_str())
    }

    pub fn expression(self) -> Cursor<T> {
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

    pub fn body(self) -> Cursor<T> {
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

    pub fn ascend(self) -> Cursor<T> {
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

    pub fn ascend(self) -> Cursor<T> {
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

    pub fn binder(&self) -> Option<&str> {
        self.binder.as_ref().map(|a| a.as_str())
    }

    pub fn self_binder(&self) -> Option<&str> {
        self.self_binder.as_ref().map(|a| a.as_str())
    }

    pub fn erased(&self) -> bool {
        self.erased
    }

    pub fn argument_type(self) -> Cursor<T> {
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

    pub fn return_type(self) -> Cursor<T> {
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

    pub fn ascend(self) -> Cursor<T> {
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

    pub fn term(self) -> Cursor<T> {
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

    pub fn ascend(self) -> Cursor<T> {
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
    pub fn annotation(&self) -> &T {
        &self.annotation
    }

    pub fn annotation_mut(&mut self) -> &mut T {
        &mut self.annotation
    }

    pub fn ascend(self) -> Cursor<T> {
        Cursor::ascend_helper(self.up, Term::Hole(self.annotation))
            .unwrap_or_else(|(path, term)| Cursor::from_term_and_path(term, path))
    }
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
}

impl<T> Cursor<T> {
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

impl<T> From<Term<T>> for Cursor<T> {
    fn from(term: Term<T>) -> Self {
        Cursor::from_term_and_path(term, Path::Top)
    }
}

pub struct Context<T> {
    cursor: Cursor<T>,
    done: bool,
    next: Option<Option<String>>,
}

impl<T: Clone> Iterator for Context<T> {
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
        }
    }
}

impl From<Cursor> for term::Term<String> {
    fn from(cursor: Cursor) -> Self {
        match cursor {
            Cursor::Lambda(cursor) => term::Term::Lambda {
                erased: cursor.erased(),
                body: Box::new(cursor.body().into()),
            },
            Cursor::Application(cursor) => term::Term::Apply {
                erased: cursor.erased(),
                function: Box::new(cursor.clone().function().into()),
                argument: Box::new(cursor.argument().into()),
            },
            Cursor::Put(cursor) => term::Term::Put(Box::new(cursor.term().into())),
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
                expression: Box::new(cursor.clone().expression().into()),
                body: Box::new(cursor.body().into()),
            },

            Cursor::Universe(_) => term::Term::Universe,
            Cursor::Function(cursor) => term::Term::Function {
                erased: cursor.erased(),
                argument_type: Box::new(cursor.clone().argument_type().into()),
                return_type: Box::new(cursor.return_type().into()),
            },
            Cursor::Wrap(cursor) => term::Term::Wrap(Box::new(cursor.term().into())),

            Cursor::Hole(_) => panic!(),
        }
    }
}
