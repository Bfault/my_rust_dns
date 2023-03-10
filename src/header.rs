use std::io::Result;

use crate::rescode::ResultCode;
use crate::packet::BytePacketBuffer;

#[derive(Clone, Debug)]
pub struct Header {
    pub id: u16,

    pub qr: bool,
    pub opcode: u8,
    pub aa: bool,
    pub tc: bool,
    pub rd: bool,

    pub ra: bool,
    pub z: u8,
    pub rcode: ResultCode,

    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16,
    pub arcount: u16,
}

impl Header {
    pub fn new() -> Header {
        Header {
            id: 0,

            qr: false,
            opcode: 0,
            aa: false,
            tc: false,
            rd: false,

            ra: false,
            z: 0,
            rcode: ResultCode::NOERROR,

            qdcount: 0,
            ancount: 0,
            nscount: 0,
            arcount: 0,
        }
    }

    pub fn read(&mut self, buffer: &mut BytePacketBuffer) -> Result<()> {
        self.id = buffer.read_u16()?;

        let flags = buffer.read_u16()?;

        self.qr = (flags & 0x8000) == 0x8000;                                   // 0x8000   = 1000 0000 0000 0000
        self.opcode = ((flags & 0x7800) >> 11) as u8;                           // 0x7800   = 0111 1000 0000 0000
        self.aa = (flags & 0x400) == 0x400;                                     // 0x400    = 0000 0100 0000 0000
        self.tc = (flags & 0x200) == 0x200;                                     // 0x200    = 0000 0010 0000 0000
        self.rd = (flags & 0x100) == 0x100;                                     // 0x100    = 0000 0001 0000 0000

        self.ra = (flags & 0x80) == 0x80;                                       // 0x80     = 0000 0000 1000 0000
        self.z = ((flags & 0x70) >> 4) as u8;                                   // 0x70     = 0000 0000 0111 0000
        self.rcode = ResultCode::from_num((flags & 0xf) as u8);                 // 0xf      = 0000 0000 0000 1111

        self.qdcount = buffer.read_u16()?;
        self.ancount = buffer.read_u16()?;
        self.nscount = buffer.read_u16()?;
        self.arcount = buffer.read_u16()?;

        Ok(())
    }

    pub fn write(&self, buffer: &mut BytePacketBuffer) -> Result<()> {
        buffer.write_u16(self.id)?;

        buffer.write_u8(
            ((self.qr as u8) << 7) as u8        // x000 0000
                | (self.opcode << 3)                // 0xxx x000
                | ((self.aa as u8) << 2)            // 0000 0x00
                | ((self.tc as u8) << 1)            // 0000 00x0
                | (self.rd as u8),                  // 0000 000x
        )?;

        buffer.write_u8(
            ((self.ra as u8) << 7)
                | ((self.z as u8) << 6)
                | (self.rcode as u8),
        )?;

        buffer.write_u16(self.qdcount)?;
        buffer.write_u16(self.ancount)?;
        buffer.write_u16(self.nscount)?;
        buffer.write_u16(self.arcount)?;

        Ok(())
    }
}