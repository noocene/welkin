use crate::{
    compiler::{AbsolutePath, Resolve},
    parser::{
        term::{Arm, Block, Match, Section},
        Ident, Path, Term,
    },
};
use welkin_core::term::Term as CoreTerm;

use super::Compile;

impl Compile<AbsolutePath> for Match {
    type Relative = Path;
    type Absolute = AbsolutePath;
    type Unit = Ident;

    fn compile<R: Resolve<Self::Relative, Absolute = Self::Absolute, Unit = Self::Unit>>(
        self,
        resolver: R,
    ) -> CoreTerm<AbsolutePath> {
        let self_ident = Ident("~match-self-ty".into());

        let motive = self.sections.first().and_then(|section| {
            if self.sections.len() == 1 {
                Some(section.ty.clone().compile(resolver.descend(None)))
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

        let self_path = Path(vec![self_ident.clone()]);

        let motive = motive.unwrap_or_else(|| {
            Match {
                expression: Box::new(Term::Reference(self_path.clone())),
                sections: vec![Section {
                    ty: Term::Universe,
                    arms: sections
                        .clone()
                        .into_iter()
                        .map(|(arm, ty)| Arm {
                            binding: arm.binding,
                            introductions: arm.introductions,
                            expression: ty,
                        })
                        .collect(),
                }],
            }
            .compile(resolver.descend(Some(self_ident.clone())))
        });

        let mut term = Term::Application {
            function: self.expression,
            erased: true,
            arguments: vec![Term::Lambda {
                argument: self_ident.clone(),
                body: Box::new(Term::Block(Block::AbsoluteCore(motive))),
                erased: false,
            }],
        };

        for (arm, _) in sections.into_iter() {
            let mut expr = arm.expression;
            for argument in arm.introductions.into_iter().rev() {
                expr = Term::Lambda {
                    argument,
                    erased: false,
                    body: Box::new(expr),
                };
            }
            term = Term::Application {
                function: Box::new(term),
                arguments: vec![expr],
                erased: false,
            };
        }

        term.compile(resolver)
    }
}
