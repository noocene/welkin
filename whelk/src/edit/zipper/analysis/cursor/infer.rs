use welkin_core::term::EqualityCache;

use crate::edit::{
    zipper::{
        analysis::{AnalysisTerm, TypedDefinitions},
        Cursor,
    },
    UiSection,
};

impl Cursor<UiSection> {
    pub fn infer<T: TypedDefinitions<Option<UiSection>>>(
        &self,
        root_ty: AnalysisTerm<()>,
        defs: &T,
        cache: &mut impl EqualityCache,
    ) -> Option<AnalysisTerm<()>> {
        let annotation = self.annotation().annotation.clone();

        *annotation.borrow_mut() = None;

        let mut cursor = self.clone();

        while !cursor.is_top() {
            cursor = cursor.ascend();
        }

        let term: AnalysisTerm<Option<UiSection>> = cursor.into();

        let root_ty: AnalysisTerm<Option<UiSection>> = root_ty.map_annotation(&|_| None);

        let _ = term.check_in(
            &root_ty,
            defs,
            &mut |annotation, ty| {
                if let Some(annotation) = annotation {
                    let annotation = &annotation.annotation;
                    *annotation.borrow_mut() = Some(ty.clone().clear_annotation());
                }
            },
            &mut |_, _| {},
            cache,
        );

        let annotation = &*annotation.borrow();
        annotation.clone()
    }
}
