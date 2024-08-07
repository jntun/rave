//
// Created by Justin Tunheim on 6/26/24
//

use std::io::{Read, Cursor, Seek, SeekFrom};
use byteorder::{BigEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;
use colored::*;

use crate::nbt::{self, NBT};

const KIB: usize = 1024;
const ENTRIES: usize = KIB;
const BOUNDARY: usize = 4*KIB;

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

#[derive(Clone, Copy)]
pub struct Responsible {
    source: bool,
    addr: usize,
    byte: u8,
}

pub struct Report {
    err: Error,
    pos: u64,
    responsible: [Responsible; 20],
}

pub struct Parser {
    length: usize,
    bytes:  Cursor<Vec<u8>>,
    copy:   Vec<u8>,
}

pub struct Chunk {
    hdr_pair: ChunkHeaderPair,
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

#[derive(Clone, Debug)]
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

    fn chunk(&mut self, hdr_pair: ChunkHeaderPair) -> Result<Chunk, Error> {
        let Ok(raw_length) = self.bytes.read_u32::<BigEndian>() else {
            return Err(Error::ChunkLength);
        };
        let length = raw_length as usize - 1;
        let Ok(compression) = self.bytes.read_u8() else {
            return Err(Error::Compression);
        };

        let mut nbt_data = Vec::new();
        let pos = self.bytes.position() as usize;
        match compression {
            ZLIB => {
                if let Err(e) = ZlibDecoder::new(&self.copy[pos..pos+length]).read_to_end(&mut nbt_data) {
                    return Err(Error::Decompress(compression, e.to_string()));
                };
            }
            _ => return Err(Error::CompressionType(compression)),
        }

        let mut root = NBT::default();
        if let Err(e) = nbt::Parser::new(nbt_data).parse(&mut root) {
            return Err(Error::ChunkNBT(e));
        }

        Ok(Chunk{ hdr_pair, root: root })
    }
}


impl Parser {
    pub fn parse(&mut self) -> Result<Vec<Chunk>, Report> {
        if self.copy.len() == 0 {
            return Ok(Vec::new())
        }
        let locations = match self.locations() {
            Ok(locs) => locs,
            Err(e)   => return Err(Report::new(e, &mut self.bytes.clone())),
        };
        let timestamps = match self.timestamps() {
            Ok(times) => times,
            Err(e)    => return Err(Report::new(e, &mut self.bytes.clone())),
        };

        let mut chunk_data = Vec::new();
        let mut chunks = Vec::new();

        for (location, timestamp) in locations.into_iter().zip(timestamps.into_iter()) {
            if location.offset != 0 && location.sector != 0 {
                chunk_data.push(ChunkHeaderPair{ location, timestamp });
            }
        }

        chunk_data = match sort_chunk_data_by_location(chunk_data) {
            Ok(data) => data,
            Err(e)   => return Err(Report::new(e, &mut self.bytes.clone())),
        };
        for data in chunk_data.into_iter() {
            let offset = data.location.offset as usize * BOUNDARY;
            self.bytes.seek(SeekFrom::Start(offset as u64)).expect("Invalid chunk offset");
            match self.chunk(data) {
                Ok(chunk) => chunks.push(chunk),
                Err(e)    => return Err(Report::new(e, &mut self.bytes.clone())),
            }
        }

        Ok(chunks)
    }

    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            length: bytes.len(),
            bytes:  Cursor::new(bytes.clone()),
            copy:   bytes.clone(),
        }
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

impl Chunk {
    pub fn nbt(&self) -> &NBT {
        &self.root
    }

    pub fn nbt_mut(&mut self) -> &mut NBT {
        &mut self.root
    }

    pub fn nbt_owned(self) -> NBT {
        self.root
    }
}


impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::CouldntSortChunks => write!(f, "{}", "unable to sort chunks"),
            Error::ChunkLength => write!(f, "{}", "unable to parse chunk length"),
            Error::Compression => write!(f, "{}", "unable to parse compression"),
            Error::CompressionType(compression) => write!(f, "invalid compression type: {:#02x}", compression),
            Error::Decompress(compression, err) => write!(f, "failed decompression of type {}: {}", compression, err),
            Error::ChunkNBT(err) => write!(f, "failed parsing chunk nbt: {}", err),
            Error::Unimplemented => write!(f, "{}", "not implemented (yet :^)"),
        }
    }
}

impl Report {
    pub fn new(err: Error, bytes: &mut Cursor<Vec<u8>>) -> Self {
        /* FIXME:
        *     We need to know the size of what we just tried to scan
        *     so we can calculate how far back the actual source of the error lies.
        *     For now, we just assume it was the last byte pulled out from the cursor.
        *
        *                                   vv */
        let start = bytes.position().clone()-1;

        match start {
            0..10 => bytes.seek(SeekFrom::Start(0)),
            _     => bytes.seek(SeekFrom::Current(-10)),
        };
        let mut blame: [Responsible; 20] = [Responsible{ source: false, addr: 0, byte: 0} ; 20];
        for responsible in blame.iter_mut() {
            responsible.addr = bytes.position() as usize;
            responsible.byte = bytes.read_u8().unwrap();
            if responsible.addr == start as usize {
                responsible.source = true;
            }
        }
        Self { err: err, pos: start, responsible: blame }
    }
}

impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (i, responsible) in self.responsible.iter().enumerate() {
            if responsible.source {
                let out = format!("{:#x} {:#02x}\n", responsible.addr, responsible.byte);
                write!(f, "{}", out.green())?;
            } else {
                write!(f, "{:#x} {:#02x}\n", responsible.addr, responsible.byte)?;
            }
        }
        write!(f, "\nerror occured at byte {} ({:#02x}):\n\t{}",
                self.pos,
                self.pos,
                self.err
        )
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
