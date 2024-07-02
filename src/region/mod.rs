//
// Created by Justin Tunheim on 6/26/24
//

use std::io::{Read, Cursor};
use byteorder::{BigEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;

use crate::nbt::{self, NBT};

const SECTOR: usize = 1024 * 4;
const ENTRIES: usize = 1024;

const __GZIP: u8 = 1; /* "unused in practice" */
const ZLIB  : u8 = 2;
const NONE  : u8 = 3;
const LZ4   : u8 = 4;
const CUSTOM: u8 = 127;

pub enum Error {
    CouldntSortChunks,
    ChunkLength,
    Compression,
    CompressionType(u8),
    Decompress(u8, String),
    ChunkNBT(nbt::Error),
    Unimplemented,
}

pub struct Parser {
    length: usize,
    bytes:  Cursor<Vec<u8>>,
    copy:   Vec<u8>,
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
struct ChunkHeaderPair {
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

    fn chunk(&mut self, hdr_pair: &ChunkHeaderPair) -> Result<Chunk, Error> {
        let Ok(raw_length) = self.bytes.read_u32::<BigEndian>() else {
            return Err(Error::ChunkLength);
        };
        let length = raw_length as usize - 1;
        let Ok(compression) = self.bytes.read_u8() else {
            return Err(Error::Compression);
        };

        let mut nbt_data = Vec::new();
        match compression {
            ZLIB => {
                let pos = self.bytes.position() as usize;
                if let Err(e) = ZlibDecoder::new(&self.copy[pos..pos+length]).read_to_end(&mut nbt_data) {
                    return Err(Error::Decompress(compression, e.to_string()));
                };
            }
            _ => return Err(Error::CompressionType(compression)),
        }

        let mut nbts = Vec::new();
        if let Err(e) = nbt::Parser::new(nbt_data).parse(&mut nbts) {
            return Err(Error::ChunkNBT(e));
        }

        println!("root:\n{}", nbts.first().unwrap());

        todo!();

        Ok(Chunk{ size: 0, root: NBT::default() })
    }
}

fn sort_chunk_data_by_location(data: Vec<ChunkHeaderPair>) -> Result<Vec<ChunkHeaderPair>, Error> {
    let mut chunk_data = Vec::new();
    for datum in data.into_iter() {
        let offset: usize = datum.location.offset.try_into().unwrap();
        if offset >= chunk_data.len() {
            chunk_data.resize(offset+1, ChunkHeaderPair::default());
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
               chunk_data.push(ChunkHeaderPair{ location, timestamp });
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
            bytes:  Cursor::new(bytes.clone()),
            copy:   bytes.clone(),
        }
    }
}

impl Location {
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::CouldntSortChunks => write!(f, "{}", "unable to sort chunks"),
            Error::ChunkLength => write!(f, "{}", "unable to parse chunk length"),
            Error::Compression => write!(f, "{}", "unable to parse compression"),
            Error::CompressionType(compression) => write!(f, "invalid compression type: {}", compression),
            Error::Decompress(compression, err) => write!(f, "failed decompression of type {}: {}", compression, err),
            Error::ChunkNBT(err) => write!(f, "failed parsing chunk nbt: {}", err),
            Error::Unimplemented => write!(f, "{}", "not implemented (yet :^)"),
        }
    }
}

impl Default for ChunkHeaderPair {
    fn default() -> Self {
        Self {
            location: Location { offset: 0, sector: 0 },
            timestamp: Timestamp { entry: 0 },
        }
    }
}
