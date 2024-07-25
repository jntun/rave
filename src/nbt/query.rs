//
// Created by Justin Tunheim on 7/22/24
//

use crate::nbt::{ NBT,TAGString };

pub enum Error {
}

pub(crate) fn find_first_by_name(name: TAGString, root: &NBT) -> Result<NBT, Error> {
    Ok(NBT::default())
}

pub(crate) fn find_many_by_name(name: TAGString, root: &NBT) -> Result<NBT, Error> {
    Ok(NBT::default())
}
