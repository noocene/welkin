use welkin_core::term::Term as CoreTerm;

use std::fmt::Debug;

use crate::{
    compiler::{term::Compile as _, AbsolutePath, Resolve},
    parser::{Data, Ident, Path, Term},
};

use super::Compile;

impl Compile<AbsolutePath> for Data {
    type Relative = Path;
    type Absolute = AbsolutePath;
    type Unit = Ident;

    fn compile<R: Debug + Resolve<Path, Unit = Ident, Absolute = AbsolutePath>>(
        self,
        r: R,
    ) -> Vec<(AbsolutePath, CoreTerm<AbsolutePath>, CoreTerm<AbsolutePath>)> {
        let canonical_path = r.canonicalize(Path(vec![self.ident.clone()]));

        let mut declarations = vec![];

        let mut return_type = Box::new(CoreTerm::Universe);
        let mut ret_resolver = r.proceed();

        for (arg, _) in self.type_arguments.iter() {
            ret_resolver = ret_resolver.descend(None);
            ret_resolver = ret_resolver.descend(Some(arg.clone()));
        }

        for (_, ty) in self.type_arguments.iter().rev() {
            return_type = Box::new(CoreTerm::Function {
                erased: true,
                argument_type: Box::new(
                    ty.as_ref()
                        .cloned()
                        .unwrap_or(Term::Universe)
                        .compile(ret_resolver.proceed()),
                ),
                return_type,
            });
            ret_resolver = ret_resolver.ascend().ascend();
        }

        let mut resolver = r.proceed();

        for (arg, _) in self.type_arguments.iter() {
            resolver = resolver.descend(Some(arg.clone()));
        }

        let self_ident = Ident("~self".into());
        let prop_ident = Ident("~prop".into());

        let mut term = Box::new(CoreTerm::Function {
            erased: true,
            argument_type: Box::new(CoreTerm::Function {
                erased: false,
                argument_type: {
                    let mut ty = Box::new(CoreTerm::Reference(canonical_path.clone()));
                    for (arg, _) in &self.type_arguments {
                        ty = Box::new(CoreTerm::Apply {
                            erased: true,
                            function: ty,
                            argument: Box::new(CoreTerm::Variable(
                                resolver
                                    .resolve(&Path(vec![arg.clone()]))
                                    .unwrap()
                                    .unwrap_index(),
                            )),
                        });
                    }
                    ty
                },
                return_type: Box::new(CoreTerm::Universe),
            }),
            return_type: {
                resolver = resolver.descend(Some(self_ident.clone()));
                resolver = resolver.descend(Some(prop_ident.clone()));

                for variant in &self.variants {
                    resolver = resolver.descend(None).descend(Some(variant.ident.clone()));
                }
                let mut ty = Box::new(CoreTerm::Apply {
                    erased: false,
                    function: Box::new(CoreTerm::Variable(
                        resolver
                            .resolve(&Path(vec![prop_ident.clone()]))
                            .unwrap()
                            .unwrap_index(),
                    )),
                    argument: Box::new(CoreTerm::Variable(
                        resolver
                            .resolve(&Path(vec![self_ident.clone()]))
                            .unwrap()
                            .unwrap_index(),
                    )),
                });
                for variant in self.variants.iter().rev() {
                    resolver = resolver.ascend().ascend();

                    ty = Box::new(CoreTerm::Function {
                        return_type: ty,
                        erased: false,
                        argument_type: {
                            let mut variant_resolver = resolver.proceed();
                            for (inhabitant, _) in &variant.inhabitants {
                                variant_resolver = variant_resolver.descend(None);
                                variant_resolver =
                                    variant_resolver.descend(Some(inhabitant.clone()));
                            }

                            let mut ty = Box::new(CoreTerm::Apply {
                                erased: false,
                                function: Box::new(CoreTerm::Variable(
                                    variant_resolver
                                        .resolve(&Path(vec![prop_ident.clone()]))
                                        .unwrap()
                                        .unwrap_index(),
                                )),
                                argument: {
                                    let mut function =
                                        Box::new(CoreTerm::Reference(resolver.canonicalize(Path(
                                            vec![self.ident.clone(), variant.ident.clone()],
                                        ))));
                                    for (arg, _) in &self.type_arguments {
                                        function = Box::new(CoreTerm::Apply {
                                            function,
                                            erased: true,
                                            argument: Box::new(CoreTerm::Variable(
                                                variant_resolver
                                                    .resolve(&Path(vec![arg.clone()]))
                                                    .unwrap()
                                                    .unwrap_index(),
                                            )),
                                        })
                                    }
                                    for (ident, _) in &variant.inhabitants {
                                        function = Box::new(CoreTerm::Apply {
                                            argument: Box::new(CoreTerm::Variable(
                                                variant_resolver
                                                    .resolve(&Path(vec![ident.clone()]))
                                                    .unwrap()
                                                    .unwrap_index(),
                                            )),
                                            erased: false,
                                            function,
                                        })
                                    }
                                    function
                                },
                            });

                            let mut arg_resolver = resolver.proceed();
                            for (id, _) in &variant.inhabitants {
                                arg_resolver = arg_resolver.descend(None).descend(Some(id.clone()));
                            }

                            for (_, ity) in variant.inhabitants.iter().rev() {
                                arg_resolver = arg_resolver.ascend().ascend();
                                ty = Box::new(CoreTerm::Function {
                                    erased: false,
                                    argument_type: Box::new(
                                        ity.clone().compile(arg_resolver.proceed()),
                                    ),
                                    return_type: ty,
                                })
                            }

                            ty
                        },
                    });
                }
                ty
            },
        });

        for _ in &self.type_arguments {
            term = Box::new(CoreTerm::Lambda {
                erased: true,
                body: term,
            })
        }

        declarations.push((canonical_path.clone(), *return_type, *term));

        for variant in &self.variants {
            let mut resolver = r.proceed();

            let path = resolver.canonicalize(Path(vec![self.ident.clone(), variant.ident.clone()]));
            let mut ty = Box::new(CoreTerm::Reference(canonical_path.clone()));

            let mut t_resolver = resolver.proceed();

            for (arg, _) in &self.type_arguments {
                t_resolver = t_resolver.descend(None);
                t_resolver = t_resolver.descend(Some(arg.clone()));
            }

            for (arg, _) in &variant.inhabitants {
                t_resolver = t_resolver.descend(None);
                t_resolver = t_resolver.descend(Some(arg.clone()));
            }

            let mut ty_resolver = t_resolver.proceed();

            for (arg, _) in &self.type_arguments {
                ty = Box::new(CoreTerm::Apply {
                    erased: true,
                    function: ty,
                    argument: Box::new(CoreTerm::Variable(
                        ty_resolver
                            .resolve(&Path(vec![arg.clone()]))
                            .unwrap()
                            .unwrap_index(),
                    )),
                });
            }

            for (_, ity) in variant.inhabitants.iter().rev() {
                ty_resolver = ty_resolver.ascend().ascend();

                ty = Box::new(CoreTerm::Function {
                    erased: false,
                    return_type: ty,
                    argument_type: Box::new(ity.clone().compile(ty_resolver.proceed())),
                });
            }

            for (_, t) in self.type_arguments.iter().rev() {
                ty = Box::new(CoreTerm::Function {
                    erased: true,
                    return_type: ty,
                    argument_type: Box::new(
                        t.as_ref()
                            .cloned()
                            .unwrap_or(Term::Universe)
                            .compile(ty_resolver.proceed()),
                    ),
                });
            }

            for (arg, _) in &self.type_arguments {
                resolver = resolver.descend(Some(arg.clone()));
            }

            for (ident, _) in &variant.inhabitants {
                resolver = resolver.descend(Some(ident.clone()));
            }

            resolver = resolver.descend(Some(Ident("~prop".into())));

            for variant in &self.variants {
                resolver = resolver.descend(Some(variant.ident.clone()));
            }

            let mut term = Box::new(CoreTerm::Variable(
                resolver
                    .resolve(&Path(vec![variant.ident.clone()]))
                    .unwrap()
                    .unwrap_index(),
            ));

            for (ident, _) in &variant.inhabitants {
                term = Box::new(CoreTerm::Apply {
                    function: term,
                    erased: false,
                    argument: Box::new(CoreTerm::Variable(
                        resolver
                            .resolve(&Path(vec![ident.clone()]))
                            .unwrap()
                            .unwrap_index(),
                    )),
                });
            }

            for _ in self.variants.iter().rev() {
                term = Box::new(CoreTerm::Lambda {
                    erased: false,
                    body: term,
                });
            }

            term = Box::new(CoreTerm::Lambda {
                erased: true,
                body: term,
            });

            for _ in &variant.inhabitants {
                term = Box::new(CoreTerm::Lambda {
                    erased: false,
                    body: term,
                })
            }

            for _ in &self.type_arguments {
                term = Box::new(CoreTerm::Lambda {
                    erased: true,
                    body: term,
                })
            }

            declarations.push((path, *ty, *term));
        }

        declarations
    }
}
