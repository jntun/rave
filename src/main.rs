//
// Created by Justin Tunheim on 6/20/24
//

use config::Configuration;

mod nbt;
mod region;
mod gestalt;
mod config;

use config::{Value, Scope, Command, Method};

const CODENAME: &str = "RAVE";

fn commands() -> String {
    format!("{}{}",
        "\n\t--root  | -r : Path to a Minecraft Java save",
        "\n\t--index | -i : Can be combined with commands that iterate / list to select a specific element"
    )
}

fn usage() -> String {
    format!("Usage: {} [OPTIONS] [COMMAND]\n\t{}", CODENAME.to_lowercase(), commands())
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
        command:   Value::Default(Command::List(Scope::All)),
        save_root: Value::Default(working_path().expect("compatible operating system.")),
        index:     Value::None,
    };

    let mut args = std::env::args().enumerate().skip(1);

    loop {
        let (i, arg) = match args.next() {
            Some((i, arg)) => (i, arg),
            None => break,
        };
        match arg.as_str() {
            "-i" | "--index" => {
                let Some(idx_iter) = args.next() else {
                    return println!("--index or -i argument requires a unsigned integer paramater e.g 'rave search \"minecraft:air\" --index 0");
                };
            },
            "-r" | "--root" => {
                let Some(dir) = args.next() else {
                    return println!("--root or -r argument requires a path parameter e.g 'rave --root ~/my/rave/save/'");
                };
                config.save_root = Value::User(dir.1);
            },
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
            "search" | "s" => {
                let Some(name) = args.next() else {
                    return println!("please provide the 'search' or 's' command with a name to search for. e.g 'rave search | s < name >'");
                };
                config.command = Value::User(Command::Search(Method::Name(name.1)));
            },
            "--help" => {
                println!("{}", usage());
                return;
            },
            _ => return println!("{}\n\tsupplied unknown argument {}.", usage(), arg),
        }
    }

    if let Err(e) = gestalt::run(config) {
        println!("Error operating on save:\n\n\t{}.", e); 
    }
}
