//
// Created by Justin Tunheim on 6/20/24
//

use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};

const MAX_DEPTH_NEST: i32 = 512;

pub type TAGByte = u8;
pub type TAGShort = i16;
pub type TAGInt = i32;
pub type TAGLong = i64;
pub type TAGFloat = f32;
pub type TAGDouble = f64;

pub struct TAGByteArray {
    pub body: Vec<TAGByte>,
}

pub struct TAGString {
    pub str: Vec<TAGByte>,
}

pub struct TAGList {
    id:         TAGByte,
    pub tags:   Vec<Payload>,
}

pub struct TAGCompound {
    pub tags: Vec<NBT>,
}

pub struct TAGIArray {
    pub ints: Vec<TAGInt>,
}

pub struct TAGLArray {
    pub longs: Vec<TAGLong>,
}

pub enum Payload {
    End,
    Byte(TAGByte),
    Short(TAGShort),
    Int(TAGInt),
    Long(TAGLong),
    Float(TAGFloat),
    Double(TAGDouble),
    BArray(TAGByteArray),
    String(TAGString),
    List(TAGList),
    Compound(TAGCompound),
    IArray(TAGIArray),
    LArray(TAGLArray),
}

pub struct NBT {
    pub name:    TAGString,
    pub payload: Payload,
}

pub enum Error {
    EndOfBytes,
    InvalidType,
    NegativeLength(i32),
    ExceedsMaxNestingDepth(i32),
    InvalidListType(TAGByte),
    InvalidByteSequence,
    TAGString(String),
    TAGShort(String),
    TAGByte,
}

pub struct Parser {
    length: usize,
    bytes: Cursor<Vec<u8>>,
}

impl Parser {
    fn nbt_byte(&mut self) -> Result<TAGByte, Error> {
        let Ok(byte) = self.bytes.read_u8() else {
            return Err(Error::TAGByte)
        };
        Ok(byte)
    }

    fn nbt_short(&mut self) -> Result<TAGShort, Error> {
        let short = self.bytes.read_u16::<BigEndian>().unwrap();
        Ok(short as TAGShort)
    }

    fn nbt_int(&mut self) -> Result<TAGInt, Error> {
        let int = self.bytes.read_i32::<BigEndian>().unwrap();
        Ok(int as TAGInt)
    }

    fn nbt_long(&mut self) -> Result<TAGLong, Error> {
        let long = self.bytes.read_i64::<BigEndian>().unwrap();
        Ok(long)
    }

    fn nbt_float(&mut self) -> Result<TAGFloat, Error> {
        let float = self.bytes.read_f32::<BigEndian>().unwrap();
        Ok(float)
    }

    fn nbt_double(&mut self) -> Result<TAGDouble, Error> {
        let double = self.bytes.read_f64::<BigEndian>().unwrap();
        Ok(double)
    }

    fn nbt_barray(&mut self) -> Result<TAGByteArray, Error> {
        let length = self.nbt_int()?;
        self.check_length(length)?;
        let mut body = Vec::new();
        for _ in 0..length {
            body.push(self.nbt_byte()?);
        }
        Ok(TAGByteArray{body})
    }

    fn nbt_string(&mut self) -> Result<TAGString, Error> {
        let length = match self.nbt_short() {
            Ok(length) => length,
            Err(_)     => return Err(Error::TAGString(format!("String NBT length field is invalid"))),
        };
        if length as usize >= self.length {
            return Err(Error::EndOfBytes)
        }
        let mut str = Vec::new();
        for _ in 0..length {
            let Ok(byte) = self.nbt_byte() else {
                return Err(Error::TAGString(format!("String NBT character byte data is invalid"))); 
            };
            str.push(byte);
        }
            
        Ok(TAGString{str})
    }

    fn nbt_list(&mut self) -> Result<TAGList, Error> {
        let id = self.nbt_byte()?;
        let length = self.nbt_int()?;
        self.check_length(length)?;

        if length > MAX_DEPTH_NEST as i32 {
            return Err(Error::ExceedsMaxNestingDepth(length));
        }

        let mut tags = Vec::new();
        for _ in 0..length {
            match id {
                01 => tags.push(Payload::Byte(self.nbt_byte()?)),
                02 => tags.push(Payload::Short(self.nbt_short()?)),
                03 => tags.push(Payload::Int(self.nbt_int()?)),
                04 => tags.push(Payload::Long(self.nbt_long()?)),
                05 => tags.push(Payload::Float(self.nbt_float()?)),
                06 => tags.push(Payload::Double(self.nbt_double()?)),
                07 => tags.push(Payload::BArray(self.nbt_barray()?)),
                08 => tags.push(Payload::String(self.nbt_string()?)),
                09 => tags.push(Payload::List(self.nbt_list()?)),
                10 => tags.push(Payload::Compound(self.nbt_compound()?)),
                11 => tags.push(Payload::IArray(self.nbt_iarray()?)),
                12 => tags.push(Payload::LArray(self.nbt_larray()?)),
                _ => return Err(Error::InvalidListType(id)),
            }
        }
        Ok(TAGList{id, tags})
    }

    fn nbt_compound(&mut self) -> Result<TAGCompound, Error> {
        let mut compound = TAGCompound{tags: Vec::new()};
        for _ in 0..MAX_DEPTH_NEST {
            let tag = self.consume()?;
            compound.tags.push(tag);
            match compound.tags.last().unwrap().payload {
                Payload::End => return Ok(compound),
                _ => (),
            }
        }
        Err(Error::ExceedsMaxNestingDepth(513))
    }

    fn nbt_iarray(&mut self) -> Result<TAGIArray, Error> {
        let length = self.nbt_int()?;
        self.check_length(length)?;
        let mut ints = Vec::new();
        for _ in 0..length {
            ints.push(self.nbt_int()?);
        }
        Ok(TAGIArray{ints})
    }

