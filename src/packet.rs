use crate::query::Query;
use crate::record::Record;

use crate::header::Header;

pub struct BytePacketBuffer {
    buf: [u8; 512],
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

    pub fn seek(&mut self, pos: usize) -> Result<()> {
        self.pos = pos;

        Ok(())
    }

    pub fn read(&mut self) -> Result<u8> {
        if self.pos >= self.buf.len() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "EOF"));
        }
        
        let byte = self.buf[self.pos];
        self.pos += 1;

        OK(byte)
    }

    pub fn get(&self) -> Result<u8> {
        if self.pos >= self.buf.len() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "EOF"));
        }
        
        let byte = self.buf[self.pos];

        OK(byte)
    }

    pub fn get_range(&self, start: usize, len: usize) -> Result<&[u8]> {
        if start + len >= self.buf.len() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "EOF"));
        }
        
        let bytes = &self.buf[start..start+len];

        OK(bytes)
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let byte1 = self.read()? as u16;
        let byte2 = self.read()? as u16;

        let bytes = byte1 << 8 | byte2;

        OK(bytes)
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let byte1 = self.read()? as u32;
        let byte2 = self.read()? as u32;
        let byte3 = self.read()? as u32;
        let byte4 = self.read()? as u32;

        let bytes = byte1 << 24 | byte2 << 16 | byte3 << 8 | byte4;

        OK(bytes)
    }

    pub fn read_qnmae(&self, outstr: &mut String) -> Result(()) {
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

        OK(())
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
        let mut packet = Packet::new();
        packet.header.read(buffer)?;

        for _ in 0..packet.header.questions {
            let mut question = Question::new("".to_string(), QueryType::UNKNOWN(0));
            question.read(buffer)?;
            packet.questions.push(question);
        }

        for _ in 0..packet.header.answers {
            let rec = Record::read(buffer)?;
            packet.answers.push(rec);
        }
        for _ in 0..packet.header.authoritative_entries {
            let rec = Record::read(buffer)?;
            packet.authorities.push(rec);
        }
        for _ in 0..packet.header.resource_entries {
            let rec = Record::read(buffer)?;
            packet.resources.push(rec);
        }

        Ok(packet)
    }
}