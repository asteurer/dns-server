use std::{env, error, net::UdpSocket};
use anyhow::anyhow;

use crate::{build::build_message, parse::parse_message, types::{DNSMessage, DNSQuestion, ResourceRecord}};
mod types;
mod parse;
mod build;

fn main() -> Result<(), Box<dyn error::Error>>{
    let args: Vec<String> = env::args().collect();
    let is_forward_mode = args.len() >= 3 && args[1] == "--resolver".to_string();

    let forward_conn: Option<UdpSocket> = if is_forward_mode {
        let forward_addr = &args[2];
        println!("Forwarding queries to {}", forward_addr);
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(forward_addr)?;
        Some(socket)
    } else {
        println!("Running in resolve mode");
        None
    };

    println!("Server running on 127.0.0.1:2053");
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];
    
    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                let data: Vec<u8> = buf[..size].to_vec();
                println!("LOG: begin iteration");
                for b in data.iter() {
                    println!("LOG: byte: {:08b}", b);
                }

                let mut response: Vec<u8> = Vec::new();

                match &forward_conn {
                    Some(c) => {
                        match forward_request(data, c) {
                            Ok(bytes) => response = bytes,
                            Err(e) => eprintln!("failed to forward request: {}", e),
                        }
                    },

                    None => {
                        match resolve_request(data) {
                            Ok(bytes) => response = bytes,
                            Err(e) => eprintln!("failed to resolve request: {}", e),
                        }
                    }
                }
                    
                udp_socket
                    .send_to(&response, source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn forward_request(data: Vec<u8>, forward_conn: &UdpSocket) -> Result<Vec<u8>, anyhow::Error>{
    let mut msg = parse_message(&data);
    let mut all_answers: Vec<ResourceRecord> = Vec::new();
    
    for q in msg.questions.iter() {
        let mut header = msg.header;
        header.qdcount = 1;
         
        let questions = vec![DNSQuestion{
            qname: q.qname.to_owned(), 
            qtype: q.qtype,
            qclass: q.qclass,
        }];

        let answers: Vec<ResourceRecord> = Vec::new();

        let msg_data = build_message(DNSMessage { header, questions, answers});

        match forward_conn.send(&msg_data) {
            Ok(_) => (),
            Err(e) => return Err(anyhow!("failed to send data to forward server: {e}",)),
        };
        
        let mut buf = [0; 512];
        let bytes_received = forward_conn.recv(&mut buf)?;
        let response_data: Vec<u8> = buf[..bytes_received].to_vec();

        let mut response = parse_message(&response_data);
        
        if response.header.ancount == 0 {
            response.answers = Vec::new();
        }

        all_answers.extend_from_slice(&response.answers);
    }

    msg.header.qr = types::QR::Response;
    msg.answers = all_answers;
    msg.header.ancount = msg.answers.len() as u16;

    Ok(build_message(msg))
}

fn resolve_request(data: Vec<u8>) -> Result<Vec<u8>, anyhow::Error> {
    let mut msg = parse_message(&data);
    msg.header.qr = types::QR::Response;
    msg.header.aa = false;
    msg.header.tc = false;
    msg.header.ra = false;
    msg.header.z = 0;

    for q in msg.questions.iter_mut() {
        q.qtype = types::RecordType::A;
        q.qclass = types::ClassType::IN;

        let answer = ResourceRecord {
            name: q.qname.to_owned(),
            record_type: types::RecordType::A,
            class: types::ClassType::IN,
            ttl: 0,
            rdlength: 4,
            rdata: vec![192, 168, 0, 6],
        };

        msg.answers.push(answer);
    }

    msg.header.ancount = msg.answers.len() as u16;
    
    Ok(build_message(msg))
}
