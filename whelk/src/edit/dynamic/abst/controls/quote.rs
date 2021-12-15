use crate::edit::{
    dynamic::abst::{
        implementation::Root, AbstractDynamic, DynamicContext, Field, FieldRead,
        FieldTriggersRemove, HasField, HasInitializedField, HasStatic, Replace, Static,
    },
    zipper::{dynamic::Dynamic, Term},
};

use super::{ControlData, Invoke};

pub struct Quote<T: HasInitializedField<Term<()>> + HasStatic + ?Sized> {
    field: Option<<T as HasField<Term<()>>>::Field>,
    prefix: Option<<T as HasField<Static>>::Field>,
    field_content: Term<()>,
}

impl<T: HasInitializedField<Term<()>> + HasStatic + ?Sized> Quote<T> {
    pub fn new() -> Self {
        Quote {
            field: None,
            prefix: None,
            field_content: Term::Hole(()),
        }
    }
}

impl<T: DynamicContext + HasStatic + Replace + HasInitializedField<Term<()>> + ?Sized>
    AbstractDynamic<T> for Quote<T>
where
    <T as HasField<Term<()>>>::Field: FieldRead<Data = Term<()>> + FieldTriggersRemove,
{
    fn render(&mut self, context: &mut T) {
        let prefix = self
            .prefix
            .get_or_insert_with(|| {
                let field = <T as HasField<Static>>::create_field(context, Static("~quote".into()));
                context.append_field(field.handle());
                field
            })
            .handle();

        let mut field_content = None;
        if self.field.is_none() {
            field_content = Some(self.field_content.clone());
        }

        let f = self.field.get_or_insert_with(|| {
            let field = <T as HasField<Term<()>>>::create_field(
                context,
                Some(field_content.take().unwrap()),
            );

            context.append_field_after(field.handle(), prefix);

            field
        });

        let field = <T as HasField<Term<()>>>::field(context, &*f);

        if let Some(data) = field.read() {
            self.field_content = data;
        }

        if field.trigger_remove() {
            context.replace(Term::Dynamic(Dynamic::new((), Root::new(Invoke::new()))));
        }
    }

    fn expand(&self) -> Term<()> {
        // TODO
        // w::Ast::try_from(self.field_content.clone()).unwrap_or(Term::Hole(()))
        Term::Hole(())
    }

    fn encode(&self) -> ControlData {
        ControlData::Quote(self.field_content.clone().into())
    }
}

impl<T: HasInitializedField<Term<()>> + HasStatic + ?Sized> From<Term<()>> for Quote<T> {
    fn from(term: Term<()>) -> Self {
        Quote {
            field: None,
            prefix: None,
            field_content: term,
        }
    }
}
