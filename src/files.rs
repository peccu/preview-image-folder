use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;

fn list_files_by_reverse_modified(target: &str) -> Vec<fs::DirEntry> {
    let paths: fs::ReadDir = fs::read_dir(target).unwrap();
    let mut sorted = paths.filter_map(|e| e.ok()).collect::<Vec<fs::DirEntry>>();
    sorted.sort_by(|a, b| {
        b.metadata()
            .unwrap()
            .modified()
            .unwrap()
            .cmp(&a.metadata().unwrap().modified().unwrap())
    });
    sorted
}

fn vec_to_json(vec: Vec<fs::DirEntry>) -> Vec<u8> {
    let entries = vec
        .into_iter()
        .map(|e| e.file_name().into_string().ok())
        .map(|s| String::from(s.unwrap()))
        .collect::<Vec<String>>();

    // https://gist.github.com/jimmychu0807/9a89355e642afad0d2aeda52e6ad2424
    format!("[\"{}\"]", entries.join("\",\""))
        .as_bytes()
        .to_vec()
}

pub fn list_images(target: &str) -> Vec<u8> {
    vec_to_json(list_files_by_reverse_modified(target))
}

fn _read_file(name: &str) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(name)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

pub fn read_file(name: &str) -> Vec<u8> {
    let path = env::current_dir();
    println!("pwd: {:?} -> {:?}", path, name);
    match _read_file(name) {
        Ok(buf) => buf,
        _ => b"Error".to_vec(),
    }
}
