use crate::{
    compiler::{AbsolutePath, Resolve},
    parser::{
        term::{Arm, Block, Match, Section},
        util::{BumpBox, BumpVec},
        Ident, Path, Term,
    },
};
use welkin_core::term::Term as CoreTerm;

use super::Compile;

impl<'a> Compile<AbsolutePath> for Match<'a> {
    type Relative = Path<'a>;
    type Absolute = AbsolutePath;
    type Unit = Ident<'a>;

    fn compile<R: Resolve<Self::Relative, Absolute = Self::Absolute, Unit = Self::Unit>>(
        self,
        resolver: R,
    ) -> CoreTerm<AbsolutePath> {
        let bump = self.expression.bump;

        let self_ident = Ident::from_str("~match-self-ty", bump);

        let motive = self.sections.first().and_then(|section| {
            if self.sections.len() == 1 {
                let mut descent_resolver = resolver.proceed();
                for index in self.indices.iter() {
                    descent_resolver = descent_resolver.descend(Some(index.clone()));
                }
                Some(
                    section
                        .ty
                        .clone()
                        .compile(descent_resolver.descend(Some(section.self_binding.clone()))),
                )
            } else {
                None
            }
        });

        let sections = self
            .sections
            .into_iter()
            .map(|section| {
                let ty = section.ty.clone();
                section.arms.into_iter().map(move |arm| (arm, ty.clone()))
            })
            .flatten()
            .collect::<Vec<_>>();

        let self_path = Path(BumpVec::unary_in(self_ident.clone(), bump));

        let mut descent_resolver = resolver.proceed();
        for index in self.indices.iter() {
            descent_resolver = descent_resolver.descend(Some(index.clone()));
        }

        let motive = motive.unwrap_or_else(|| {
            Match {
                indices: BumpVec::new_in(bump),
                expression: BumpBox::new_in(Term::Reference(self_path.clone()), bump),
                sections: BumpVec::unary_in(
                    Section {
                        self_binding: self_ident.clone(),
                        ty: Term::Universe,
                        arms: BumpVec::from_iterator(
                            sections.clone().into_iter().map(|(arm, ty)| Arm {
                                introductions: arm.introductions,
                                expression: ty,
                            }),
                            bump,
                        ),
                    },
                    bump,
                ),
            }
            .compile(descent_resolver.descend(Some(self_ident.clone())))
        });

        let mut term = Term::Application {
            function: self.expression,
            erased: true,
            arguments: BumpVec::unary_in(
                {
                    let mut arg = Term::Lambda {
                        argument: self_ident.clone(),
                        body: BumpBox::new_in(Term::Block(Block::AbsoluteCore(motive)), bump),
                        erased: false,
                    };
                    for index in self.indices {
                        arg = Term::Lambda {
                            argument: index,
                            body: BumpBox::new_in(arg, bump),
                            erased: false,
                        };
                    }
                    arg
                },
                bump,
            ),
        };

        for (arm, _) in sections.into_iter() {
            let mut expr = arm.expression;
            for (argument, erased) in arm.introductions.into_iter().rev() {
                expr = Term::Lambda {
                    argument,
                    erased,
                    body: BumpBox::new_in(expr, bump),
                };
            }
            term = Term::Application {
                function: BumpBox::new_in(term, bump),
                arguments: BumpVec::unary_in(expr, bump),
                erased: false,
            };
        }

        term.compile(resolver)
    }
}
