use crate::edit::{zipper::Term, UiSection};

#[derive(Clone, Debug)]
pub enum ReferenceMutation {
    Update(String),
    Focus,
    Remove,
}

#[derive(Clone, Debug)]
pub enum LambdaMutation {
    Update(String),
    Focus,
    Remove,
    ToggleErased,
}

#[derive(Clone, Debug)]
pub enum HoleMutation {
    Focus,
    Replace(Term<UiSection>),
    ToParent,
}

#[derive(Clone, Debug)]
pub enum ApplicationMutation {
    Focus,
    Remove,
    ToggleErased,
}

#[derive(Clone, Debug)]
pub enum UniverseMutation {
    Focus,
    Remove,
}

#[derive(Clone, Debug)]
pub enum WrapMutation {
    Focus,
    Remove,
}

#[derive(Clone, Debug)]
pub enum PutMutation {
    Focus,
    Remove,
}

#[derive(Clone, Debug)]
pub enum DuplicationMutation {
    Focus,
    Remove,
    Update(String),
}

#[derive(Clone, Debug)]
pub enum FunctionMutation {
    Focus,
    FocusSelf,
    Remove,
    Update(String),
    UpdateSelf(String),
    ToggleErased,
}
