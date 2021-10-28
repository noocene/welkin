use crate::edit::{
    dynamic::abst::{
        AbstractDynamic, Color, DynamicContext, Field, FieldRead, FieldSetColor,
        FieldTriggersRemove, HasField, HasInitializedField,
    },
    zipper::Term,
};
use mincodec::MinCodec;

use super::ControlData;

#[derive(MinCodec, Clone)]
pub struct InvokeData {
    field_content: String,
}

pub struct Invoke<T: DynamicContext + HasField<String> + ?Sized> {
    field: Option<<T as HasField<String>>::Field>,
    data: InvokeData,
}

impl<T: DynamicContext + HasInitializedField<String> + ?Sized> AbstractDynamic<T> for Invoke<T>
where
    T::Field: FieldRead<Data = String> + FieldSetColor + FieldTriggersRemove,
{
    fn render(&mut self, context: &mut T) {
        let field_content = self.data.field_content.clone();

        let field = self.field.get_or_insert_with(|| {
            let field = context.create_field(Some(field_content));
            context.field(&field).set_color(Color::Reference);
            context.append_field(field.handle());
            field
        });

        let field = context.field(&*field);

        if let Some(data) = field.read() {
            self.data.field_content = data;
        }

        if field.trigger_remove() {
            context.remove();
        }
    }

    fn expand(&self) -> Term<()> {
        Term::Reference(self.data.field_content.clone(), ())
    }

    fn encode(&self) -> ControlData {
        ControlData::Invoke(self.data.clone())
    }
}

impl<T: DynamicContext + HasField<String> + ?Sized> Invoke<T> {
    pub fn new() -> Self {
        Self {
            field: None,
            data: InvokeData {
                field_content: "".into(),
            },
        }
    }
}

impl<T: DynamicContext + HasField<String> + ?Sized> From<InvokeData> for Invoke<T> {
    fn from(data: InvokeData) -> Self {
        Self { field: None, data }
    }
}
