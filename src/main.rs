//
// Created by Justin Tunheim on 6/20/24
//

use std::io::Read;

mod nbt;
mod region;

const CODENAME: &str = "CAVE";
const ROOT_DIR: &str = "/Users/justin/Documents/rave/";
const WORKING_DIR: &str = "static/01/region/";

fn repeat_char(c: char, i: usize) -> String {
    let mut out = String::new();
    for _ in 0..i {
        out.push(c);
    }
    out
}

fn working_path() -> String {
    format!("{}{}", ROOT_DIR, WORKING_DIR)
}

fn main() {
    for entry in std::fs::read_dir(working_path()).unwrap() {
        let Ok(mca) = entry else {
            eprintln!("Internal error resolving directory entry in '{}'", working_path());
            return;
        };
        let path = String::from(mca.path().to_str().unwrap());
        let raw_data = match std::fs::read(path.clone()) {
            Ok(raw) => raw,
            Err(e) => {
                eprintln!("Could not open '{}':\n\t{}", working_path(), e);
                return;
            }
        };
        print!("{} doing '{}' {}", repeat_char('-', 10), path, repeat_char('-', 10));

        let region = match region::Parser::new(raw_data).parse() {
            Ok(region) => region,
            Err(e) => {
                eprintln!("Something went wrong:\n\t{}", e);
                return;
            }
        };
        println!(" - done");
    }

    println!("done");
}
