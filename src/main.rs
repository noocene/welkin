use std::{
    collections::HashMap, fs::read_to_string, path::Component, process::exit, time::SystemTime,
};

use combine::{stream::position, EasyParser};
use welkin::{
    compiler::{
        item::Compile as _, term::Compile as _, AbsolutePath, BumpPath, LocalResolver, Resolve,
    },
    parser::{items, BlockItem, BumpString, BumpVec, Ident, Item, Path},
    Bumpalo,
};

use walkdir::WalkDir;
use welkin_core::term::{
    alloc::{Allocator, IntoInner, Reallocate},
    Primitives, Term, TypedDefinitions,
};

fn format_size<T, V: Primitives<T>, A: Allocator<T, V>>(term: Term<T, V, A>) -> String {
    if let Term::Lambda { body, .. } = term {
        if let Term::Lambda { body, .. } = body.into_inner() {
            let mut term = body;
            let mut ctr = 0;
            while let Term::Apply { argument, .. } = term.into_inner() {
                ctr += 1;
                term = argument;
            }
            return format!("SIZE = {}", ctr);
        }
        panic!()
    }
    panic!()
}

fn main() {
    let mut declarations = vec![];
    let bump = bumpalo::Bump::new();

    let mut parsing_time = 0;
    let mut tc_time = 0;
    let mut codegen_time = 0;

    for entry in WalkDir::new(std::env::args().skip(1).next().unwrap_or_else(|| {
        eprintln!("USAGE:\nwelkin <SOURCE_DIR>");
        exit(1)
    }))
    .into_iter()
    .skip(1)
    {
        let entry = entry.unwrap();
        if entry.file_type().is_dir() {
            continue;
        }
        let entry = entry.into_path();
        let mut name = entry
            .components()
            .skip(1)
            .map(|a| match a {
                Component::Normal(a) => a.to_string_lossy().split(".").next().unwrap().to_owned(),
                _ => panic!(),
            })
            .collect::<Vec<String>>();
        name = name
            .into_iter()
            .skip_while(|a| !a.chars().next().unwrap().is_uppercase())
            .collect();
        if name.len() > 1 && name.last().unwrap() == &name[name.len() - 2] {
            name.pop();
        }
        let hr_name = name.clone().join("::");
        let data = read_to_string(entry).unwrap();
        let data = data.trim();
        let data = data
            .split('\n')
            .filter(|a| !a.trim().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        let data = position::Stream::new(data.trim());
        let now = SystemTime::now();

        let (items, remainder) = items(&bump).easy_parse(data).unwrap_or_else(|e| {
            println!("{}in {}", e, hr_name);
            exit(1)
        });

        parsing_time += now.elapsed().unwrap().as_millis();
        if !remainder.input.is_empty() {
            eprintln!(
                "PARSING ENDED BEFORE EOF IN {} WITH \n{}\nREMAINING",
                hr_name, remainder.input
            );
            exit(1);
        } else {
            let now = SystemTime::now();
            for item in items {
                if let Item::Declaration(t) = item {
                    let ty = t.ty.compile(LocalResolver::new());
                    let term = t.term.compile(LocalResolver::new());
                    let name = if name.len() == 0 {
                        vec![t.ident.0]
                    } else {
                        name.clone()
                            .into_iter()
                            .map(|a| BumpString::from_str(&a, &bump))
                            .collect()
                    };

                    declarations.push((
                        LocalResolver::new().canonicalize(Path(BumpVec::from_iterator(
                            name.clone().into_iter().map(Ident),
                            &bump,
                        ))),
                        ty,
                        term,
                    ));
                } else if let Item::Block(block) = item {
                    #[allow(irrefutable_let_patterns)]
                    if let BlockItem::Data(data) = block {
                        let compiled = data.compile(LocalResolver::new());
                        declarations.extend(compiled);
                    }
                }
            }
            codegen_time += now.elapsed().unwrap().as_millis();
        }
    }

    let defs_bump = bumpalo::Bump::new();
    let defs_bm = Bumpalo(&defs_bump);

    let defs: HashMap<_, _> = declarations
        .clone()
        .into_iter()
        .map(|(a, b, c): (AbsolutePath, Term<_, _>, Term<_, _>)| {
            (
                BumpPath::new_in(a, &defs_bump),
                (
                    defs_bm.reallocate(
                        b.map_reference(|a| Term::Reference(BumpPath::new_in(a, &defs_bump))),
                    ),
                    defs_bm.reallocate(
                        c.map_reference(|a| Term::Reference(BumpPath::new_in(a, &defs_bump))),
                    ),
                ),
            )
        })
        .collect();

    let mut ok = 0;
    let mut err = 0;
    let mut errs = String::new();

    let now = SystemTime::now();

    for (path, (ty, term)) in &defs {
        let mut er = 0;

        let bp = bumpalo::Bump::new();
        let bm = Bumpalo(&bp);

        let ty = bm.reallocating_copy(ty);
        let term = bm.reallocating_copy(term);

        if let Err(e) = term.is_stratified() {
            errs.push_str(&format!("{:?} IS NOT STRATIFIED\n\t{:?}\n", path, e));
            er = 1;
        }
        if ty.is_stratified().is_err() {
            errs.push_str(&format!("{:?} (TYPE) IS NOT STRATIFIED\n", path));
            er = 1;
        }

        if let Err(e) = term.check_in(&ty, &defs, &bm) {
            errs.push_str(&format!("\n{:?} ERR\n{:?}\n", path, e));
            er = 1;
        } else {
            if let Err(e) = ty.check_in(&Term::Universe, &defs, &bm) {
                errs.push_str(&format!(
                    "\n{:?} ERR\n{:?}\nwhen checking {:?} in universe\n",
                    path, e, ty
                ));
                er = 1;
            } else {
                ok += if er != 0 { 0 } else { 1 };
            }
        }
        err += er;
    }
    tc_time += now.elapsed().unwrap().as_millis();

    println!("{}", errs);
    println!("CHECKED {}", ok + err);
    println!("{} OK", ok);
    println!("{} ERR", err);
    println!(
        "PARSING {}ms | CODEGEN {}ms | TC {}ms",
        parsing_time, codegen_time, tc_time
    );

    if err == 0 {
        println!("\nmain normalizes to:\n{}", {
            let data = defs
                .get_typed(&BumpPath::new_in(
                    AbsolutePath(vec!["main".into()]),
                    &defs_bump,
                ))
                .unwrap();
            let (ty, term) = data.as_ref();
            let ty = defs_bm.copy(ty);
            let term = defs_bm.copy(term);
            let mut main = term.stratified_in(&defs, &defs_bm).unwrap();
            main.normalize().unwrap();
            let main = main.into_inner();

            if ty
                .equivalent_in(
                    &Term::Reference(BumpPath::new_in(
                        AbsolutePath(vec!["Size".into()]),
                        &defs_bump,
                    )),
                    &defs,
                    &defs_bm,
                )
                .unwrap()
            {
                format_size(main)
            } else {
                format!("{:?}", main)
            }
        });
    } else {
        exit(1);
    }
}
