use crate::edit::{
    dynamic::abst::{
        implementation::Root, AbstractDynamic, Color, DynamicContext, Field, FieldRead,
        FieldSetColor, FieldTriggersAppend, FieldTriggersRemove, HasField, HasInitializedField,
        HasStatic, Replace, Static,
    },
    zipper::{dynamic::Dynamic, Term},
};

use super::{CompressedString, ControlData, Literal};

pub struct StringLiteral<T: HasInitializedField<String> + HasStatic + ?Sized> {
    field: Option<<T as HasField<String>>::Field>,
    prefix: Option<<T as HasField<Static>>::Field>,
    b_prefix: Option<<T as HasField<Static>>::Field>,
    field_content: String,
}

impl<T: HasInitializedField<String> + HasStatic + ?Sized> StringLiteral<T> {
    pub fn new() -> Self {
        StringLiteral {
            field: None,
            prefix: None,
            b_prefix: None,
            field_content: "".into(),
        }
    }
}

impl<T: DynamicContext + HasStatic + Replace + HasInitializedField<String> + ?Sized>
    AbstractDynamic<T> for StringLiteral<T>
where
    <T as HasField<String>>::Field:
        FieldRead<Data = String> + FieldTriggersAppend + FieldTriggersRemove + FieldSetColor,
    <T as HasField<Static>>::Field: FieldSetColor,
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

        let prefix = self
            .b_prefix
            .get_or_insert_with(|| {
                let field = <T as HasField<Static>>::create_field(context, Static("String".into()));
                <T as HasField<Static>>::field(context, &field).set_color(Color::Reference);

                context.append_field_after(field.handle(), prefix);
                field
            })
            .handle();

        let field_content = self.field_content.clone();

        let f = self.field.get_or_insert_with(|| {
            let field = <T as HasField<String>>::create_field(context, Some(field_content));
            <T as HasField<String>>::field(context, &field).set_color(Color::Data);

            context.append_field_after(field.handle(), prefix);

            field
        });

        let field = <T as HasField<String>>::field(context, &*f);

        if let Some(data) = field.read() {
            self.field_content = data;
        }

        if field.trigger_remove() {
            context.replace(Term::Dynamic(Dynamic::new((), Root::new(Literal::new()))));
        }

        let field = <T as HasField<String>>::field(context, &*f);
    }

    fn expand(&self) -> Term<()> {
        let term = Term::Compressed(Box::new(CompressedString::new(self.field_content.clone())));
        term
    }

    fn encode(&self) -> ControlData {
        ControlData::StringLiteral(self.field_content.clone())
    }
}

impl<T: HasInitializedField<String> + HasStatic + ?Sized> From<String> for StringLiteral<T> {
    fn from(field_content: String) -> Self {
        StringLiteral {
            field_content,
            field: None,
            prefix: None,
            b_prefix: None,
        }
    }
}
