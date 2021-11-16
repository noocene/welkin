use crate::edit::zipper::Term;

use self::controls::ControlData;

pub mod controls;
pub mod implementation;

pub trait Field {
    type Handle;

    fn handle(&self) -> Self::Handle;
}

pub trait FieldRead: Field {
    type Data;
}

pub trait FieldFilter: Field {
    type Element;
}

pub trait FieldFocus: Field {}

pub trait FieldTriggersRemove: Field {}

pub trait FieldTriggersAppend: Field {}

#[derive(Clone)]
pub enum Color {
    Data,
    Reference,
    Binding,
    Hole,
    Type,
}

pub trait FieldSetColor: Field {}

pub trait FieldContext<T: Field> {
    fn read(&self) -> Option<T::Data>
    where
        T: FieldRead,
    {
        todo!()
    }

    fn set_filter(&self, predicate: Box<dyn Fn(T::Element) -> bool>)
    where
        T: FieldFilter,
    {
        todo!()
    }

    fn set_color(&mut self, color: Color)
    where
        T: FieldSetColor,
    {
        todo!()
    }

    fn trigger_remove(&self) -> bool
    where
        T: FieldTriggersRemove,
    {
        todo!()
    }

    fn trigger_append(&self) -> bool
    where
        T: FieldTriggersAppend,
    {
        todo!()
    }

    fn context(&mut self) -> &mut T::Context
    where
        T: Container,
    {
        todo!()
    }

    fn focus(&mut self)
    where
        T: FieldFocus,
    {
        todo!()
    }
}

pub trait DynamicContext {
    type Handle;

    fn remove_field(&mut self, field: Box<dyn Field<Handle = Self::Handle>>);
    fn append_field(&mut self, field: Self::Handle);
    fn append_field_after(&mut self, field: Self::Handle, after: Self::Handle);

    fn remove(&mut self);
}

pub trait Replace: DynamicContext {
    fn replace(&mut self, with: Term<()>);
}

pub struct Static(pub String);

pub trait HasStatic: HasField<Static, Initializer = Static> {}

pub trait Container: Field {
    type Context: DynamicContext;
}

pub trait HasContainer<T>: HasField<T, Initializer = ()>
where
    <Self as HasField<T>>::Field: Container,
{
}

pub struct VStack;

pub struct Wrapper;

pub struct Hole;

pub trait HasHole: HasField<Hole, Initializer = ()> {}

pub trait HasField<T>: DynamicContext {
    type Field: Field<Handle = Self::Handle>;
    type Initializer;

    fn create_field(&mut self, initializer: Self::Initializer) -> Self::Field;

    fn field(&mut self, field: &Self::Field) -> &mut dyn FieldContext<Self::Field>;
}

pub trait HasInitializedField<T>: HasField<T, Initializer = Option<T>> {}

pub trait AbstractDynamic<T: DynamicContext + ?Sized> {
    fn render(&mut self, context: &mut T);

    fn encode(&self) -> ControlData;

    fn expand(&self) -> Term<()>;
}

impl<T: DynamicContext + ?Sized> AbstractDynamic<T> for Box<dyn AbstractDynamic<T>> {
    fn render(&mut self, context: &mut T) {
        <dyn AbstractDynamic<T> as AbstractDynamic<T>>::render(&mut **self, context)
    }

    fn encode(&self) -> ControlData {
        <dyn AbstractDynamic<T> as AbstractDynamic<T>>::encode(&**self)
    }

    fn expand(&self) -> Term<()> {
        <dyn AbstractDynamic<T> as AbstractDynamic<T>>::expand(&**self)
    }
}
