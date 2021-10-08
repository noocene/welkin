use std::{
    collections::{HashMap, HashSet},
    convert::TryInto,
    fmt::{self, Display, Formatter},
    marker::PhantomData,
    str::FromStr,
};

use bumpalo::Bump;
use combine::{
    attempt, many1,
    parser::{
        char::{alpha_num, spaces},
        choice::or,
        combinator::no_partial,
    },
    sep_by1, token, Parser, Stream,
};
#[doc(hidden)]
pub use thiserror::Error;
use welkin::{
    compiler::{item::Compile, LocalResolver},
    parser::{self, BumpBox, BumpVec, Data, Ident, Path, Variant},
};
#[doc(hidden)]
pub use welkin::{parser::AbsolutePath, SerializableData};
#[doc(hidden)]
pub use welkin_core;
use welkin_core::term::{Referent, Term};

pub trait FromWelkin: Sized {
    type Error;

    fn from_welkin(term: Term<String>) -> Result<Self, Self::Error>;
}

pub trait ToWelkin {
    type Error;

    fn to_welkin(self) -> Result<Term<String>, Self::Error>;
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum AdtConstructor {
    Inductive,
    Other(&'static AdtDefinition),
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct ConcreteType {
    constructor: AdtConstructor,
    params: &'static [Type],
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]

pub enum Type {
    Parameter(usize),
    Data {
        constructor: AdtConstructor,
        params: &'static [Type],
    },
}

impl Type {
    fn generate<'a>(self, bump: &'a Bump, this: &str) -> parser::Term<'a> {
        match self {
            Type::Parameter(idx) => parser::Term::Reference(Path(BumpVec::unary_in(
                Ident::from_str(&format!("T{}", idx), &bump),
                &bump,
            ))),
            Type::Data {
                constructor,
                params,
            } => parser::Term::Application {
                erased: true,
                function: match constructor {
                    AdtConstructor::Inductive => BumpBox::new_in(
                        parser::Term::Reference(Path(BumpVec::unary_in(
                            Ident::from_str(this, &bump),
                            &bump,
                        ))),
                        bump,
                    ),
                    AdtConstructor::Other(definition) => BumpBox::new_in(
                        parser::Term::Reference(Path(BumpVec::unary_in(
                            Ident::from_str(definition.name, &bump),
                            &bump,
                        ))),
                        bump,
                    ),
                },
                arguments: BumpVec::from_iterator(
                    params.iter().map(|ty| ty.clone().generate(&bump, this)),
                    &bump,
                ),
            },
        }
    }
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub struct AdtVariant {
    pub fields: &'static [Type],
    pub name: &'static str,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct AdtDefinition {
    pub variants: &'static [AdtVariant],
    pub name: &'static str,
    pub params: usize,
}

#[derive(Debug)]
pub struct Definition {
    pub path: AbsolutePath,
    pub ty: Term<AbsolutePath>,
    pub term: Term<AbsolutePath>,
}

#[derive(Debug)]
pub struct Definitions {
    pub definitions: Vec<Definition>,
}

impl Display for Definitions {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut write_definition = |definition: &Definition, pad: bool| -> fmt::Result {
            write!(
                f,
                "{}{:?}:\n{:?}\n=\n{:?}",
                if pad { "\n\n" } else { "" },
                definition.path,
                definition.ty,
                definition.term,
            )?;

            Ok(())
        };

        let mut definitions = self.definitions.iter();

        if let Some(definition) = definitions.next() {
            write_definition(definition, false)?;

            for definition in definitions {
                write_definition(definition, true)?;
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
struct AbsolutePathReferent(AbsolutePath);

impl<Input: Stream<Token = char>> Referent<Input> for AbsolutePathReferent {
    fn as_str(&self) -> Option<&str> {
        if let Some(segment) = (self.0).0.first() {
            if (self.0).0.len() == 1 {
                return Some(segment.as_str());
            }
        }
        None
    }

    fn parse<'a>() -> Box<dyn Parser<Input, Output = Self, PartialState = ()> + 'a>
    where
        Input: 'a,
    {
        no_partial(
            spaces()
                .with(sep_by1(
                    many1(or(alpha_num(), token('_'))),
                    attempt((token(':'), token(':'))),
                ))
                .map(AbsolutePath)
                .map(AbsolutePathReferent),
        )
        .boxed()
    }
}

impl FromStr for Definitions {
    type Err = welkin_core::term::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut definitions = vec![];

        let defs =
            <welkin_core::term::typed::Definitions<AbsolutePathReferent> as FromStr>::from_str(s)?;

        for (path, (ty, term)) in defs.terms {
            definitions.push(Definition {
                path: path.0,
                ty: ty.map_reference(|r| Term::Reference(r.0)),
                term: term.map_reference(|r| Term::Reference(r.0)),
            });
        }

        Ok(Definitions { definitions })
    }
}

mod typed_sealed {
    use crate::{Adt, Analogous, Dummy};

    pub trait Sealed {}
    impl<T: Analogous> Sealed for T where T::Analogue: Adt {}
    impl<const INDEX: usize> Sealed for Dummy<INDEX> {}
}

pub trait Typed: typed_sealed::Sealed {
    const TYPE: Type;
}

impl<T: Analogous> Typed for T
where
    T::Analogue: Adt,
{
    const TYPE: Type = Type::Data {
        constructor: AdtConstructor::Other(&<<T as Analogous>::Analogue as Adt>::DEFINITION),
        params: <<T as Analogous>::Analogue as Adt>::PARAMS,
    };
}

#[doc(hidden)]
pub struct Dummy<const INDEX: usize>(PhantomData<[(); INDEX]>);

impl<const INDEX: usize> Typed for Dummy<INDEX> {
    const TYPE: Type = Type::Parameter(INDEX);
}

impl AdtDefinition {
    fn generate_data(self, bump: &'_ Bump) -> parser::Data<'_> {
        Data {
            variants: BumpVec::from_iterator(
                self.variants.iter().map(|variant| Variant {
                    ident: Ident::from_str(variant.name, &bump),
                    inhabitants: BumpVec::from_iterator(
                        variant.fields.iter().enumerate().map(|(idx, ty)| {
                            (
                                Ident::from_str(&format!("field{}", idx), &bump),
                                ty.clone().generate(&bump, self.name),
                                false,
                            )
                        }),
                        &bump,
                    ),
                    indices: BumpVec::new_in(&bump),
                }),
                &bump,
            ),
            type_arguments: BumpVec::from_iterator(
                (0..self.params)
                    .map(|idx| (Ident::from_str(&format!("T{}", idx), &bump), None, true)),
                &bump,
            ),
            indices: BumpVec::new_in(&bump),
            ident: Ident::from_str(self.name, &bump),
        }
    }

    pub fn generate(self) -> Definitions {
        let bump = Bump::new();

        let data = self.generate_data(&bump);

        Definitions {
            definitions: data
                .compile(LocalResolver::new())
                .into_iter()
                .map(|(path, ty, term)| Definition { path, ty, term })
                .collect(),
        }
    }
}

pub trait Adt {
    const DEFINITION: AdtDefinition;
    const PARAMS: &'static [Type];
}

pub trait Analogous {
    type Analogue;
}

pub trait FromAnalogue: Analogous<Analogue = <Self as FromAnalogue>::Analogue> {
    type Analogue: FromWelkin;

    fn from_analogue(analogue: <Self as FromAnalogue>::Analogue) -> Self;
}

pub trait ToAnalogue: Analogous<Analogue = <Self as ToAnalogue>::Analogue> {
    type Analogue: ToWelkin;

    fn to_analogue(self) -> <Self as ToAnalogue>::Analogue;
}

impl<T: Analogous> Analogous for Box<T> {
    type Analogue = T::Analogue;
}

mod sealed {
    pub trait Sealed {}
    impl<T> Sealed for Box<T> {}
}

pub trait Wrapper: sealed::Sealed {
    type Inner;
}

impl<T> Wrapper for Box<T> {
    type Inner = T;
}

trait ResolveParam {
    fn resolve(&self, idx: usize) -> (Type, Box<dyn ResolveParam>);
}

impl<T: ResolveParam + ?Sized> ResolveParam for Box<T> {
    fn resolve(&self, idx: usize) -> (Type, Box<dyn ResolveParam>) {
        T::resolve(&**self, idx)
    }
}

struct SliceResolver {
    params: &'static [Type],
}

impl ResolveParam for SliceResolver {
    fn resolve(&self, idx: usize) -> (Type, Box<dyn ResolveParam>) {
        (
            self.params[idx].clone(),
            Box::new(SliceResolver {
                params: match &self.params[idx] {
                    Type::Parameter(_) => todo!(),
                    Type::Data { params, .. } => params,
                },
            }),
        )
    }
}

fn resolve_dependencies(
    ty: &Type,
    register: &mut impl FnMut(&'static AdtDefinition),
    resolve_param: &impl ResolveParam,
) {
    match ty {
        Type::Parameter(idx) => {
            let (ty, resolver) = &resolve_param.resolve(*idx);
            resolve_dependencies(ty, &mut *register, resolver)
        }
        Type::Data {
            constructor,
            params,
        } => {
            if let AdtConstructor::Other(constructor) = constructor {
                register(constructor);
                for variant in constructor.variants {
                    for field in variant.fields {
                        resolve_dependencies(field, &mut *register, &*resolve_param)
                    }
                }
            }
            for param in *params {
                resolve_dependencies(param, &mut *register, &*resolve_param)
            }
        }
    }
}

fn generate_all_data<'a, A: Adt>(bump: &'a Bump) -> Vec<Data<'a>> {
    let mut dependencies = HashSet::new();

    dependencies.insert(&A::DEFINITION);

    for variant in A::DEFINITION.variants {
        for field in variant.fields {
            resolve_dependencies(
                field,
                &mut |definition| {
                    dependencies.insert(definition);
                },
                &SliceResolver { params: A::PARAMS },
            )
        }
    }

    dependencies
        .into_iter()
        .map(|a| a.clone().generate_data(&bump))
        .collect()
}

fn generate_all<A: Adt>() -> Definitions {
    let mut dependencies = HashSet::new();

    dependencies.insert(&A::DEFINITION);

    for variant in A::DEFINITION.variants {
        for field in variant.fields {
            resolve_dependencies(
                field,
                &mut |definition| {
                    dependencies.insert(definition);
                },
                &SliceResolver { params: A::PARAMS },
            )
        }
    }

    Definitions {
        definitions: dependencies
            .into_iter()
            .flat_map(|a| a.clone().generate().definitions.into_iter())
            .collect(),
    }
}

pub fn concrete_type<A: Adt>() -> Term<AbsolutePath> {
    fn helper(definition: &AdtDefinition, params: &[Type]) -> Term<AbsolutePath> {
        let mut term = Term::Reference(AbsolutePath(vec![definition.name.to_owned()]));

        for param in params.iter() {
            term = Term::Apply {
                erased: true,
                function: Box::new(term),
                argument: Box::new(match param {
                    Type::Parameter(_) => panic!(),
                    Type::Data {
                        constructor,
                        params,
                    } => match constructor {
                        AdtConstructor::Inductive => panic!(),
                        AdtConstructor::Other(definition) => helper(definition, params),
                    },
                }),
            };
        }

        term
    }

    helper(&A::DEFINITION, A::PARAMS)
}

#[derive(Debug, Error)]
pub enum CheckError {
    #[error("definition {0:?} is missing in welkin source")]
    Missing(AbsolutePath),
    #[error("definition {0:?} does not match declaration in welkin source")]
    Mismatch(AbsolutePath),
}

#[derive(Debug, Error)]
pub enum CanonicalEquivalenceError {
    #[error("check error: {0}")]
    Check(#[from] CheckError),
    #[error("bincode error: {0}")]
    Bincode(#[from] Box<bincode::ErrorKind>),
}

#[doc(hidden)]
pub fn check_in_helper(ty_defs: &Definitions, against: &Definitions) -> Result<(), CheckError> {
    let defs: HashMap<_, _> = against
        .definitions
        .iter()
        .map(|a| (a.path.clone(), (a.ty.clone(), a.term.clone())))
        .collect();

    for def in &ty_defs.definitions {
        if !def.ty.equals(
            &defs
                .get(&def.path)
                .ok_or(CheckError::Missing(def.path.clone()))?
                .0,
        ) {
            return Err(CheckError::Mismatch(def.path.clone()));
        }
        if !def.term.equals(
            &defs
                .get(&def.path)
                .ok_or(CheckError::Missing(def.path.clone()))?
                .1,
        ) {
            return Err(CheckError::Mismatch(def.path.clone()));
        }
    }

    Ok(())
}

pub fn check_all_in<A: Adt>(against: &Definitions) -> Result<(), CheckError> {
    check_in_helper(&generate_all::<A>(), against)
}

pub fn check_in<A: Adt>(against: &Definitions) -> Result<(), CheckError> {
    check_in_helper(&A::DEFINITION.generate(), against)
}

fn canonically_equivalent_fields(
    rust: &Term<AbsolutePath>,
    welkin: &Term<AbsolutePath>,
    definitions: &[SerializableData],
) -> bool {
    match (rust, welkin) {
        (Term::Variable(a), Term::Variable(b)) => a == b,
        (
            Term::Lambda { body, erased },
            Term::Lambda {
                body: b_body,
                erased: b_erased,
            },
        ) => canonically_equivalent_fields(body, b_body, definitions) && erased == b_erased,
        (
            Term::Apply {
                function,
                argument,
                erased,
            },
            Term::Apply {
                function: b_function,
                argument: b_argument,
                erased: b_erased,
            },
        ) => {
            let mut arguments = vec![&**b_argument];
            let mut func = &**b_function;

            while let Term::Apply {
                argument,
                function,
                erased: true,
            } = func
            {
                arguments.push(&**argument);
                func = &**function;
            }

            if let Term::Reference(path) = func {
                if let Some(segment) = path.0.first() {
                    if path.0.len() == 1 {
                        if let Some(data) = definitions.iter().find(|data| &data.ident == segment) {
                            arguments.reverse();
                            arguments.truncate(arguments.len() - data.indices);

                            let mut term = func.clone();

                            for argument in arguments {
                                term = Term::Apply {
                                    function: Box::new(term),
                                    argument: Box::new(argument.clone()),
                                    erased: true,
                                };
                            }

                            return rust.equals(&term);
                        }
                    }
                }
            }

            canonically_equivalent_fields(function, b_function, definitions)
                && canonically_equivalent_fields(argument, b_argument, definitions)
                && erased == b_erased
        }
        (Term::Put(a), Term::Put(b)) => canonically_equivalent_fields(a, b, definitions),
        (
            Term::Duplicate { expression, body },
            Term::Duplicate {
                expression: b_expression,
                body: b_body,
            },
        ) => {
            canonically_equivalent_fields(expression, b_expression, definitions)
                && canonically_equivalent_fields(body, b_body, definitions)
        }
        (Term::Reference(a), Term::Reference(b)) => a == b,
        (Term::Primitive(a), Term::Primitive(b)) => a == b,
        (Term::Universe, Term::Universe) => true,
        (
            Term::Function {
                argument_type,
                return_type,
                erased,
            },
            Term::Function {
                argument_type: b_argument_type,
                return_type: b_return_type,
                erased: b_erased,
            },
        ) => {
            canonically_equivalent_fields(argument_type, b_argument_type, definitions)
                && canonically_equivalent_fields(return_type, b_return_type, definitions)
                && erased == b_erased
        }
        (
            Term::Annotation {
                checked,
                expression,
                ty,
            },
            Term::Annotation {
                checked: b_checked,
                expression: b_expression,
                ty: b_ty,
            },
        ) => {
            canonically_equivalent_fields(expression, b_expression, definitions)
                && canonically_equivalent_fields(ty, b_ty, definitions)
                && checked == b_checked
        }
        (Term::Wrap(a), Term::Wrap(b)) => canonically_equivalent_fields(a, b, definitions),
        _ => false,
    }
}

fn canonically_equivalent_data(
    rust: &SerializableData,
    welkin: &SerializableData,
    definitions: &[SerializableData],
) -> bool {
    rust.ident == welkin.ident
        && rust.type_arguments == welkin.type_arguments
        && rust
            .variants
            .iter()
            .map(|a| a.0.as_str())
            .collect::<HashSet<_>>()
            == welkin
                .variants
                .iter()
                .map(|a| a.0.as_str())
                .collect::<HashSet<_>>()
        && rust
            .variants
            .iter()
            .zip(welkin.variants.iter())
            .all(|((_, rust), (_, welkin))| {
                rust.inhabitants.len() == welkin.inhabitants.len()
                    && rust.inhabitants.iter().zip(welkin.inhabitants.iter()).all(
                        |((_, rust), (_, welkin))| {
                            canonically_equivalent_fields(rust, welkin, definitions)
                        },
                    )
            })
}

fn canonically_equivalent_in_helper(
    data: Vec<SerializableData>,
    against: &[SerializableData],
) -> Result<(), CheckError> {
    for definition in data {
        if let Some(other) = against.iter().find(|data| data.ident == definition.ident) {
            if !canonically_equivalent_data(&definition, other, against) {
                println!("{:?} != {:?}", definition, other);
                return Err(CheckError::Mismatch(AbsolutePath(vec![definition.ident])));
            }
        } else {
            return Err(CheckError::Missing(AbsolutePath(vec![definition.ident])));
        }
    }

    Ok(())
}

pub fn canonically_equivalent_in<A: Adt>(against: &[u8]) -> Result<(), CanonicalEquivalenceError> {
    let bump = Bump::new();

    Ok(canonically_equivalent_in_helper(
        vec![A::DEFINITION.generate_data(&bump).try_into().unwrap()],
        &bincode::deserialize::<Vec<_>>(against)?,
    )?)
}

#[doc(hidden)]
pub fn deserialize_defs(buffer: &[u8]) -> Result<Vec<SerializableData>, CanonicalEquivalenceError> {
    bincode::deserialize(buffer).map_err(CanonicalEquivalenceError::Bincode)
}

pub fn canonically_equivalent_all_in<A: Adt>(
    against: &[u8],
) -> Result<(), CanonicalEquivalenceError> {
    let bump = Bump::new();

    Ok(canonically_equivalent_in_helper(
        generate_all_data::<A>(&bump)
            .into_iter()
            .map(|a| a.try_into().unwrap())
            .collect(),
        &bincode::deserialize::<Vec<_>>(against)?,
    )?)
}

#[macro_export]
macro_rules! impl_wrapper {
    (
        $($a:ty),+
    ) => {
        $(
        const _: () = {
        extern crate welkin_binding;
        impl welkin_binding::Analogous for $a {
            type Analogue = Self;
        }

        impl welkin_binding::ToAnalogue for $a {
            type Analogue = Self;

            fn to_analogue(self) -> <Self as welkin_binding::ToAnalogue>::Analogue {
                self
            }
        }

        impl welkin_binding::FromAnalogue for $a {
            type Analogue = Self;

            fn from_analogue(analogue: <Self as welkin_binding::FromAnalogue>::Analogue) -> Self {
                analogue
            }
        }

        impl welkin_binding::Adt for $a {
            const DEFINITION: welkin_binding::AdtDefinition = welkin_binding::AdtDefinition {
                name: stringify!($a),
                variants: &[],
                params: 0,
            };

            const PARAMS: &'static [welkin_binding::Type] = &[];
        }
        impl welkin_binding::ToWelkin for $a {
            type Error = std::convert::Infallible;

            fn to_welkin(self) -> Result<welkin_binding::welkin_core::term::Term<String>, Self::Error> {
                Ok(self.0)
            }
        }

        impl welkin_binding::FromWelkin for $a {
            type Error = std::convert::Infallible;

            fn from_welkin(term: welkin_binding::welkin_core::term::Term<String>) -> Result<Self, Self::Error> {
                Ok(Self(term))
            }
        }; ()};)+
    };
}
