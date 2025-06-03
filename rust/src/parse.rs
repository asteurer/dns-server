use crate::types::{ClassType, DNSHeader, DNSMessage, DNSQuestion, Opcode, RecordType, ResourceRecord, QR, RCODE};

pub fn parse_message(data: &Vec<u8>) -> DNSMessage {
    let header_data: [u8; 12] = data[0..12].try_into().unwrap();
    let non_header_data: Vec<u8>= data[12..].to_vec();

    let mut header = parse_header(&header_data);
    let (questions, answer_idx) = parse_question(&non_header_data, header.qdcount);
    let (answers, _current_byte) = parse_record(&non_header_data, header.ancount, answer_idx);

    if header.opcode != Opcode::QUERY {
        header.rcode = RCODE::NotImplemented;
    }

    DNSMessage {
        header,
        questions,
        answers,
    }
}

fn parse_header(data: &[u8; 12]) -> DNSHeader {
    let id = u16::from_be_bytes([data[0], data[1]]);

    let byte_two = data[2];
    
    let qr = match byte_two & 0b1000_0000 {
        0 => QR::Query,
        _ => QR::Response, 
    };

    let opcode = match (byte_two & 0b0111_1000) >> 3 {
        0 => Opcode::QUERY,
        1 => Opcode::IQUERY,
        2 => Opcode::STATUS,
        _ => Opcode::Reserved,
    };

    let aa = match byte_two & 0b0000_0100 {
        0 => false,
        _ => true,
    };

    let tc = match byte_two & 0b0000_0010 {
        0 => false,
        _ => true,
    };

    let rd = match byte_two & 0b0000_0001 {
        0 => false,
        _ => true,
    };

    let byte_three= data[3];

    let ra = match byte_three & 0b1000_0000 {
        0 => false,
        _ => true,
    };

    let z = (byte_three & 0b0111_0000) >> 4;

    let rcode = match byte_three & 0b0000_1111 {
        0 => RCODE::NoError,
        1 => RCODE::FormatError,
        2 => RCODE::ServerFailure,
        3 => RCODE::NameError,
        4 => RCODE::NotImplemented,
        5 => RCODE::Refused,
        _ => RCODE::Reserved,
    };

    let qdcount = u16::from_be_bytes([data[4], data[5]]);
    let ancount = u16::from_be_bytes([data[8], data[7]]);
    let nscount = u16::from_be_bytes([data[8], data[9]]);
    let arcount = u16::from_be_bytes([data[10], data[11]]);

    DNSHeader{
        id, qr, opcode, aa, tc, rd, ra, z, rcode, 
        qdcount, ancount, nscount, arcount,
    }
}

fn parse_question(data: &Vec<u8>, num_questions: u16) -> (Vec<DNSQuestion>, usize){
    let mut questions: Vec<DNSQuestion> = Vec::new();
    let mut current_byte = 0;

    for _i in 0..num_questions {
        let (qname, mut start, is_pointer) = parse_domain(data, current_byte);
        if is_pointer {
            start += 2; 
        }


        current_byte = start + 1;

        let qtype = match u16::from_be_bytes([data[current_byte], data[current_byte + 1]]) {
            1 => RecordType::A,
            _ => RecordType::Other,
        };

        current_byte += 2;

        let qclass = match u16::from_be_bytes([data[current_byte], data[current_byte + 1]]) {
            1 => ClassType::IN,
            _ => ClassType::Other,
        };

        current_byte += 2;

        questions.push(DNSQuestion {qname, qtype, qclass});
    }

    (questions, current_byte)

}

fn parse_record(data: &Vec<u8>, num_answers: u16, current_byte: usize) -> (Vec<ResourceRecord>, usize){
    let mut records: Vec<ResourceRecord> = Vec::new();
    let mut current_byte = current_byte;

    for _i in 0..num_answers {
        let (name, mut start, is_pointer) = parse_domain(data, current_byte);
        if is_pointer {
            start += 1;
        }

        current_byte = start + 1;

        let record_type = match u16::from_be_bytes([data[current_byte], data[current_byte+1]]) {
            1 => RecordType::A,
            _ => RecordType::Other,
        };
        
        current_byte +=2 ;

        let class = match u16::from_be_bytes([data[current_byte], data[current_byte+1]]) {
            1 => ClassType::IN,
            _ => ClassType::Other,
        };

        current_byte += 2;

        let ttl = u32::from_be_bytes([data[current_byte], data[current_byte+1], data[current_byte+2], data[current_byte+3]]);

        current_byte += 4;

        let rdlength = u16::from_be_bytes([data[current_byte], data[current_byte+1]]);

        current_byte += 2;

        let rdata: Vec<u8> = data[current_byte..current_byte+rdlength as usize].to_vec();

        current_byte += rdlength as usize;

        records.push(ResourceRecord{name, record_type, class, ttl, rdlength, rdata});

    }

    (records, current_byte)

}