    fn nbt_larray(&mut self) -> Result<TAGLArray, Error> {
        let length = self.nbt_int()?;
        self.check_length(length)?;
        let mut longs = Vec::new();
        for _ in 0..length {
            longs.push(self.nbt_long()?);
        }
        Ok(TAGLArray{longs})
    }

    fn check_length(&self, length: TAGInt) -> Result<(), Error> {
        if length < 0 {
            return Err(Error::NegativeLength(length))
        }
        if length >= self.length as i32 {
            return Err(Error::EndOfBytes);
        }
        Ok(())
    }

    fn consume(&mut self) -> Result<NBT, Error> {
        let mut data: Payload = Payload::End;
        let Ok(byte) = self.nbt_byte() else {
            return Err(Error::InvalidType);
        };

        let name = match byte {
            0 => { TAGString{str: Vec::new()} },
            1..12 => {
                match self.nbt_string() {
                    Ok(name) => name,
                    Err(e)   => return Err(e),
                }
            },
            _ => { return Err(Error::InvalidByteSequence); }
        };

        match byte {
            0  => (),
            1  => data = Payload::Byte(self.nbt_byte()?),
            2  => data = Payload::Short(self.nbt_short()?),
            3  => data = Payload::Int(self.nbt_int()?),
            4  => data = Payload::Long(self.nbt_long()?),
            5  => data = Payload::Float(self.nbt_float()?),
            6  => data = Payload::Double(self.nbt_double()?),
            7  => data = Payload::BArray(self.nbt_barray()?),
            8  => data = Payload::String(self.nbt_string()?),
            9  => data = Payload::List(self.nbt_list()?),
            10 => data = Payload::Compound(self.nbt_compound()?),
            11 => data = Payload::IArray(self.nbt_iarray()?),
            12 => data = Payload::LArray(self.nbt_larray()?),
            _ => return Err(Error::InvalidByteSequence),
        };

        return Ok(NBT{name, payload: data});
    }

    fn at_end(&self) -> bool {
        if self.bytes.position() as usize == self.length {
            return true;
        }
        return false;
    }
}

impl Parser {
    pub fn parse(&mut self, root: &mut NBT) -> Result<(), Error> {
        match self.consume() {
            Ok(nbt) => {
                root.name    = nbt.name;
                root.payload = nbt.payload;
            }
            Err(e)  => return Err(e),
        };
        
        Ok(())
    }

    pub fn new(bytes: Vec<u8>) -> Self {
        Self { 
            length: bytes.len(),
            bytes: Cursor::new(bytes), 
        }
    }
}

impl Default for NBT {
    fn default() -> Self {
        Self {
            name:    TAGString{str: Vec::new()},
            payload: Payload::End,
        }
    }
}

impl std::fmt::Display for NBT {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.name.str, self.payload)
    }
}

impl std::fmt::Display for TAGString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8(self.str.clone()).unwrap())
    }
}

impl std::fmt::Display for Payload {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Payload::End => write!(f, "{}", "<00> End {}"),
            Payload::Byte(b) => write!(f, "<01> Byte {}", b),
            Payload::Short(s) => write!(f, "<02> Short {}", s),
            Payload::Int(i) => write!(f, "<03> Int {}", i),
            Payload::Long(l) => write!(f, "<04> Long {}", l),
            Payload::Float(fl) => write!(f, "<05> Float {}", fl),
            Payload::Double(d) => write!(f, "<06> Double {}", d),
            Payload::BArray(array) => write!(f, "<07> BArray {}", String::from_utf8(array.body.clone()).unwrap()),
            Payload::String(str) => write!(f, "<08> String {:?}", String::from_utf8(str.str.clone()).unwrap()),
            Payload::List(list) => {
                write!(f, "<09> List {} [", list.tags.len())?;
                for tag in list.tags.iter() {
                    write!(f, "\n\t{}", tag)?;
                }
                write!(f, "\n\t]")
            }
            Payload::Compound(compound) => {
                write!(f, "{}", "<10> Compound [\n")?;
                for tag in compound.tags.iter() {
                    write!(f, "\n\t{}: {}", tag.name, tag.payload)?;
                }
                write!(f, "\n{}", "]")
            }
            Payload::IArray(iarray) => {
                write!(f, "{} {}\n\t{}", "<11> IArray ", iarray.ints.len(), "( ", )?;
                for int in iarray.ints.iter() {
                    write!(f, "{}, ", int)?;
                }
                write!(f, "{}", ")")
            }
            Payload::LArray(larray) => {
                write!(f, "{}\n\t{}", "<12> LArray", "( ")?;
                for long in larray.longs.iter() {
                    write!(f, "{}, ", long)?;
                }
                write!(f, "{}", ")")
            }
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::EndOfBytes => write!(f, "{}", "Reached end of byte sequence while attempting to parse!"),
            Error::InvalidType => write!(f, "{}", "Encountered an invalid opcode byte sequence."),
            Error::InvalidListType(tag_id) => write!(f, "List cannot contain elements of type '{}'.", tag_id),
            Error::InvalidByteSequence => write!(f, "Reached unparseable byte sequence."),
            Error::NegativeLength(length) => write!(f, "{} is an invalid length due to being negative.", length), 
            Error::ExceedsMaxNestingDepth(length) => write!(f, "Tag with nested tags exceeds maximum allowed nesting depth of {}. Length of tag: {}", MAX_DEPTH_NEST, length),
            Error::TAGString(msg) => write!(f, "{}", msg),
            Error::TAGShort(msg) => write!(f, "{}", msg),
            Error::TAGByte => write!(f, "{}", "Unable to read a NBT byte value (this should never happen...)."),
        }
    }
}
