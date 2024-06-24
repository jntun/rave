//
// Created by Justin Tunheim on 6/20/24
//

use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};

const max_nest_depth: i32 = 512;

type TAGByte = u8;
type TAGShort = i16;
type TAGInt = i32;
type TAGLong = i64;
type TAGFloat = f32;
type TAGDouble = f64;

struct TAGByteArray {
    body:   Vec<TAGByte>,
}

struct TAGString {
    str:    Vec<TAGByte>,
}

struct TAGList {
    id:     TAGByte,
    tags:   Vec<NBTData>,
}

struct TAGCompound {
    tags: Vec<NBT>,
}

enum NBTData {
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
    IArray,
    LArray,
}

struct NBT {
    name:    TAGString,
    payload: NBTData,
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
    tags:  Vec<NBT>,
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

        if length > max_nest_depth as i32 {
            return Err(Error::ExceedsMaxNestingDepth(length));
        }

        let mut tags = Vec::new();
        for _ in 0..length {
            match id {
                01 => tags.push(NBTData::Byte(self.nbt_byte()?)),
                02 => tags.push(NBTData::Short(self.nbt_short()?)),
                03 => tags.push(NBTData::Int(self.nbt_int()?)),
                04 => tags.push(NBTData::Long(self.nbt_long()?)),
                05 => tags.push(NBTData::Float(self.nbt_float()?)),
                06 => tags.push(NBTData::Double(self.nbt_double()?)),
                07 => tags.push(NBTData::BArray(self.nbt_barray()?)),
                08 => tags.push(NBTData::String(self.nbt_string()?)),
                09 => tags.push(NBTData::List(self.nbt_list()?)),
                //10 => tags.push(NBTData::Compound(self.nbt_compound()?)),
                _ => return Err(Error::InvalidListType(id)),
            }
        }
        Ok(TAGList{id, tags})
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
        let mut newline = true;
        let mut data: NBTData = NBTData::End;
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
            0  => {
                print!("<0>  End");
            }
            1  => {
                print!("<1>  Byte");
                data = NBTData::Byte(self.nbt_byte()?);
            }
            2  => {
                print!("<2>  Short");
                data = NBTData::Short(self.nbt_short()?);
            }
            3  => {
                print!("<3>  Int");
                data = NBTData::Int(self.nbt_int()?);
            }
            4  => {
                print!("<4>  Long");
                data = NBTData::Long(self.nbt_long()?);
            }
            5  => {
                print!("<5>  Float");
                data = NBTData::Float(self.nbt_float()?);
            }
            6  => {
                print!("<6>  Double");
                data = NBTData::Double(self.nbt_double()?);
            }
            7  => print!("<7>  BArray"),
            8  => {
                print!("<8>  String");
                data = NBTData::String(self.nbt_string()?);
            },
            9  => {
                print!("<9>  List");
                data = NBTData::List(self.nbt_list()?);
            }
            10 => print!("<10> Compound"),
            11 => {
                print!("<11> IArray");
                //data = NBTData::IArray(self.nbt_iarray()?);
                return Err(Error::InvalidByteSequence);
            }
            12 => {
                print!("<12> LArray");
                //data = NBTData::LArray(self.nbt_larray()?);
                return Err(Error::InvalidByteSequence);
            }
            //_ => return Err(Error::InvalidByteSequence),
            _ => {
                newline = false;
            }
        };

        print!(" ");

        for byte in name.str.clone() {
            print!("{}", byte as char);
        }

        if newline {
            print!("\n");
        }
        
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
    pub fn parse(&mut self) -> Result<(), Error> {
        while !self.at_end() {
            match self.consume() {
                Ok(nbt) => self.tags.push(nbt),
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    pub fn new(bytes: Vec<u8>) -> Self {
        Self { 
            length: bytes.len(),
            bytes: Cursor::new(bytes), 
            tags:  Vec::new(),
        }
    }
}

impl std::fmt::Display for NBT {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.name.str, self.payload)
    }
}

impl std::fmt::Display for NBTData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NBTData::End => write!(f, "{}", "<00> End {}"),
            NBTData::Byte(b) => write!(f, "<01> Byte [{}]", b),
            NBTData::Short(s) => write!(f, "<02> Short [{}]", s),
            NBTData::Int(i) => write!(f, "<03> Int [{}]", i),
            NBTData::Long(l) => write!(f, "<04> Long [{}]", l),
            NBTData::Float(fl) => write!(f, "<05> Float [{}]", fl),
            NBTData::Double(d) => write!(f, "<06> Double [{}]", d),
            NBTData::BArray(array) => write!(f, "<07> BArray {}", String::from_utf8(array.body.clone()).unwrap()),
            NBTData::String(str) => write!(f, "<08> String {:?}", str.str),
            NBTData::List(list) => {
                write!(f, "<09> List {}[", list.id)?;
                for tag in list.tags.iter() {
                    write!(f, "{} ", tag)?;
                }
                write!(f, "]")
            }
            NBTData::Compound(compound) => write!(f, "{}", "<10> Compound"),
            NBTData::IArray => write!(f, "{}", "<11> IArray"),
            NBTData::LArray => write!(f, "{}", "<12> LArray"),
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
            Error::ExceedsMaxNestingDepth(length) => write!(f, "Tag with nested tags exceeds maximum allowed nesting depth of {}. Length of tag: {}", max_nest_depth, length),
            Error::TAGString(msg) => write!(f, "{}", msg),
            Error::TAGShort(msg) => write!(f, "{}", msg),
            Error::TAGByte => write!(f, "{}", "Unable to read a NBT byte value (this should never happen...)."),
        }
    }
}
