use std::{
    collections::{HashMap, HashSet},
    fs::read_to_string,
    path::Component,
    process::exit,
    time::SystemTime,
};

use combine::{stream::position, EasyParser};
use welkin::{
    compiler::{item::Compile as _, term::Compile as _, BumpPath, LocalResolver, Resolve},
    Bumpalo,
};

use parser::{
    items, term::term as parse_term, AbsolutePath, BlockItem, BumpString, BumpVec, Ident, Item,
    Path,
};

use walkdir::WalkDir;
use welkin_core::{
    net::{Net, VisitNetExt},
    term::{
        alloc::{Allocator, IntoInner, Reallocate},
        Index, MapCache, Primitives, Term, TypedDefinitions,
    },
};

fn read_size<T, V: Primitives<T>, A: Allocator<T, V>>(term: Term<T, V, A>) -> String {
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

fn read_bool<T, V: Primitives<T>, A: Allocator<T, V>>(term: Term<T, V, A>) -> bool {
    if let Term::Lambda { body, .. } = term {
        if let Term::Lambda { body, .. } = body.into_inner() {
            if let Term::Variable(var) = body.into_inner() {
                var.0 == 1
            } else {
                panic!()
            }
        } else {
            panic!()
        }
    } else {
        panic!()
    }
}

fn read_word<T, V: Primitives<T>, A: Allocator<T, V>>(term: Term<T, V, A>) -> Vec<bool> {
    let mut data = vec![];
    let mut term = term;
    loop {
        while let Term::Lambda { body, .. } = term {
            term = body.into_inner();
        }
        match term {
            Term::Variable(_) => break data,
            Term::Apply {
                argument, function, ..
            } => {
                match function.into_inner() {
                    Term::Variable(Index(0)) => data.push(true),
                    Term::Variable(Index(1)) => data.push(false),
                    _ => panic!("invalid word"),
                };
                term = argument.into_inner();
            }
            _ => panic!("invalid word"),
        }
    }
}

fn read_char<T, V: Primitives<T>, A: Allocator<T, V>>(term: Term<T, V, A>) -> char {
    if let Term::Lambda { body, .. } = term {
        if let Term::Apply { argument, .. } = body.into_inner() {
            let bits = read_word(argument.into_inner());
            let mut bytes = [0u8; 4];

            for (bit, bits) in bits.as_slice().chunks(8).rev().enumerate() {
                let mut byte = 0u8;

                for idx in bits
                    .iter()
                    .enumerate()
                    .filter(|(_, bit)| **bit)
                    .map(|(idx, _)| idx)
                {
                    byte |= 1 << (7 - idx);
                }

                bytes[bit] = byte;
            }

            char::from_u32(u32::from_be_bytes(bytes)).unwrap()
        } else {
            panic!()
        }
    } else {
        panic!()
    }
}

fn read_vector<T, V: Primitives<T>, A: Allocator<T, V>, U>(
    term: Term<T, V, A>,
    read_element: impl Fn(Term<T, V, A>) -> U,
) -> Vec<U> {
    let mut data = vec![];
    let mut term = term;
    loop {
        while let Term::Lambda { body, .. } = term {
            term = body.into_inner();
        }
        match term {
            Term::Variable(_) => break data,
            Term::Apply {
                argument, function, ..
            } => {
                if let Term::Apply { argument, .. } = function.into_inner() {
                    data.push(read_element(argument.into_inner()));
                } else {
                    panic!()
                }
                term = argument.into_inner();
            }
            _ => panic!("invalid vector"),
        }
    }
}

fn read_string<T, V: Primitives<T>, A: Allocator<T, V>>(term: Term<T, V, A>) -> String {
    if let Term::Lambda { body, .. } = term {
        if let Term::Apply { argument, .. } = body.into_inner() {
            read_vector(argument.into_inner(), read_char)
                .into_iter()
                .collect()
        } else {
            panic!()
        }
    } else {
        panic!()
    }
}

fn read_sized<T, V: Primitives<T>, A: Allocator<T, V>, U>(
    term: Term<T, V, A>,
    read_element: impl Fn(Term<T, V, A>) -> U,
) -> U {
    if let Term::Lambda { body, .. } = term {
        if let Term::Apply { argument, .. } = body.into_inner() {
            read_element(argument.into_inner())
        } else {
            panic!()
        }
    } else {
        panic!()
    }
}

fn main() {
    let mut cache = MapCache::new();

    let mut declarations = vec![];
    let bump = bumpalo::Bump::new();

    let mut parsing_time = 0;
    let mut tc_times = vec![];
    let mut codegen_time = 0;

    let names = if let Ok(dump) = std::env::var("WELKIN_DUMP_NAMES") {
        dump.split(",").into_iter().map(String::from).collect()
    } else {
        HashSet::new()
    };

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
                    if names.contains(&hr_name) {
                        println!("NAME: {}\n{:?}\n{:?}\n", hr_name, ty, term);
                    }
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
                        if names.contains(&hr_name) {
                            for (_, ty, term) in &compiled {
                                println!("NAME: {}\n{:?}\n{:?}\n", hr_name, ty, term);
                            }
                        }
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

    let mut tc_time = 0;

    for (path, (ty, term)) in &defs {
        let now = SystemTime::now();

        let mut er = 0;

        let bp = bumpalo::Bump::new();
        let bm = Bumpalo(&bp);

        let ty = bm.reallocating_copy(ty);
        let term = bm.reallocating_copy(term);

        if term.is_recursive_in(&defs, &bm, &defs_bm) {
            errs.push_str(&format!("{:?} IS DEFINED RECURSIVELY\n", path));
            er = 1;
        }

        if ty.is_recursive_in(&defs, &bm, &defs_bm) {
            errs.push_str(&format!("{:?} (TYPE) IS DEFINED RECURSIVELY\n", path));
            er = 1;
        }

        if let Err(e) = term.is_stratified() {
            errs.push_str(&format!("{:?} IS NOT STRATIFIED\n\t{:?}\n", path, e));
            er = 1;
        }
        if ty.is_stratified().is_err() {
            errs.push_str(&format!("{:?} (TYPE) IS NOT STRATIFIED\n", path));
            er = 1;
        }

        if let Err(e) = term.check_in(&ty, &defs, &bm, &mut cache) {
            errs.push_str(&format!("\n{:?} ERR\n{:?}\n", path, e));
            er = 1;
        } else {
            if let Err(e) = ty.check_in(&Term::Universe, &defs, &bm, &mut cache) {
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
        let elapsed = now.elapsed().unwrap().as_millis();
        tc_times.push((path, elapsed));
        tc_times.sort_by_key(|(_, a)| *a);
        tc_times.reverse();
        tc_times.truncate(3);
        tc_time += elapsed;
    }

    println!("{}", errs);
    println!("CHECKED {}", ok + err);
    println!("{} OK", ok);
    println!("{} ERR", err);
    println!(
        "PARSING {}ms | CODEGEN {}ms | TC {}ms",
        parsing_time, codegen_time, tc_time
    );

    if tc_time > 200 {
        println!("\nTOP 3 TC:");
        for (name, time) in tc_times {
            println!(
                "{:width$} TOOK {}ms",
                {
                    let mut data = format!("{:?}", name);
                    let l = data.len();
                    data.truncate(22);
                    if l > 22 {
                        data.push_str("...");
                    }
                    data
                },
                time,
                width = 25
            );
        }
    }

    if err == 0 {
        println!("\nmain normalizes to:\n{}", {
            let name = BumpPath::new_in(AbsolutePath(vec!["main".into()]), &defs_bump);
            let data = defs.get_typed(&name).unwrap();
            let (ty, term) = data.as_ref();
            let mut ty = defs_bm.copy(ty);
            let term = defs_bm.copy(term);
            let main = term.stratified_in(&defs, &defs_bm).unwrap();
            let mut net = main.clone().into_net::<Net<u32>>().unwrap();
            net.reduce_all();
            let main: Term<String> = net.read_term(welkin_core::net::Index(0));

            if let Some(a) = std::env::args().nth(2) {
                if a == "--bundle" {
                    std::fs::write("./whelk/welkin/term", bincode::serialize(&main).unwrap())
                        .unwrap();
                    return;
                }
            }

            while let Term::Wrap(t) = ty {
                ty = t.into_inner();
            }

            let mut is_ty = |ty: &Term<_, _, _>, name: &str| {
                ty.equivalent_in(
                    &defs_bm.reallocating_copy(
                        &parse_term(Default::default(), &bump)
                            .easy_parse(name)
                            .unwrap()
                            .0
                            .compile(LocalResolver::new())
                            .map_reference(|a| Term::Reference(BumpPath::new_in(a, &defs_bump))),
                    ),
                    &defs,
                    &defs_bm,
                    &mut cache,
                )
                .unwrap()
            };

            if is_ty(&ty, "Size") {
                read_size(main)
            } else if is_ty(&ty, "Bool") {
                format!("BOOL = {:?}", read_bool(main))
            } else if is_ty(&ty, "Char") {
                format!("CHAR = {:?}", read_char(main))
            } else if is_ty(&ty, "Sized[String]") {
                format!("SIZED STRING = {:?}", read_sized(main, read_string))
            } else {
                format!("{:?}", main)
            }
        });
    } else {
        exit(1);
    }
}
