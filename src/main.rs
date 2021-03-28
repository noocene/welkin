use std::{fs::read_to_string, path::Component};

use combine::{stream::position, EasyParser};
use welkin::parser::items;

use walkdir::WalkDir;

fn main() {
    // let data = read_to_string("test.wc").unwrap();
    // let mut data = data.split('%');
    // let ty: Term<String> = data.next().unwrap().parse().unwrap();
    // let term: Term<String> = data.next().unwrap().parse().unwrap();
    // term.check(
    //     &ty,
    //     &Single("This".into(), (Term::Universe, Term::Universe)),
    // )
    // .unwrap();

    for entry in WalkDir::new(std::env::args().skip(1).next().unwrap())
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
        let ident = name.last().unwrap().clone();
        if name.len() > 1 && name.last().unwrap() == &name[name.len() - 2] {
            name.pop();
        }
        let name = name.join(".");
        let data = read_to_string(entry).unwrap();
        let data = position::Stream::new(data.as_str());
        let (items, remainder) = items().easy_parse(data).unwrap_or_else(|e| {
            println!("{}", e);
            panic!()
        });
        println!("{:?}", items);
        break;
    }
}
