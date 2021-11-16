use crate::edit::{
    dynamic::abst::{
        implementation::Root, AbstractDynamic, Color, DynamicContext, Field, FieldRead,
        FieldSetColor, FieldTriggersAppend, FieldTriggersRemove, HasField, HasInitializedField,
        HasStatic, Replace, Static,
    },
    zipper::{dynamic::Dynamic, Term},
};

use super::{ControlData, Invoke};

mod codegen;
mod size;
mod string;
pub use codegen::*;
pub use size::*;
pub use string::*;

pub struct Literal<T: HasInitializedField<String> + HasStatic + ?Sized> {
    field: Option<<T as HasField<String>>::Field>,
    prefix: Option<<T as HasField<Static>>::Field>,
    field_content: String,
}

impl<T: HasInitializedField<String> + HasStatic + ?Sized> Literal<T> {
    pub fn new() -> Self {
        Literal {
            field: None,
            prefix: None,
            field_content: "".into(),
        }
    }
}

impl<T: DynamicContext + HasStatic + Replace + HasInitializedField<String> + ?Sized>
    AbstractDynamic<T> for Literal<T>
where
    <T as HasField<String>>::Field:
        FieldRead<Data = String> + FieldTriggersAppend + FieldTriggersRemove + FieldSetColor,
{
    fn render(&mut self, context: &mut T) {
        let prefix = self
            .prefix
            .get_or_insert_with(|| {
                let field = <T as HasField<Static>>::create_field(context, Static("~lit".into()));
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
            context.replace(Term::Dynamic(Dynamic::new((), Root::new(Invoke::new()))));
        }

        let field = <T as HasField<String>>::field(context, &*f);

        if field.trigger_append() {
            if let Some(term) = match self.field_content.as_str() {
                "String" => Some(Term::Dynamic(Dynamic::new(
                    (),
                    Root::new(StringLiteral::new()),
                ))),
                "Size" => Some(Term::Dynamic(Dynamic::new(
                    (),
                    Root::new(SizeLiteral::new()),
                ))),
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
        ControlData::Literal
    }
}
