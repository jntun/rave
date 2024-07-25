//
// Created by Justin Tunheim on 7/23/24
//

use crate::{nbt, config};

pub enum Error {
    Search(nbt::query::Error),
}

type FileList = Option<Vec<String>>;

pub struct RegionManager {
    overworld: FileList,
    the_end:   FileList,
    nether:    FileList,
}

struct Engine {
}

pub fn run(config: config::Configuration) -> Result<(), Error> {
    let mut engine = Engine::new(&config);

    match config.command.value().unwrap() {
        config::Command::List(scope) => todo!(),
        config::Command::Search(method) => engine.search(config)?,
    };

    Ok(())
}

impl Engine {
    fn search(&mut self, config: config::Configuration) -> Result<(), Error> {
        Ok(())
    }
}

impl Engine {
    pub fn new(config: &config::Configuration) -> Self {
        Self {}
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "error")
    }
}
