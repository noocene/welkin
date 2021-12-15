use crate::edit::{
    dynamic::abst::{
        implementation::Root, AbstractDynamic, Color, DynamicContext, Field, FieldRead,
        FieldSetColor, FieldTriggersAppend, FieldTriggersRemove, HasField, HasInitializedField,
        HasStatic, Replace, Static,
    },
    zipper::{dynamic::Dynamic, Term},
};

use super::{ControlData, Literal, Quote};

pub struct Invoke<T: HasInitializedField<String> + HasStatic + ?Sized> {
    field: Option<<T as HasField<String>>::Field>,
    prefix: Option<<T as HasField<Static>>::Field>,
    field_content: String,
}

impl<T: HasInitializedField<String> + HasStatic + ?Sized> Invoke<T> {
    pub fn new() -> Self {
        Invoke {
            field: None,
            prefix: None,
            field_content: "".into(),
        }
    }
}

impl<T: DynamicContext + HasStatic + Replace + HasInitializedField<String> + ?Sized>
    AbstractDynamic<T> for Invoke<T>
where
    <T as HasField<String>>::Field:
        FieldRead<Data = String> + FieldTriggersAppend + FieldTriggersRemove + FieldSetColor,
{
    fn render(&mut self, context: &mut T) {
        let prefix = self
            .prefix
            .get_or_insert_with(|| {
                let field = <T as HasField<Static>>::create_field(context, Static("~".into()));
                context.append_field(field.handle());
                field
            })
            .handle();

        let f = self.field.get_or_insert_with(|| {
            let field = <T as HasField<String>>::create_field(context, None);
            <T as HasField<String>>::field(context, &field).set_color(Color::Reference);

            context.append_field_after(field.handle(), prefix);

            field
        });

        let field = <T as HasField<String>>::field(context, &*f);

        if let Some(data) = field.read() {
            self.field_content = data;
        }

        if field.trigger_remove() {
            context.remove();
        }

        let field = <T as HasField<String>>::field(context, &*f);

        if field.trigger_append() {
            if let Some(term) = match self.field_content.as_str() {
                "lit" | "literal" => {
                    Some(Term::Dynamic(Dynamic::new((), Root::new(Literal::new()))))
                }
                "quote" => Some(Term::Dynamic(Dynamic::new((), Root::new(Quote::new())))),
                _ => None,
            } {
                context.replace(term);
            }
        }
    }

    fn expand(&self) -> Term<()> {
        Term::Hole(())
    }

    fn encode(&self) -> ControlData {
        ControlData::Invoke
    }
}
