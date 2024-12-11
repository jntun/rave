//
// Created by Justin Tunheim on 7/23/24
//

use crate::{nbt, config, region};

mod directory {
    use crate::config;

    pub(crate) type List = Vec<String>;

    pub(crate) struct WorldFile {
        pub overworld: List,
        pub nether:    List,
        pub the_end:   List,
    }

	pub(crate) fn region_files(config: &config::Configuration) -> WorldFile {
        let base = config.save_root.value().unwrap();
        let mut bundle = WorldFile{
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

fn plural(i: usize) -> char {
	if i == 1 {
		' '
	} else {
		's'
	}
}

pub fn run(config: config::Configuration) -> Result<(), Error> {
    let mut gestalt = Gestalt::new(&config);


    match config.command.value().unwrap() {
        config::Command::List(_)   => gestalt.list(config)?,
        config::Command::Search(_) => gestalt.search(config)?,
    };

    Ok(())
}

pub(crate) fn chunks_in_world(world: directory::List) -> Result<Vec<region::Chunk>, Error> {
	let mut chunks = Vec::new();
	for dir in world {
		let files = match std::fs::read_dir(dir) {
			Ok(f) => f,
			Err(e) => return Err(Error::ReadFile(e)),
		};
		for file in files {
			let buffer = match std::fs::read(file.unwrap().path()) {
				Ok(b) => b,
				Err(e) => return Err(Error::ReadFile(e)),
			};
			let mut region = match region::Parser::new(buffer).parse() {
				Ok(r) => r,
				Err(e) => return Err(Error::Region(e)),
			};
			chunks.append(&mut region);
		}
	}
	Ok(chunks)
}

pub(crate) fn chunks_in_bundle(world_bundle: &directory::WorldFile) -> Result<Vec<region::Chunk>, Error> {
	let mut chunks = Vec::new();
	println!("starting overworld...");
	chunks.append(&mut chunks_in_world(world_bundle.overworld.clone())?);
	println!("overworld done. starting nether...");
	chunks.append(&mut chunks_in_world(world_bundle.nether.clone())?);
	println!("nether done. starting the end...");
	chunks.append(&mut chunks_in_world(world_bundle.the_end.clone())?);
	Ok(chunks)
}

impl Gestalt {
    fn fast_find(&mut self, method: &config::Method) -> Result<(), Error> {
        Ok(())
    }

    fn find_nbt(&mut self, config: &config::Configuration, file_list: directory::List) -> Result<(), Error> {
        let method = match config.command.value().unwrap() {
            config::Command::Search(method) => method,
            _ => return Err(Error::Command(format!("Gestalt::find() should not be called on anything but a config::Command::Search"))), 
        };

/*
*       let Some(idx) = config.index.value() else {
*           return self.fast_find(method);
*       };
*
*       match idx {
*           config::Index::First => return self.fast_find(method),
*           _ => (),
*       }
*/

        let mut results: Vec<nbt::NBT> = Vec::new();
		let mut visited_nodes = 0;
		
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
                let region = match region::Parser::new(buffer).parse() {
                    Ok(r) => r,
                    Err(e) => return Err(Error::Region(e)),
                };
                for chunk in region {
                    match method {
                        config::Method::Name(name) => {
                            match nbt::query::find_many_by_name(&nbt::TAGString::from(name.clone()), chunk.nbt_owned()) {
                                Ok(mut nbts) => {
									results.append(&mut nbts);
								}
                                Err(e) => {
									match e {
										nbt::query::Error::NotFound => {
											visited_nodes += 1;
											continue;
										},
									}
								},
                            }
                        }
                    }

/*
*                   match idx {
*                       config::Index::Value(i) => {
*                           if &results.len() == i {
*                               break;
*                           }
*                       },
*                       _ => (),
*                   }
*/

                }
            }
        }
		println!("searched: {} node{}", visited_nodes, plural(visited_nodes));
        Ok(())
    }

    fn search(&mut self, config: config::Configuration) -> Result<(), Error> {
        let region_files = directory::region_files(&config);
        println!("{}\n{:?}", "starting search...", config);
        self.find_nbt(&config, region_files.overworld)?;
        self.find_nbt(&config, region_files.nether)?;
        self.find_nbt(&config, region_files.the_end)?;
        println!("{}", "finished search.");
        Ok(())
    }

	fn list(&mut self, config: config::Configuration) -> Result<(), Error> {
		let save_dir = directory::region_files(&config);
		let chunks = chunks_in_bundle(&save_dir)?;
		for chunk in chunks {
			println!("{}", chunk.nbt());
		}
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
        match self {
            Self::Search(e) => f.write_fmt(format_args!("search: {}", e)),
            Self::ReadFile(e) => f.write_fmt(format_args!("reading: {}", e)),
            Self::Region(r) => f.write_fmt(format_args!("{}", r)),
            Self::Command(cmd) => f.write_fmt(format_args!("command: {}", cmd)),
            Self::Finding => f.write_fmt(format_args!("{}", "query gave no results")),
        }
    }
}
