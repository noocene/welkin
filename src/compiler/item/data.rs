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

        let all_args = self
            .type_arguments
            .iter()
            .map(|(a, b, erased)| (a.clone(), b.clone(), *erased))
            .chain(
                self.indices
                    .iter()
                    .map(|(a, b)| (a.clone(), Some(b.clone()), true)),
            )
            .collect::<Vec<_>>();

        for (arg, _, _) in all_args.iter() {
            ret_resolver = ret_resolver.descend(None);
            ret_resolver = ret_resolver.descend(Some(arg.clone()));
        }

        for (_, ty, erased) in all_args.iter().rev() {
            ret_resolver = ret_resolver.ascend().ascend();

            let erased = *erased;

            return_type = Box::new(CoreTerm::Function {
                erased,
                argument_type: Box::new(
                    ty.clone()
                        .unwrap_or(Term::Universe)
                        .compile(ret_resolver.proceed()),
                ),
                return_type,
            });
        }

        let mut resolver = r.proceed();

        for (arg, _, _) in all_args.iter() {
            resolver = resolver.descend(Some(arg.clone()));
        }

        let self_ident = Ident("~self".into());
        let prop_ident = Ident("~prop".into());

        let mut term = Box::new(CoreTerm::Function {
            erased: true,
            argument_type: {
                let mut arg_resolver = resolver.proceed();
                for (index, _) in &self.indices {
                    arg_resolver = arg_resolver.descend(None).descend(Some(index.clone()));
                }
                let mut arg = Box::new(CoreTerm::Function {
                    erased: false,
                    argument_type: {
                        let mut ty = Box::new(CoreTerm::Reference(canonical_path.clone()));
                        for (arg, _, erased) in &all_args {
                            let erased = *erased;

                            ty = Box::new(CoreTerm::Apply {
                                erased,
                                function: ty,
                                argument: Box::new(CoreTerm::Variable(
                                    arg_resolver
                                        .resolve(&Path(vec![arg.clone()]))
                                        .unwrap()
                                        .unwrap_index(),
                                )),
                            });
                        }
                        ty
                    },
                    return_type: Box::new(CoreTerm::Universe),
                });
                for (_, ty) in &self.indices {
                    arg_resolver = arg_resolver.ascend().ascend();
                    arg = Box::new(CoreTerm::Function {
                        return_type: arg,
                        erased: true,
                        argument_type: Box::new(ty.clone().compile(arg_resolver.proceed())),
                    });
                }
                arg
            },
            return_type: {
                resolver = resolver.descend(Some(self_ident.clone()));
                resolver = resolver.descend(Some(prop_ident.clone()));

                for variant in &self.variants {
                    resolver = resolver.descend(None).descend(Some(variant.ident.clone()));
                }
                let mut ty = Box::new(CoreTerm::Apply {
                    erased: false,
                    function: {
                        let mut prop = Box::new(CoreTerm::Variable(
                            resolver
                                .resolve(&Path(vec![prop_ident.clone()]))
                                .unwrap()
                                .unwrap_index(),
                        ));
                        for (index, _) in &self.indices {
                            prop = Box::new(CoreTerm::Apply {
                                erased: true,
                                function: prop,
                                argument: Box::new(CoreTerm::Variable(
                                    resolver
                                        .resolve(&Path(vec![index.clone()]))
                                        .unwrap()
                                        .unwrap_index(),
                                )),
                            });
                        }
                        prop
                    },
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
                                function: {
                                    let mut prop = Box::new(CoreTerm::Variable(
                                        variant_resolver
                                            .resolve(&Path(vec![prop_ident.clone()]))
                                            .unwrap()
                                            .unwrap_index(),
                                    ));
                                    for index in &variant.indices {
                                        prop = Box::new(CoreTerm::Apply {
                                            erased: true,
                                            function: prop,
                                            argument: Box::new(
                                                index.clone().compile(variant_resolver.proceed()),
                                            ),
                                        });
                                    }
                                    prop
                                },
                                argument: {
                                    let mut function =
                                        Box::new(CoreTerm::Reference(resolver.canonicalize(Path(
                                            vec![self.ident.clone(), variant.ident.clone()],
                                        ))));
                                    for (arg, _, erased) in &self.type_arguments {
                                        let erased = *erased;

                                        function = Box::new(CoreTerm::Apply {
                                            function,
                                            erased,
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

        for (_, _, erased) in all_args.iter().rev() {
            let erased = *erased;

            term = Box::new(CoreTerm::Lambda { erased, body: term })
        }

        declarations.push((canonical_path.clone(), *return_type, *term));

        for variant in &self.variants {
            let mut resolver = r.proceed();

            let path = resolver.canonicalize(Path(vec![self.ident.clone(), variant.ident.clone()]));
            let mut ty = Box::new(CoreTerm::Reference(canonical_path.clone()));

            let mut t_resolver = resolver.proceed();

            for (arg, _, _) in &self.type_arguments {
                t_resolver = t_resolver.descend(None);
                t_resolver = t_resolver.descend(Some(arg.clone()));
            }

            for (arg, _) in &variant.inhabitants {
                t_resolver = t_resolver.descend(None);
                t_resolver = t_resolver.descend(Some(arg.clone()));
            }

            let mut ty_resolver = t_resolver.proceed();

            for (arg, _, erased) in self.type_arguments.iter() {
                let erased = *erased;

                ty = Box::new(CoreTerm::Apply {
                    erased,
                    function: ty,
                    argument: Box::new(CoreTerm::Variable(
                        ty_resolver
                            .resolve(&Path(vec![arg.clone()]))
                            .unwrap()
                            .unwrap_index(),
                    )),
                });
            }

            for index in &variant.indices {
                ty = Box::new(CoreTerm::Apply {
                    erased: true,
                    function: ty,
                    argument: Box::new(index.clone().compile(ty_resolver.proceed())),
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

            for (_, t, erased) in self.type_arguments.iter().rev() {
                ty_resolver = ty_resolver.ascend().ascend();

                let erased = *erased;

                ty = Box::new(CoreTerm::Function {
                    erased,
                    return_type: ty,
                    argument_type: Box::new(
                        t.as_ref()
                            .cloned()
                            .unwrap_or(Term::Universe)
                            .compile(ty_resolver.proceed()),
                    ),
                });
            }

            for (arg, _, _) in &self.type_arguments {
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

            for (_, _, erased) in self.type_arguments.iter().rev() {
                let erased = *erased;

                term = Box::new(CoreTerm::Lambda { erased, body: term })
            }

            declarations.push((path, *ty, *term));
        }

        declarations
    }
}
