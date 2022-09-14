// Run this cli like this:
// cargo +nightly run --example list-files

#![feature(core_intrinsics)]

use std::fs;

fn print_type_of<T>(_: &T) {
    println!("{}", { std::intrinsics::type_name::<T>() });
}

fn list_files_by_reverse_modified(target: &str) -> Vec<fs::DirEntry> {
    let paths: fs::ReadDir = fs::read_dir(target).unwrap();
    for path in paths {
        println!("Name: {}", path.unwrap().path().display());
    }

    let paths: fs::ReadDir = fs::read_dir(target).unwrap();
    for path in paths {
        print_type_of(&path)
    }
    println!("modified");
    let paths: fs::ReadDir = fs::read_dir(target).unwrap();
    println!("{:?}", paths.count());

    let paths: fs::ReadDir = fs::read_dir(target).unwrap();
    let mut sorted = paths.filter_map(|e| e.ok()).collect::<Vec<fs::DirEntry>>();
    sorted.sort_by(|a, b| {
        b.metadata()
            .unwrap()
            .modified()
            .unwrap()
            .cmp(&a.metadata().unwrap().modified().unwrap())
    });
    // for path in sorted {
    //     println!("sorted Name: {}", path.path().display());
    // };
    return sorted;
}

fn vec_to_json(vec: Vec<fs::DirEntry>) -> Vec<u8> {
    let files = vec
        .into_iter()
        .map(|e| e.file_name().into_string().ok())
        .map(|s| String::from(s.unwrap()))
        .collect::<Vec<String>>();

    // https://gist.github.com/jimmychu0807/9a89355e642afad0d2aeda52e6ad2424
    format!("[\"{}\"]", files.join("\",\"")).as_bytes().to_vec()
}

fn main() {
    list_files_by_reverse_modified("./");
    let res = list_files_by_reverse_modified("./");
    println!("{:?}", vec_to_json(res))
}
