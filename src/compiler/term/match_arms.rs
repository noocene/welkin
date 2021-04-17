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
                let mut descent_resolver = resolver.proceed();
                for index in &self.indices {
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

        let self_path = Path(vec![self_ident.clone()]);

        let mut descent_resolver = resolver.proceed();
        for index in &self.indices {
            descent_resolver = descent_resolver.descend(Some(index.clone()));
        }

        let motive = motive.unwrap_or_else(|| {
            Match {
                indices: vec![],
                expression: Box::new(Term::Reference(self_path.clone())),
                sections: vec![Section {
                    self_binding: self_ident.clone(),
                    ty: Term::Universe,
                    arms: sections
                        .clone()
                        .into_iter()
                        .map(|(arm, ty)| Arm {
                            introductions: arm.introductions,
                            expression: ty,
                        })
                        .collect(),
                }],
            }
            .compile(descent_resolver.descend(Some(self_ident.clone())))
        });

        let mut term = Term::Application {
            function: self.expression,
            erased: true,
            arguments: vec![{
                let mut arg = Term::Lambda {
                    argument: self_ident.clone(),
                    body: Box::new(Term::Block(Block::AbsoluteCore(motive))),
                    erased: false,
                };
                for index in self.indices {
                    arg = Term::Lambda {
                        argument: index,
                        body: Box::new(arg),
                        erased: false,
                    };
                }
                arg
            }],
        };

        for (arm, _) in sections.into_iter() {
            let mut expr = arm.expression;
            for (argument, erased) in arm.introductions.into_iter().rev() {
                expr = Term::Lambda {
                    argument,
                    erased,
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
