use crate::edit::{
    dynamic::abst::{
        AbstractDynamic, Color, DynamicContext, Field, FieldSetColor, HasField,
        HasInitializedField, HasStatic, Static,
    },
    zipper::Term,
};

use super::ControlData;

pub struct Variable<T: HasStatic + ?Sized> {
    prefix: Option<<T as HasField<Static>>::Field>,
    index: Option<<T as HasField<Static>>::Field>,
    content: usize,
}

impl<T: HasInitializedField<String> + HasStatic + ?Sized> Variable<T> {
    pub fn new(size: usize) -> Self {
        Variable {
            index: None,
            prefix: None,
            content: size,
        }
    }
}

impl<T: DynamicContext + HasStatic + ?Sized> AbstractDynamic<T> for Variable<T>
where
    <T as HasField<Static>>::Field: FieldSetColor,
{
    fn render(&mut self, context: &mut T) {
        let prefix = self
            .prefix
            .get_or_insert_with(|| {
                let field = <T as HasField<Static>>::create_field(context, Static("^".into()));
                context.append_field(field.handle());
                field
            })
            .handle();

        let content = self.content;

        self.index.get_or_insert_with(|| {
            let field =
                <T as HasField<Static>>::create_field(context, Static(format!("{}", content)));
            <T as HasField<Static>>::field(context, &field).set_color(Color::Binding);

            context.append_field_after(field.handle(), prefix);

            field
        });
    }

    fn expand(&self) -> Term<()> {
        Term::Hole(())
    }

    fn encode(&self) -> ControlData {
        ControlData::Variable(self.content)
    }
}
