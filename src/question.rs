use std::io::Result;

use crate::query::QueryType;
use crate::packet::BytePacketBuffer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    pub name: String,
    pub qtype: QueryType,
}

impl Question {
    pub fn new(name: String, qtype: QueryType) -> Question {
        Question {
            name,
            qtype,
        }
    }

    pub fn read(&mut self, buffer: &mut BytePacketBuffer) -> Result<()> {
        buffer.read_qname(&mut self.name)?;
        self.qtype = QueryType::from_num(buffer.read_u16()?);
        let _ = buffer.read_u16()?; // qclass

        Ok(())
    }
}