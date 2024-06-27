//
// Created by Justin Tunheim on 6/26/24
//

use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};

use crate::nbt::NBT;

const ENTRIES: usize = 1024;

pub enum Error {
    InvalidLocation(String),
}

pub struct Parser {
    length: usize,
    bytes:  Cursor<Vec<u8>>,
}

#[derive(Clone, Copy, Debug)]
struct Location {
    offset: u32,
    sector: u8,
}

#[derive(Clone, Copy, Debug)]
struct Timestamp {
    entry: i32,
}

impl Parser {
    fn locations(&mut self) -> Result<[Location; ENTRIES], Error> {
        let mut locations: [Location; ENTRIES] = [Location {offset: 0, sector: 0}; ENTRIES];
        for location in locations.iter_mut() {
            location.offset = self.bytes.read_u24::<BigEndian>().unwrap();
            location.sector = self.bytes.read_u8().unwrap();
        }
        Ok(locations)
    }

    fn timestamps(&mut self) -> Result<[Timestamp; ENTRIES], Error> {
        let mut timestamps: [Timestamp; ENTRIES] = [Timestamp {entry: 0}; ENTRIES];
        for timestamp in timestamps.iter_mut() {
            timestamp.entry = self.bytes.read_i32::<BigEndian>().unwrap();
        }
        Ok(timestamps)
    }
}

impl Parser {
    pub fn parse(&mut self, nbt: &mut NBT) -> Result<(), Error> {
        let mut nbt: NBT = NBT::default();
        let locations = self.locations()?;
        println!("{:?}", locations);
        Ok(())
    }

    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            length: bytes.len(),
            bytes:  Cursor::new(bytes),
        }
    }
}
