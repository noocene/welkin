use crate::edit::{
    dynamic::abst::{
        AbstractDynamic, Color, DynamicContext, Field, FieldRead, FieldSetColor,
        FieldTriggersRemove, HasField, HasInitializedField, HasStatic, Static,
    },
    zipper::Term,
};

use super::ControlData;

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

impl<T: DynamicContext + HasStatic + HasInitializedField<String> + ?Sized> AbstractDynamic<T>
    for Invoke<T>
where
    <T as HasField<String>>::Field: FieldRead<Data = String> + FieldTriggersRemove + FieldSetColor,
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

        let field = self.field.get_or_insert_with(|| {
            let field = <T as HasField<String>>::create_field(context, None);
            <T as HasField<String>>::field(context, &field).set_color(Color::Reference);

            context.append_field_after(field.handle(), prefix);

            field
        });

        let field = <T as HasField<String>>::field(context, &*field);

        if let Some(data) = field.read() {
            self.field_content = data;
        }

        if field.trigger_remove() {
            context.remove();
        }
    }

    fn expand(&self) -> Term<()> {
        Term::Hole(())
    }

    fn encode(&self) -> ControlData {
        ControlData::Invoke
    }
}
