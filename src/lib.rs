use std::ops::{Deref, DerefMut};

use bumpalo::Bump;
use welkin_core::term::{
    alloc::{Allocator, IntoInner, Reallocate, System},
    Primitives, Term,
};

pub mod hash;
use hash::{Hash, ReferenceHash};

pub mod definitions;

pub mod compiler;

mod sealed {
    use super::Term;

    pub trait Sealed {}

    impl<T> Sealed for Term<T> {}
}

pub trait TermExt: sealed::Sealed {
    fn hash(&self) -> Hash;
}

impl<T: ReferenceHash> TermExt for Term<T> {
    fn hash(&self) -> Hash {
        hash::hash(self)
    }
}

pub struct Bumpalo<'a>(pub &'a Bump);

pub struct BumpBox<'a, T>(bumpalo::boxed::Box<'a, Option<T>>);

impl<'a, T> Deref for BumpBox<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().as_ref().unwrap()
    }
}

impl<'a, T> DerefMut for BumpBox<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().as_mut().unwrap()
    }
}

impl<'a, T> IntoInner<T> for BumpBox<'a, T> {
    fn into_inner(mut self) -> T {
        self.0.as_mut().take().unwrap()
    }
}

fn clone_helper<'a, T: 'a, U: Primitives<T> + 'a>(
    term: &Term<T, U, Bumpalo<'a>>,
    alloc: &Bumpalo<'a>,
) -> Term<T, U, Bumpalo<'a>>
where
    T: Clone,
    U: Clone,
{
    use Term::*;

    match term {
        Variable(index) => Term::Variable(index.clone()),
        Lambda { body, erased } => Term::Lambda {
            body: alloc.copy_boxed(body),
            erased: *erased,
        },
        Apply {
            function,
            argument,
            erased,
        } => Term::Apply {
            function: alloc.copy_boxed(function),
            argument: alloc.copy_boxed(argument),
            erased: *erased,
        },
        Put(term) => Term::Put(alloc.copy_boxed(term)),
        Duplicate { expression, body } => Term::Duplicate {
            expression: alloc.copy_boxed(expression),
            body: alloc.copy_boxed(body),
        },
        Reference(reference) => Term::Reference(reference.clone()),
        Primitive(prim) => Term::Primitive(prim.clone()),
        Term::Universe => Term::Universe,
        Term::Function {
            argument_type,
            return_type,
            erased,
        } => Term::Function {
            erased: *erased,
            argument_type: alloc.copy_boxed(argument_type),
            return_type: alloc.copy_boxed(return_type),
        },
        Term::Annotation {
            checked,
            expression,
            ty,
        } => Term::Annotation {
            checked: *checked,
            expression: alloc.copy_boxed(expression),
            ty: alloc.copy_boxed(ty),
        },
        Term::Wrap(term) => Term::Wrap(alloc.copy_boxed(term)),
    }
}

impl<'a, T: 'a, U: Primitives<T> + 'a> Allocator<T, U> for Bumpalo<'a> {
    type Box = BumpBox<'a, Term<T, U, Self>>;

    fn copy(&self, data: &Term<T, U, Self>) -> Term<T, U, Self>
    where
        T: Clone,
        U: Clone,
    {
        clone_helper(data, self)
    }

    fn copy_boxed(&self, data: &Self::Box) -> Self::Box
    where
        T: Clone,
        U: Clone,
    {
        self.alloc(self.copy(data))
    }

    fn alloc(&self, data: Term<T, U, Self>) -> Self::Box {
        BumpBox(bumpalo::boxed::Box::new_in(Some(data), self.0))
    }
}

