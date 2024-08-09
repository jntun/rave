//
// Created by Justin Tunheim on 7/22/24
//

use crate::nbt::{ NBT, Payload, TAGString };

pub enum Error {
    NotFound
}

pub(crate) fn find_first_by_name(name: TAGString, root: &NBT) -> Result<NBT, Error> {
    Ok(NBT::default())
}

pub(crate) fn find_many_by_name(name: &TAGString, root: NBT) -> Result<Vec<NBT>, Error> {
    let mut tags = Vec::new();

    if root.name == *name {
        tags.push(root);
    } else {
        match root.payload {
            Payload::Compound(compound_tag) => {
                for nbt in compound_tag.tags {
                    tags.append(&mut find_many_by_name(name, nbt)?);
                }
            },
            _ => return Err(Error::NotFound),
        };
    }

    Ok(tags)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => f.write_str("not found")
        }
    }
}
