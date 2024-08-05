//
// Created by Justin Tunheim on 7/23/24
//

use crate::{nbt, config, region};

mod directory {
    use crate::config;

    pub(crate) type List = Vec<String>;

    pub(crate) struct WorldFileBundle {
        pub overworld: List,
        pub nether:    List,
        pub the_end:   List,
    }

    pub(crate) fn region_files(config: &config::Configuration) -> WorldFileBundle {
        let base = config.save_root.value().unwrap();
        let mut bundle = WorldFileBundle{
            overworld: Vec::new(), 
            nether: Vec::new(),
            the_end: Vec::new(),
        };
        bundle.overworld.push(format!("{}/region", base));
        bundle.nether.push(format!("{}/DIM-1/region", base));
        bundle.the_end.push(format!("{}/DIM1/region", base));
        bundle
    }
}

pub enum Error {
    Search(nbt::query::Error),
    ReadFile(std::io::Error),
    Region(region::Report),
    Command(String),
    Finding,
}

struct Gestalt {
}

pub fn run(config: config::Configuration) -> Result<(), Error> {
    let mut gestalt = Gestalt::new(&config);

    match config.command.value().unwrap() {
        config::Command::List(_) => todo!(),
        config::Command::Search(_) => gestalt.search(config)?,
    };

    Ok(())
}

impl Gestalt {
    fn fast_find(&mut self, method: &config::Method) -> Result<(), Error> {
        Ok(())
    }

    fn find_nbt(&mut self, config: &config::Configuration, file_list: directory::List) -> Result<(), Error> {
        println!("finding: {:?}", file_list);
        let method = match config.command.value().unwrap() {
            config::Command::Search(method) => method,
            _ => return Err(Error::Command(format!("Gestalt::find() should not be called on anything but a config::Command::Search"))), 
        };
        let Some(idx) = config.index.value() else {
            return self.fast_find(method);
        };
        match idx {
            config::Index::First => return self.fast_find(method),
            _ => (),
        }

        let mut results= Vec::new();
        for dir in file_list {
            let files = match std::fs::read_dir(dir) {
                Ok(f) => f,
                Err(e) => return Err(Error::ReadFile(e)),
            };
            for file in files {
                let buffer = match std::fs::read(file.unwrap().path()) {
                    Ok(b) => b,
                    Err(e) => return Err(Error::ReadFile(e)),
                };
                let region= match region::Parser::new(buffer).parse() {
                    Ok(r) => r,
                    Err(e) => return Err(Error::Region(e)),
                };
                for chunk in region {
                    let nbts = match method {
                        config::Method::Name(name) => {
                            match nbt::query::find_many_by_name(nbt::TAGString::from(name.clone()), chunk.nbt()) {
                                Ok(nbts) => nbts,
                                Err(e) => return Err(Error::Search(e)),
                            }
                        }
                    };

                    results.push(nbts);

                    match idx {
                        config::Index::Value(i) => {
                            if &results.len() == i {
                                break;
                            }
                        },
                        _ => (),
                    }
                }
            }
        }

        Ok(())
    }

    fn search(&mut self, config: config::Configuration) -> Result<(), Error> {
        let region_files = directory::region_files(&config);
        println!("{}\n{:?}", "starting...", config);
        self.find_nbt(&config, region_files.overworld)?;
        self.find_nbt(&config, region_files.nether)?;
        self.find_nbt(&config, region_files.the_end)?;
        println!("{}", "ending...");
        Ok(())
    }
}

impl Gestalt {
    pub fn new(config: &config::Configuration) -> Self {
        Self {}
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "error")
    }
}