fn parse_domain(data: &Vec<u8>, start_idx: usize) -> (Vec<u8>, usize, bool) {
    let mut result: Vec<u8> = Vec::new();
    let mut byte_is_content = false;
    // let mut already_appended_null_byte = false;
    let mut is_pointer = false;
    let mut start_idx = start_idx;

    loop {
        let mut content_length: u8 = 0; 

        if !byte_is_content {
            if data[start_idx] & 0b1100_0000 == 0b1100_0000 {
                let pointer = u16::from_be_bytes([data[start_idx], data[start_idx + 1]]) & 0b0011_1111_1111_1111;
                let offset = (pointer - 12) as usize;
                let (decompressed_data, _, _) = parse_domain(data, offset);

                result.extend(decompressed_data);
                
                // already_appended_null_byte = true;
                is_pointer = true;
                break
            }

            content_length = data[start_idx];

            // if !already_appended_null_byte {
            result.push(content_length);
                // already_appended_null_byte = false;
            // }

            if content_length == 0 {
                break
            } else {
                start_idx += 1;
            }
        }

        let next_non_content_byte = start_idx + content_length as usize;
        for idx in start_idx..next_non_content_byte {
            result.push(data[idx]);
            start_idx += 1;
        }

        byte_is_content = false;
    }

    (result, start_idx, is_pointer)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_header() {
        struct Test {
            label: String,
            data: [u8; 12],
            want: DNSHeader,
        }

        let tests: Vec<Test> = vec![
            Test {
                label: "basic test".to_string(),
                data: [
                    0b0000_0100, 0b1101_0010, // ID: 1234
                    0b1010_1010, // QR: 1, Opcode: 5, AA: 0, TC: 1, RD: 0,
                    0b1101_0011, // RA: 1, Z: 5, RCODE: 3 
                    0b0000_0000, 0b0100_0101, // QDCOUNT: 69
                    0b0000_0000, 0b0100_0110, // ANCOUNT: 70
                    0b0000_0000, 0b0100_0111, // NSCOUNT: 71
                    0b0000_0000, 0b0100_1000, // ARCOUNT: 72
                ],
                want: DNSHeader { 
                    id: 1234, 
                    qr: QR::Response, 
                    opcode: Opcode::Reserved, 
                    aa: false, tc: true, rd: false, ra: true, z: 5, 
                    rcode: RCODE::NameError, 
                    qdcount: 69, ancount: 70, nscount: 71, arcount: 72 
                }
            }
        ];

        for t in tests {
            println!("Running test \"{}\"", t.label);
            let got = parse_header(&t.data);
            assert_eq!(t.want, got);
        }
    }

    #[test]
    fn test_parse_question() {
        struct Test {
            label: String,
            num_questions: u16,
            data: Vec<u8>,
            want_questions: Vec<DNSQuestion>,
            want_idx: usize,
        }

        let tests: Vec<Test> = vec![
            Test {
                label: "basic test".to_string(),
                num_questions: 1,
                data: vec![
                    0x07,
                    b'e', b'x', b'a', b'm', b'p', b'l', b'e',
                    0x03,
                    b'c', b'o', b'm',
                    0x00,
                    0x00, 0x01,
                    0x00, 0x03,
                ],

                want_questions: vec![
                    DNSQuestion {
                        qname: vec![
                            0x07,
                            b'e', b'x', b'a', b'm', b'p', b'l', b'e',
                            0x03,
                            b'c', b'o', b'm',
                            0x00,
                        ],
                        qtype: RecordType::A,
                        qclass: ClassType::Other,
                    }
                ],

                want_idx: 17,
            },

            Test {
                label: "pointer test".to_string(),
                num_questions: 3,
                data: vec![
                    0x07,
                    b'e', b'x', b'a', b'm', b'p', b'l', b'e',
                    0x03,
                    b'c', b'o', b'm',
                    0x00,
                    0x00, 0x00,
                    0x00, 0x00,
                    0x04,
                    b't', b'e', b's', b't',
                    0b1100_0000, 0b0000_1100, // Offset relative to a fictional header
                    0x00,
                    0x00, 0x00,
                    0x00, 0x00,
                    0x02,
                    b'i', b'o',
                    0x02,
                    b'i', b'o',
                    0x00,
                    0x00, 0x01,
                    0x00, 0x01,
                ],

                want_questions: vec![
                    DNSQuestion {
                        qname: vec![
                            0x07,
                            b'e', b'x', b'a', b'm', b'p', b'l', b'e',
                            0x03,
                            b'c', b'o', b'm',
                            0x00,
                        ],

                        qtype: RecordType::Other,
                        qclass: ClassType::Other,
                    },

                    DNSQuestion {
                        qname: vec![
                            0x04,
                            b't', b'e', b's', b't',
                            0x07,
                            b'e', b'x', b'a', b'm', b'p', b'l', b'e',
                            0x03,
                            b'c', b'o', b'm',
                            0x00,
                        ], 

                        qtype: RecordType::Other,
                        qclass: ClassType::Other,
                    }, 

                    DNSQuestion {
                        qname: vec![
                            0x02,
                            b'i', b'o',
                            0x02,
                            b'i', b'o',
                            0x00,
                        ],

                        qtype: RecordType::A,
                        qclass: ClassType::IN,
                    }
                ],

                want_idx: 40,

            },

            Test {
                label: "more pointer tests".to_string(),
                num_questions: 2,
                want_idx: 42,
                data: vec![
                    0b00000011, // abc
                    0b01100001, 
                    0b01100010, 
                    0b01100011, 
                    0b00010001, // longassdomainname
                    0b01101100, 
                    0b01101111, 
                    0b01101110, 
                    0b01100111, 
                    0b01100001, 
                    0b01110011, 
                    0b01110011, 
                    0b01100100, 
                    0b01101111, 
                    0b01101101, 
                    0b01100001, 
                    0b01101001, 
                    0b01101110, 
                    0b01101110, 
                    0b01100001, 
                    0b01101101, 
                    0b01100101, 
                    0b00000011, // com 
                    0b01100011, 
                    0b01101111, 
                    0b01101101, 
                    0b00000000, // null termination
                    0b00000000, 0b00000001, 
                    0b00000000, 0b00000001, 

                    0b00000011, // def
                    0b01100100, 
                    0b01100101, 
                    0b01100110, 
                    0b11000000, 0b00010000, // pointer
                    0x00, // null termination
                    0b00000000, 0b00000001, 
                    0b00000000, 0b00000001, 
                ],

                want_questions: vec![
                    DNSQuestion {
                        qname: vec![
                            0b00000011, // abc
                            0b01100001, 
                            0b01100010, 
                            0b01100011, 
                            0b00010001, // longassdomainname
                            0b01101100, 
                            0b01101111, 
                            0b01101110, 
                            0b01100111, 
                            0b01100001, 
                            0b01110011, 
                            0b01110011, 
                            0b01100100, 
                            0b01101111, 
                            0b01101101, 
                            0b01100001, 
                            0b01101001, 
                            0b01101110, 
                            0b01101110, 
                            0b01100001, 
                            0b01101101, 
                            0b01100101, 
                            0b00000011, // com 
                            0b01100011, 
                            0b01101111, 
                            0b01101101, 
                            0b00000000, // null termination 
                        ],
                        qtype: RecordType::A,
                        qclass: ClassType::IN,
                    },

                    DNSQuestion {
                        qname: vec![
                            0b00000011, // def
                            0b01100100, 
                            0b01100101, 
                            0b01100110, 
                            0b00010001, // longassdomainname
                            0b01101100, 
                            0b01101111, 
                            0b01101110, 
                            0b01100111, 
                            0b01100001, 
                            0b01110011, 
                            0b01110011, 
                            0b01100100, 
                            0b01101111, 
                            0b01101101, 
                            0b01100001, 
                            0b01101001, 
                            0b01101110, 
                            0b01101110, 
                            0b01100001, 
                            0b01101101, 
                            0b01100101, 
                            0b00000011, // com 
                            0b01100011, 
                            0b01101111, 
                            0b01101101, 
                            0b00000000, // null termination 
                        ],
                        qtype: RecordType::A,
                        qclass: ClassType::IN,
                    }
                ],
            }
        ];    
        
        for t in tests {
            println!("Running test \"{}\"", t.label);
            let (got_questions, got_idx) = parse_question(&t.data, t.num_questions);
            for (idx, got_q) in got_questions.iter().enumerate() {
                assert_eq!(*got_q, t.want_questions[idx]);
                assert_eq!(got_idx, t.want_idx);
            }
        }
    }

    #[test]
    fn test_parse_record() {
        struct Test {
            label: String,
                num_records: u16,
                current_byte: usize,
                data: Vec<u8>,
                want_answers: Vec<ResourceRecord>,
                want_idx: usize, 
        }

        let tests: Vec<Test> = vec![
            Test {
                label: "basic test".to_string(),
                num_records: 1,
                current_byte: 17,
                want_idx: 53,
                data: vec![
                    // QUESTION
                    0x07,
                    b'e', b'x', b'a', b'm', b'p', b'l', b'e',
                    0x03,
                    b'c', b'o', b'm',
                    0x00,
                    0x00, 0x00,
                    0x00, 0x00,
                    // ANSWER
                    // Name
                    0x07,
                    b'e', b'x', b'a', b'm', b'p', b'l', b'e',
                    0x03,
                    b'c', b'o', b'm',
                    0x00,
                    // Type
                    0b0000_0000,
                    0b0000_0001,
                    // Class
                    0b0000_0000,
                    0b0000_0001,
                    // TTL
                    0b0000_0000,
                    0b0000_0000,
                    0b0000_0000,
                    0b0000_0000,
                    // RDLENGTH
                    0b0000_0000,
                    0b0000_1101,
                    //RDATA
                    b'h', b'e', b'l', b'l', b'o', b',', b' ', b'w', b'o', b'r', b'l', b'd', b'!',
                ],

                want_answers: vec! [
                    ResourceRecord {
                        name: vec![
                             0x07,
                            b'e', b'x', b'a', b'm', b'p', b'l', b'e',
                            0x03,
                            b'c', b'o', b'm',
                            0x00,
                        ],
                        record_type: RecordType::A,
                        class: ClassType::IN,
                        ttl: 0,
                        rdlength: 13,
                        rdata: "hello, world!".to_string().into_bytes(),
                    }
                ],
            },

            Test {
                label: "pointer test".to_string(),
                num_records: 1,
                current_byte: 17,
                want_idx: 42,
                data: vec![
                     // QUESTION
                    0x07,
                    b'e', b'x', b'a', b'm', b'p', b'l', b'e',
                    0x03,
                    b'c', b'o', b'm',
                    0x00,
                    0x00, 0x00,
                    0x00, 0x00,
                    // ANSWER
                    // Name
                    0b1100_0000, 0b0000_1100, // Ofset relative to fictional header
                    // Type
                    0b0000_0000,
                    0b0000_0001,
                    // Class
                    0b0000_0000,
                    0b0000_0001,
                    // TTL
                    0b0000_0000,
                    0b0000_0000,
                    0b0000_0000,
                    0b0000_0000,
                    // RDLENGTH
                    0b0000_0000,
                    0b0000_1101,
                    //RDATA
                    b'h', b'e', b'l', b'l', b'o', b',', b' ', b'w', b'o', b'r', b'l', b'd', b'!',
                ], 

                want_answers: vec! [
                    ResourceRecord {
                        name: vec![
                            0x07,
                            b'e', b'x', b'a', b'm', b'p', b'l', b'e',
                            0x03,
                            b'c', b'o', b'm',
                            0x00,
                        ],
                        record_type: RecordType::A,
                        class: ClassType::IN,
                        ttl: 0,
                        rdlength: 13,
                        rdata: "hello, world!".to_string().into_bytes(),
                    }
                ],
            }
        ];

        for t in tests {
            println!("Running test \"{}\"", t.label);
            let (got_records, got_idx) = parse_record(&t.data, t.num_records, t.current_byte);
            for (idx, got_a) in got_records.iter().enumerate() {
                assert_eq!(*got_a, t.want_answers[idx]);
                assert_eq!(got_idx, t.want_idx);
            }
        }
    }
}