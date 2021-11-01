use uuid::Uuid;

use crate::edit::dynamic::abst::{
    implementation::{
        fields::{RootContainerFieldContextData, RootFieldContext, RootFieldData},
        RootContext, RootHandle,
    },
    FieldContext, HasContainer, HasField, VStack,
};

use super::RootContainerField;

impl HasField<VStack> for RootContext {
    type Field = RootContainerField;

    type Initializer = ();

    fn create_field(&mut self, initializer: Self::Initializer) -> Self::Field {
        let sender = self.sender.clone().unwrap();

        let handle = Uuid::new_v4();

        let document = web_sys::window().unwrap().document().unwrap();
        let span = document.create_element("div").unwrap();

        span.class_list().add_2("abst-field", "vstack").unwrap();

        self.fields.insert(
            handle.clone(),
            RootFieldData::Container {
                context: RootFieldContext {
                    closures: vec![],
                    data: RootContainerFieldContextData {
                        element: span.clone(),
                        context: self.new_child(span),
                    },
                },
            },
        );

        RootContainerField(RootHandle(handle))
    }

    fn field(&mut self, field: &Self::Field) -> &mut dyn FieldContext<Self::Field> {
        let handle = &(field.0).0;

        let field = self.fields.get_mut(handle).unwrap();

        match field {
            RootFieldData::Container { context } => context,
            _ => panic!(),
        }
    }
}

impl HasContainer<VStack> for RootContext {}
