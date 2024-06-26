//
// Created by Justin Tunheim on 6/20/24
//

use std::io::Read;
use flate2::read::GzDecoder;

mod nbt;

pub const CODENAME: &str = "CAVE";
pub const ROOT_DIR: &str = "/Users/justin/Documents/rave/";
pub const SAVE_DIR: &str = "static/01/";

fn full_path() -> String {
    format!("{}{}", ROOT_DIR, SAVE_DIR)
}

fn main() {
    let Ok(raw_data) = std::fs::read(format!("{}level.dat", full_path())) else {
        panic!("Could not read level.dat");
    };

    let mut data = Vec::new();
    let Ok(_) = GzDecoder::new(&raw_data[..]).read_to_end(&mut data) else {
        panic!("Unable to decode data!");
    };

    let mut nbts = Vec::new();
    if let Err(e) = nbt::Parser::new(data).parse(&mut nbts) {
        println!("this is not working:\n\t{}", e);
        return;
    }

    println!("root: {}", nbts.first().unwrap());

    println!("done");
}