impl<'a, T: 'a, U: Primitives<T> + 'a> Reallocate<T, U, System> for Bumpalo<'a> {
    fn reallocate_boxed(&self, data: <System as Allocator<T, U>>::Box) -> Self::Box {
        self.alloc(match data.into_inner() {
            Term::Variable(data) => Term::Variable(data),
            Term::Lambda { body, erased } => Term::Lambda {
                body: <Self as Reallocate<T, U, System>>::reallocate_boxed(self, body),
                erased,
            },
            Term::Apply {
                function,
                argument,
                erased,
            } => Term::Apply {
                function: <Self as Reallocate<T, U, System>>::reallocate_boxed(self, function),
                argument: <Self as Reallocate<T, U, System>>::reallocate_boxed(self, argument),
                erased,
            },
            Term::Put(term) => Term::Put(<Self as Reallocate<T, U, System>>::reallocate_boxed(
                self, term,
            )),
            Term::Duplicate { expression, body } => Term::Duplicate {
                expression: <Self as Reallocate<T, U, System>>::reallocate_boxed(self, expression),
                body: <Self as Reallocate<T, U, System>>::reallocate_boxed(self, body),
            },
            Term::Reference(reference) => Term::Reference(reference),
            Term::Primitive(prim) => Term::Primitive(prim),
            Term::Universe => Term::Universe,
            Term::Function {
                argument_type,
                return_type,
                erased,
            } => Term::Function {
                argument_type: <Self as Reallocate<T, U, System>>::reallocate_boxed(
                    self,
                    argument_type,
                ),
                return_type: <Self as Reallocate<T, U, System>>::reallocate_boxed(
                    self,
                    return_type,
                ),
                erased,
            },
            Term::Annotation {
                checked,
                expression,
                ty,
            } => Term::Annotation {
                checked,
                expression: <Self as Reallocate<T, U, System>>::reallocate_boxed(self, expression),
                ty: <Self as Reallocate<T, U, System>>::reallocate_boxed(self, ty),
            },
            Term::Wrap(term) => Term::Wrap(<Self as Reallocate<T, U, System>>::reallocate_boxed(
                self, term,
            )),
        })
    }

    fn reallocate(&self, data: Term<T, U, System>) -> Term<T, U, Self> {
        match data {
            Term::Variable(data) => Term::Variable(data),
            Term::Lambda { body, erased } => Term::Lambda {
                body: <Self as Reallocate<T, U, System>>::reallocate_boxed(self, body),
                erased,
            },
            Term::Apply {
                function,
                argument,
                erased,
            } => Term::Apply {
                function: <Self as Reallocate<T, U, System>>::reallocate_boxed(self, function),
                argument: <Self as Reallocate<T, U, System>>::reallocate_boxed(self, argument),
                erased,
            },
            Term::Put(term) => Term::Put(<Self as Reallocate<T, U, System>>::reallocate_boxed(
                self, term,
            )),
            Term::Duplicate { expression, body } => Term::Duplicate {
                expression: <Self as Reallocate<T, U, System>>::reallocate_boxed(self, expression),
                body: <Self as Reallocate<T, U, System>>::reallocate_boxed(self, body),
            },
            Term::Reference(reference) => Term::Reference(reference),
            Term::Primitive(prim) => Term::Primitive(prim),
            Term::Universe => Term::Universe,
            Term::Function {
                argument_type,
                return_type,
                erased,
            } => Term::Function {
                argument_type: <Self as Reallocate<T, U, System>>::reallocate_boxed(
                    self,
                    argument_type,
                ),
                return_type: <Self as Reallocate<T, U, System>>::reallocate_boxed(
                    self,
                    return_type,
                ),
                erased,
            },
            Term::Annotation {
                checked,
                expression,
                ty,
            } => Term::Annotation {
                checked,
                expression: <Self as Reallocate<T, U, System>>::reallocate_boxed(self, expression),
                ty: <Self as Reallocate<T, U, System>>::reallocate_boxed(self, ty),
            },
            Term::Wrap(term) => Term::Wrap(<Self as Reallocate<T, U, System>>::reallocate_boxed(
                self, term,
            )),
        }
    }

