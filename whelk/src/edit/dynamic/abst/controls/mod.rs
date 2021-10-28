mod invoke;
pub use invoke::*;

use mincodec::MinCodec;

use super::{AbstractDynamic, FieldRead, FieldSetColor, FieldTriggersRemove, HasInitializedField};

#[derive(MinCodec, Clone)]
pub enum ControlData {
    Invoke(InvokeData),
}

impl ControlData {
    pub fn to_control<T>(self) -> Box<dyn AbstractDynamic<T>>
    where
        T: HasInitializedField<String> + 'static,
        T::Field: FieldRead<Data = String> + FieldTriggersRemove + FieldSetColor,
    {
        match self {
            ControlData::Invoke(data) => Box::new(Invoke::from(data)),
        }
    }
}
