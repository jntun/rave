//
// Created by Justin Tunheim on 6/26/24
//

use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};

use crate::nbt::NBT;

const SECTOR: usize = 1024 * 4;
const ENTRIES: usize = 1024;

pub enum Error {
    InvalidLocation(String),
    CouldntSortChunks,
}

pub struct Parser {
    length: usize,
    bytes:  Cursor<Vec<u8>>,
}

pub struct Chunk {
    size: usize,
    root: NBT,
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

#[derive(Clone)]
struct ChunkData {
    location: Location,
    timestamp: Timestamp,
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

    fn chunk(&mut self, data: &ChunkData) -> Result<Chunk, Error> {
        println!("{:<3?} {:<8?}", data.location, data.timestamp);
        Ok(Chunk{ size: 0, root: NBT::default() })
    }
}

fn sort_chunk_data_by_location(data: Vec<ChunkData>) -> Result<Vec<ChunkData>, Error> {
    let mut chunk_data = Vec::new();
    for datum in data.into_iter() {
        let offset: usize = datum.location.offset.try_into().unwrap();
        if offset >= chunk_data.len() {
            chunk_data.resize(offset+1, ChunkData::default());
        }
        chunk_data[offset] = datum;
    }

    chunk_data.retain(|datum| datum.location.offset != 0);
    Ok(chunk_data)
}

impl Parser {
    pub fn parse(&mut self) -> Result<NBT, Error> {
        let locations = self.locations()?;
        let timestamps = self.timestamps()?;

        let mut nbt: NBT = NBT::default();
        let mut chunk_data = Vec::new();
        let mut chunks = Vec::new();

        for (location, timestamp) in locations.into_iter().zip(timestamps.into_iter()) {
            if location.offset != 0 || location.sector != 0 {
               chunk_data.push(ChunkData{ location, timestamp });
            }
        }

        chunk_data = sort_chunk_data_by_location(chunk_data)?;
        for data in chunk_data.into_iter() {
            chunks.push(self.chunk(&data)?);
        }
        Ok(nbt)
    }

    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            length: bytes.len(),
            bytes:  Cursor::new(bytes),
        }
    }
}

impl Location {
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            _ => todo!(),
        }
    }
}

impl Default for ChunkData {
    fn default() -> Self {
        Self {
            location: Location { offset: 0, sector: 0 },
            timestamp: Timestamp { entry: 0 },
        }
    }
}