    fn reallocating_copy(&self, data: &Term<T, U, System>) -> Term<T, U, Self>
    where
        T: Clone,
        U: Clone,
    {
        match data {
            Term::Variable(data) => Term::Variable(data.clone()),
            Term::Lambda { body, erased } => Term::Lambda {
                body: self.alloc(<Self as Reallocate<T, U, System>>::reallocating_copy(
                    self, &*body,
                )),
                erased: *erased,
            },
            Term::Apply {
                function,
                argument,
                erased,
            } => Term::Apply {
                function: self.alloc(<Self as Reallocate<T, U, System>>::reallocating_copy(
                    self, &*function,
                )),
                argument: self.alloc(<Self as Reallocate<T, U, System>>::reallocating_copy(
                    self, &*argument,
                )),
                erased: *erased,
            },
            Term::Put(term) => Term::Put(self.alloc(
                <Self as Reallocate<T, U, System>>::reallocating_copy(self, &*term),
            )),
            Term::Duplicate { expression, body } => Term::Duplicate {
                expression: self.alloc(<Self as Reallocate<T, U, System>>::reallocating_copy(
                    self,
                    &*expression,
                )),
                body: self.alloc(<Self as Reallocate<T, U, System>>::reallocating_copy(
                    self, &*body,
                )),
            },
            Term::Reference(reference) => Term::Reference(reference.clone()),
            Term::Primitive(prim) => Term::Primitive(prim.clone()),
            Term::Universe => Term::Universe,
            Term::Function {
                argument_type,
                return_type,
                erased,
            } => Term::Function {
                argument_type: self.alloc(<Self as Reallocate<T, U, System>>::reallocating_copy(
                    self,
                    &*argument_type,
                )),
                return_type: self.alloc(<Self as Reallocate<T, U, System>>::reallocating_copy(
                    self,
                    &*return_type,
                )),
                erased: *erased,
            },
            Term::Annotation {
                checked,
                expression,
                ty,
            } => Term::Annotation {
                checked: *checked,
                expression: self.alloc(<Self as Reallocate<T, U, System>>::reallocating_copy(
                    self,
                    &*expression,
                )),
                ty: self.alloc(<Self as Reallocate<T, U, System>>::reallocating_copy(
                    self, &*ty,
                )),
            },
            Term::Wrap(term) => Term::Wrap(self.alloc(
                <Self as Reallocate<T, U, System>>::reallocating_copy(self, &*term),
            )),
        }
    }
}

