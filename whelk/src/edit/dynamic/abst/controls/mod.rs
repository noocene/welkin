mod adt;
pub use adt::*;
mod invoke;
pub use invoke::*;

use mincodec::MinCodec;

use super::{
    AbstractDynamic, Container, DynamicContext, FieldFocus, FieldRead, FieldSetColor,
    FieldTriggersAppend, FieldTriggersRemove, HasContainer, HasField, HasInitializedField,
    HasStatic, Static, VStack, Wrapper,
};

#[derive(MinCodec, Clone)]
pub enum ControlData {
    Adt(AdtData),
    Invoke,
}

impl ControlData {
    pub fn to_control<T: DynamicContext + HasStatic + HasContainer<VStack> + HasInitializedField<String> + ?Sized + 'static>(self) -> Box<dyn AbstractDynamic<T>>
        where
            <T as HasField<String>>::Field: FieldRead<Data = String> + FieldSetColor + FieldTriggersRemove,
            <T as HasField<VStack>>::Field: Container,
            <<T as HasField<VStack>>::Field as Container>::Context: HasContainer<Wrapper>,
            <<<T as HasField<VStack>>::Field as Container>::Context as HasField<Wrapper>>::Field: Container,
            <<<<T as HasField<VStack>>::Field as Container>::Context as HasField<Wrapper>>::Field as Container>::Context: HasStatic + HasInitializedField<String>,
            <<<<<T as HasField<VStack>>::Field as Container>::Context as HasField<Wrapper>>::Field as Container>::Context as HasField<String>>::Field: FieldRead<Data = String> + FieldFocus + FieldSetColor + FieldTriggersRemove + FieldTriggersAppend,
            <<<<<T as HasField<VStack>>::Field as Container>::Context as HasField<Wrapper>>::Field as Container>::Context as HasField<Static>>::Field: FieldSetColor

    {
        match self {
            ControlData::Adt(data) => Box::new(Adt::from(data)),
            ControlData::Invoke => Box::new(Invoke::new()),
        }
    }
}
