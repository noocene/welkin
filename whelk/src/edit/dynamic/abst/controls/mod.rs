mod adt;
pub use adt::*;

use mincodec::MinCodec;

use super::{
    AbstractDynamic, Container, DynamicContext, FieldFocus, FieldRead, FieldSetColor,
    FieldTriggersAppend, FieldTriggersRemove, HasContainer, HasField, HasInitializedField,
    HasStatic, Static, VStack, Wrapper,
};

#[derive(MinCodec, Clone)]
pub enum ControlData {
    Adt(AdtData),
}

impl ControlData {
    pub fn to_control<T: DynamicContext + HasContainer<VStack> + ?Sized + 'static>(self) -> Box<dyn AbstractDynamic<T>>
        where
            <T as HasField<VStack>>::Field: Container,
            <<T as HasField<VStack>>::Field as Container>::Context: HasContainer<Wrapper>,
            <<<T as HasField<VStack>>::Field as Container>::Context as HasField<Wrapper>>::Field: Container,
            <<<<T as HasField<VStack>>::Field as Container>::Context as HasField<Wrapper>>::Field as Container>::Context: HasStatic + HasInitializedField<String>,
            <<<<<T as HasField<VStack>>::Field as Container>::Context as HasField<Wrapper>>::Field as Container>::Context as HasField<String>>::Field: FieldRead<Data = String> + FieldFocus + FieldSetColor + FieldTriggersRemove + FieldTriggersAppend,
            <<<<<T as HasField<VStack>>::Field as Container>::Context as HasField<Wrapper>>::Field as Container>::Context as HasField<Static>>::Field: FieldSetColor

    {
        match self {
            ControlData::Adt(data) => Box::new(Adt::from(data)),
        }
    }
}