impl<'a, 'b, T: 'a + 'b, U: Primitives<T> + 'a + 'b> Reallocate<T, U, Bumpalo<'b>> for Bumpalo<'a> {
    fn reallocate_boxed(&self, data: <Bumpalo<'b> as Allocator<T, U>>::Box) -> Self::Box {
        self.alloc(match data.into_inner() {
            Term::Variable(data) => Term::Variable(data),
            Term::Lambda { body, erased } => Term::Lambda {
                body: <Self as Reallocate<T, U, Bumpalo<'b>>>::reallocate_boxed(self, body),
                erased,
            },
            Term::Apply {
                function,
                argument,
                erased,
            } => Term::Apply {
                function: <Self as Reallocate<T, U, Bumpalo<'b>>>::reallocate_boxed(self, function),
                argument: <Self as Reallocate<T, U, Bumpalo<'b>>>::reallocate_boxed(self, argument),
                erased,
            },
            Term::Put(term) => Term::Put(
                <Self as Reallocate<T, U, Bumpalo<'b>>>::reallocate_boxed(self, term),
            ),
            Term::Duplicate { expression, body } => Term::Duplicate {
                expression: <Self as Reallocate<T, U, Bumpalo<'b>>>::reallocate_boxed(
                    self, expression,
                ),
                body: <Self as Reallocate<T, U, Bumpalo<'b>>>::reallocate_boxed(self, body),
            },
            Term::Reference(reference) => Term::Reference(reference),
            Term::Primitive(prim) => Term::Primitive(prim),
            Term::Universe => Term::Universe,
            Term::Function {
                argument_type,
                return_type,
                erased,
            } => Term::Function {
                argument_type: <Self as Reallocate<T, U, Bumpalo<'b>>>::reallocate_boxed(
                    self,
                    argument_type,
                ),
                return_type: <Self as Reallocate<T, U, Bumpalo<'b>>>::reallocate_boxed(
                    self,
                    return_type,
                ),
                erased,
            },
            Term::Annotation {
                checked,
                expression,
                ty,
            } => Term::Annotation {
                checked,
                expression: <Self as Reallocate<T, U, Bumpalo<'b>>>::reallocate_boxed(
                    self, expression,
                ),
                ty: <Self as Reallocate<T, U, Bumpalo<'b>>>::reallocate_boxed(self, ty),
            },
            Term::Wrap(term) => Term::Wrap(
                <Self as Reallocate<T, U, Bumpalo<'b>>>::reallocate_boxed(self, term),
            ),
        })
    }

    fn reallocate(&self, data: Term<T, U, Bumpalo<'b>>) -> Term<T, U, Self> {
        match data {
            Term::Variable(data) => Term::Variable(data),
            Term::Lambda { body, erased } => Term::Lambda {
                body: <Self as Reallocate<T, U, Bumpalo<'b>>>::reallocate_boxed(self, body),
                erased,
            },
            Term::Apply {
                function,
                argument,
                erased,
            } => Term::Apply {
                function: <Self as Reallocate<T, U, Bumpalo<'b>>>::reallocate_boxed(self, function),
                argument: <Self as Reallocate<T, U, Bumpalo<'b>>>::reallocate_boxed(self, argument),
                erased,
            },
            Term::Put(term) => Term::Put(
                <Self as Reallocate<T, U, Bumpalo<'b>>>::reallocate_boxed(self, term),
            ),
            Term::Duplicate { expression, body } => Term::Duplicate {
                expression: <Self as Reallocate<T, U, Bumpalo<'b>>>::reallocate_boxed(
                    self, expression,
                ),
                body: <Self as Reallocate<T, U, Bumpalo<'b>>>::reallocate_boxed(self, body),
            },
            Term::Reference(reference) => Term::Reference(reference),
            Term::Primitive(prim) => Term::Primitive(prim),
            Term::Universe => Term::Universe,
            Term::Function {
                argument_type,
                return_type,
                erased,
            } => Term::Function {
                argument_type: <Self as Reallocate<T, U, Bumpalo<'b>>>::reallocate_boxed(
                    self,
                    argument_type,
                ),
                return_type: <Self as Reallocate<T, U, Bumpalo<'b>>>::reallocate_boxed(
                    self,
                    return_type,
                ),
                erased,
            },
            Term::Annotation {
                checked,
                expression,
                ty,
            } => Term::Annotation {
                checked,
                expression: <Self as Reallocate<T, U, Bumpalo<'b>>>::reallocate_boxed(
                    self, expression,
                ),
                ty: <Self as Reallocate<T, U, Bumpalo<'b>>>::reallocate_boxed(self, ty),
            },
            Term::Wrap(term) => Term::Wrap(
                <Self as Reallocate<T, U, Bumpalo<'b>>>::reallocate_boxed(self, term),
            ),
        }
    }

    fn reallocating_copy(&self, data: &Term<T, U, Bumpalo<'b>>) -> Term<T, U, Self>
    where
        T: Clone,
        U: Clone,
    {
        match data {
            Term::Variable(data) => Term::Variable(data.clone()),
            Term::Lambda { body, erased } => Term::Lambda {
                body: self.alloc(<Self as Reallocate<T, U, _>>::reallocating_copy(
                    self, &*body,
                )),
                erased: *erased,
            },
            Term::Apply {
                function,
                argument,
                erased,
            } => Term::Apply {
                function: self.alloc(<Self as Reallocate<T, U, _>>::reallocating_copy(
                    self, &*function,
                )),
                argument: self.alloc(<Self as Reallocate<T, U, _>>::reallocating_copy(
                    self, &*argument,
                )),
                erased: *erased,
            },
            Term::Put(term) => Term::Put(self.alloc(
                <Self as Reallocate<T, U, _>>::reallocating_copy(self, &*term),
            )),
            Term::Duplicate { expression, body } => Term::Duplicate {
                expression: self.alloc(<Self as Reallocate<T, U, _>>::reallocating_copy(
                    self,
                    &*expression,
                )),
                body: self.alloc(<Self as Reallocate<T, U, _>>::reallocating_copy(
                    self, &*body,
                )),
            },
            Term::Reference(reference) => Term::Reference(reference.clone()),
            Term::Primitive(prim) => Term::Primitive(prim.clone()),
            Term::Universe => Term::Universe,
            Term::Function {
                argument_type,
                return_type,
                erased,
            } => Term::Function {
                argument_type: self.alloc(<Self as Reallocate<T, U, _>>::reallocating_copy(
                    self,
                    &*argument_type,
                )),
                return_type: self.alloc(<Self as Reallocate<T, U, _>>::reallocating_copy(
                    self,
                    &*return_type,
                )),
                erased: *erased,
            },
            Term::Annotation {
                checked,
                expression,
                ty,
            } => Term::Annotation {
                checked: *checked,
                expression: self.alloc(<Self as Reallocate<T, U, _>>::reallocating_copy(
                    self,
                    &*expression,
                )),
                ty: self.alloc(<Self as Reallocate<T, U, _>>::reallocating_copy(self, &*ty)),
            },
            Term::Wrap(term) => Term::Wrap(self.alloc(
                <Self as Reallocate<T, U, _>>::reallocating_copy(self, &*term),
            )),
        }
    }
}
