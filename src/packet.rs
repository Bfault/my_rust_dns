use std::io::{Error, ErrorKind, Result};

use crate::header::Header;
use crate::query::QueryType;
use crate::question::Question;
use crate::record::Record;

pub struct BytePacketBuffer {
    pub buf: [u8; 512],
    pub pos: usize,
}

impl BytePacketBuffer {

    pub fn new() -> BytePacketBuffer {
        BytePacketBuffer {
            buf: [0; 512],
            pos: 0,
        }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn step(&mut self, steps: usize) -> Result<()> {
        self.pos += steps;

        Ok(())
    }

    fn seek(&mut self, pos: usize) -> Result<()> {
        self.pos = pos;

        Ok(())
    }

    pub fn read(&mut self) -> Result<u8> {
        if self.pos >= self.buf.len() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "EOF"));
        }
        
        let byte = self.buf[self.pos];
        self.pos += 1;

        Ok(byte)
    }

    pub fn get(&mut self, pos: usize) -> Result<u8> {
        if pos >= self.buf.len() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "EOF"));
        }
        
        Ok(self.buf[pos])
    }

    pub fn get_range(&mut self, start: usize, len: usize) -> Result<&[u8]> {
        if start + len >= self.buf.len() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "EOF"));
        }
        
        Ok(&self.buf[start..start+len])
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let byte1 = self.read()? as u16;
        let byte2 = self.read()? as u16;

        let bytes = byte1 << 8 | byte2;

        Ok(bytes)
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let byte1 = self.read()? as u32;
        let byte2 = self.read()? as u32;
        let byte3 = self.read()? as u32;
        let byte4 = self.read()? as u32;

        let bytes = byte1 << 24 | byte2 << 16 | byte3 << 8 | byte4;

        Ok(bytes)
    }

    pub fn read_qname(&mut self, outstr: &mut String) -> Result<()> {
        let mut pos = self.pos;

        let mut jumped = false;
        let max_jumps = 5;
        let mut jumps_performed = 0;

        let mut delim = "";

        loop {
            if jumps_performed > max_jumps {
                return Err(Error::new(ErrorKind::InvalidData, "Too many jumps"));
            }
            let len = self.get(pos)?;
            if (len & 0xC0) == 0xC0 {
                if !jumped {
                    self.seek(pos + 2)?;
                }
                    
                let b2 = self.get(pos + 1)? as u16;
                let offset = (((len as u16) ^ 0xC0) << 8) | b2;
                pos = offset as usize;

                jumped = true;
                jumps_performed += 1;
                continue;
            } else {
                pos += 1;

                if len == 0 {
                    break;
                }
                outstr.push_str(delim);
                let str_buffer = self.get_range(pos, len as usize)?;
                outstr.push_str(&String::from_utf8_lossy(str_buffer).to_lowercase());

                delim = ".";

                pos += len as usize;
            }
        }

        if !jumped {
            self.seek(pos)?;
        }

        Ok(())
    }

    fn write(&mut self, val: u8) -> Result<()> {
        if self.pos >= self.buf.len() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "EOF"));
        }
        self.buf[self.pos] = val;
        self.pos += 1;
        Ok(())
    }

    pub fn write_u8(&mut self, val: u8) -> Result<()> {
        self.write(val)?;

        Ok(())
    }

    pub fn write_u16(&mut self, val: u16) -> Result<()> {
        self.write((val >> 8) as u8)?;
        self.write((val & 0xFF) as u8)?;

        Ok(())
    }

    pub fn write_u32(&mut self, val: u32) -> Result<()> {
        self.write(((val >> 24) & 0xFF) as u8)?;
        self.write(((val >> 16) & 0xFF) as u8)?;
        self.write(((val >> 8) & 0xFF) as u8)?;
        self.write(((val >> 0) & 0xFF) as u8)?;

        Ok(())
    }

    pub fn write_qname(&mut self, qname: &str) -> Result<()> {
        for label in qname.split('.') {
            let len = label.len();
            if len > 0x3f {
                return Err(Error::new(ErrorKind::InvalidData, "Label exceeds 63 bytes"));
            }

            self.write_u8(len as u8)?;
            for b in label.as_bytes() {
                self.write_u8(*b)?;
            }
        }

        self.write_u8(0)?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Packet {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<Record>,
    pub authorities: Vec<Record>,
    pub resources: Vec<Record>,
}

impl Packet {
    pub fn new() -> Packet {
        Packet {
            header: Header::new(),
            questions: Vec::new(),
            answers: Vec::new(),
            authorities: Vec::new(),
            resources: Vec::new(),
        }
    }

    pub fn from_buffer(buffer: &mut BytePacketBuffer) -> Result<Packet> {
        let mut result = Packet::new();
        result.header.read(buffer)?;
        
        for _ in 0..result.header.qdcount {
            let mut question = Question::new("".to_string(), QueryType::UNKNOWN(0));
            question.read(buffer)?;
            result.questions.push(question);
        }

        for _ in 0..result.header.ancount {
            let rec = Record::read(buffer)?;
            result.answers.push(rec);
        }
        for _ in 0..result.header.nscount {
            let rec = Record::read(buffer)?;
            result.authorities.push(rec);
        }
        for _ in 0..result.header.arcount {
            let rec = Record::read(buffer)?;
            result.resources.push(rec);
        }

        Ok(result)
    }

    pub fn write(&mut self, buffer: &mut BytePacketBuffer) -> Result<()> {
        self.header.qdcount = self.questions.len() as u16;
        self.header.ancount = self.answers.len() as u16;
        self.header.nscount = self.authorities.len() as u16;
        self.header.arcount = self.resources.len() as u16;

        self.header.write(buffer)?;

        for question in &self.questions {
            question.write(buffer)?;
        }
        for rec in &self.answers {
            rec.write(buffer)?;
        }
        for rec in &self.authorities {
            rec.write(buffer)?;
        }
        for rec in &self.resources {
            rec.write(buffer)?;
        }

        Ok(())
    }
}