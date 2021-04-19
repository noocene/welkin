use std::{collections::HashMap, fs::read_to_string, path::Component, process::exit};

use combine::{stream::position, EasyParser};
use welkin::{
    compiler::{item::Compile as _, term::Compile as _, AbsolutePath, LocalResolver, Resolve},
    parser::{items, BlockItem, Ident, Item, Path},
};

use walkdir::WalkDir;
use welkin_core::term::{Term, TypedDefinitions};

fn format_size(term: Term<AbsolutePath>) -> String {
    if let Term::Lambda { body, .. } = term {
        if let Term::Lambda { body, .. } = *body {
            let mut term = body;
            let mut ctr = 0;
            while let Term::Apply { argument, .. } = *term {
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
        let (items, remainder) = items().easy_parse(data).unwrap_or_else(|e| {
            println!("{}in {}", e, hr_name);
            exit(1)
        });
        if !remainder.input.is_empty() {
            eprintln!(
                "PARSING ENDED BEFORE EOF IN {} WITH \n{}\nREMAINING",
                hr_name, remainder.input
            );
            exit(1);
        } else {
            for item in items {
                if let Item::Declaration(t) = item {
                    let ty = t.ty.compile(LocalResolver::new());
                    let term = t.term.compile(LocalResolver::new());
                    let name = if name.len() == 0 {
                        vec![t.ident.0]
                    } else {
                        name.clone()
                    };

                    declarations.push((
                        LocalResolver::new()
                            .canonicalize(Path(name.clone().into_iter().map(Ident).collect())),
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
        }
    }

    let defs: HashMap<_, _> = declarations
        .clone()
        .into_iter()
        .map(|(a, b, c)| (a, (b, c)))
        .collect();
    let mut ok = 0;
    let mut err = 0;
    let mut errs = String::new();
    for (path, (ty, term)) in &defs {
        let mut er = 0;
        if let Err(e) = term.is_stratified(&defs) {
            errs.push_str(&format!("{:?} IS NOT STRATIFIED\n\t{:?}\n", path, e));
            er = 1;
        }
        if ty.is_stratified(&defs).is_err() {
            errs.push_str(&format!("{:?} (TYPE) IS NOT STRATIFIED\n", path));
            er = 1;
        }

        if let Err(e) = term.check(&ty, &defs) {
            errs.push_str(&format!("\n{:?} ERR\n{:?}\n", path, e));
            er = 1;
        } else {
            if let Err(e) = ty.check(&Term::Universe, &defs) {
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
    println!("{}", errs);
    println!("CHECKED {}", ok + err);
    println!("{} OK", ok);
    println!("{} ERR", err);

    if err == 0 {
        println!("\nmain normalizes to:\n{}", {
            let (ty, term) = defs
                .get_typed(&AbsolutePath(vec!["main".into()]))
                .unwrap()
                .clone();
            let mut main = term.stratified(&defs).unwrap();
            main.normalize().unwrap();
            let main = main.into_inner();

            if ty
                .equivalent(&Term::Reference(AbsolutePath(vec!["Size".into()])), &defs)
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
