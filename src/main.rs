//
// Created by Justin Tunheim on 6/20/24
//

use config::Configuration;

mod nbt;
mod region;
mod config;

use config::{Value, Scope, Command};

const CODENAME: &str = "RAVE";

fn usage() -> String {
    format!("Usage: {} [OPTIONS] [COMMAND]", CODENAME.to_lowercase())
}

fn working_path() -> Option<String> {
    match std::env::consts::OS {
        "macos"   => Some(String::from("~/Library/Application Support/minecraft/saves")),
        "windows" => Some(String::from("%appdata%/.minecraft/saves/")),
        _         => None,
    }
}

fn main() {
    let mut config = Configuration {
        command: Value::Default(Command::List(Scope::All)),
        save_root: Value::Default(working_path().expect("compatible operating system.")),
    };

    let mut args = std::env::args().enumerate().skip(1);

    loop {
        let (i, arg) = match args.next() {
            Some((i, arg)) => (i, arg),
            None => break,
        };
        match arg.as_str() {
            "-r" | "--root" => {
                let Some(dir) = args.next() else {
                    return println!("--root or -r argument requires a path parameter e.g 'rave --root ~/my/rave/save/'");
                };
                config.save_root = Value::User(dir.1);
            }
            "list" | "l" => {
                let Some(peek) = std::env::args().nth(i+1) else {
                    continue;
                };
                match peek.as_str() {
                    "region" | "r" => {
                        let _ = args.next().expect("argument variables don't match? This shouldn't be possible, ever.");
                        config.command = Value::User(Command::List(Scope::Region));
                    },
                    _ => return println!("unrecognized parameter '{}' given to list command. e.g 'rave list < r | region >'", peek),
                }
            },
            _ => return println!("{}\n\tsupplied unknown argument {}.", usage(), arg),
        }
    }
}
