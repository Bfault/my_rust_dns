use std::env;
use std::io::Result;
use std::net::UdpSocket;

mod packet;
mod header;
mod query;
mod record;
mod question;
mod rescode;

use packet::{BytePacketBuffer, Packet};

use crate::query::QueryType;

fn main() -> Result<()> {
    let qname = env::args().nth(1).unwrap_or("google.com".to_string());
    let qtype = QueryType::A;

    let server = ("8.8.8.8", 53);
    let socket = UdpSocket::bind(("0.0.0.0", 43210))?;

    let mut packet = Packet::new();

    packet.header.id = 666;
    packet.header.qdcount = 1;
    packet.header.rd = true;
    packet.questions.push(
        question::Question::new(qname.to_string(), qtype)
    );

    let mut req_buffer = BytePacketBuffer::new();
    packet.write(&mut req_buffer)?;
    socket.send_to(&req_buffer.buf[0..req_buffer.pos], server)?;

    let mut res_buffer = BytePacketBuffer::new();
    socket.recv_from(&mut res_buffer.buf)?;

    let res_packet = Packet::from_buffer(&mut res_buffer)?;
    println!("{:#?}", res_packet.header);

    for q in res_packet.questions {
        println!("{:#?}", q);
    }
    for rec in res_packet.answers {
        println!("{:#?}", rec);
    }
    for rec in res_packet.authorities {
        println!("{:#?}", rec);
    }
    for rec in res_packet.resources {
        println!("{:#?}", rec);
    }

    Ok(())
}