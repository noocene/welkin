use crate::edit::{
    dynamic::abst::{
        implementation::Root, AbstractDynamic, Color, DynamicContext, Field, FieldFilter,
        FieldRead, FieldSetColor, FieldTriggersAppend, FieldTriggersRemove, HasField,
        HasInitializedField, HasStatic, Replace, Static,
    },
    zipper::{dynamic::Dynamic, Term},
};

use super::{CompressedSize, ControlData, Literal};

pub struct SizeLiteral<T: HasInitializedField<String> + HasStatic + ?Sized> {
    field: Option<<T as HasField<String>>::Field>,
    prefix: Option<<T as HasField<Static>>::Field>,
    b_prefix: Option<<T as HasField<Static>>::Field>,
    field_content: usize,
}

impl<T: HasInitializedField<String> + HasStatic + ?Sized> SizeLiteral<T> {
    pub fn new() -> Self {
        SizeLiteral {
            field: None,
            prefix: None,
            b_prefix: None,
            field_content: 0,
        }
    }
}

impl<T: DynamicContext + HasStatic + Replace + HasInitializedField<String> + ?Sized>
    AbstractDynamic<T> for SizeLiteral<T>
where
    <T as HasField<String>>::Field: FieldRead<Data = String>
        + FieldTriggersAppend
        + FieldFilter<Element = char>
        + FieldTriggersRemove
        + FieldSetColor,
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
                let field = <T as HasField<Static>>::create_field(context, Static("Size".into()));
                <T as HasField<Static>>::field(context, &field).set_color(Color::Reference);

                context.append_field_after(field.handle(), prefix);
                field
            })
            .handle();

        let field_content = self.field_content.clone();

        let f = self.field.get_or_insert_with(|| {
            let field =
                <T as HasField<String>>::create_field(context, Some(format!("{}", field_content)));
            {
                let field = <T as HasField<String>>::field(context, &field);
                field.set_color(Color::Data);
                field.set_filter(Box::new(|data| "0123456789".contains(data)));
            }

            context.append_field_after(field.handle(), prefix);

            field
        });

        let field = <T as HasField<String>>::field(context, &*f);

        if let Some(data) = field.read() {
            self.field_content = data.parse().unwrap_or(0);
        }

        if field.trigger_remove() {
            context.replace(Term::Dynamic(Dynamic::new((), Root::new(Literal::new()))));
        }

        let field = <T as HasField<String>>::field(context, &*f);
    }

    fn expand(&self) -> Term<()> {
        let term = Term::Compressed(Box::new(CompressedSize::new(self.field_content)));
        term
    }

    fn encode(&self) -> ControlData {
        ControlData::SizeLiteral(self.field_content)
    }
}

impl<T: HasInitializedField<String> + HasStatic + ?Sized> From<usize> for SizeLiteral<T> {
    fn from(field_content: usize) -> Self {
        SizeLiteral {
            field_content,
            field: None,
            prefix: None,
            b_prefix: None,
        }
    }
}
