use crate::types::{ClassType, DNSHeader, DNSMessage, DNSQuestion, Opcode, RecordType, ResourceRecord, QR, RCODE};

pub fn build_message(message: DNSMessage) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    result.extend_from_slice(&build_header(message.header));
    result.extend_from_slice(&build_questions(message.questions));
    result.extend_from_slice(&build_records(message.answers));

    result
}

fn build_header(h: DNSHeader) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();

    // Place the ID
    result.extend_from_slice(&h.id.to_be_bytes());

    let mut byte_three: u8;

    match h.qr {
        QR::Response => byte_three = 0b1000_0000,
        QR::Query => byte_three = 0,
    }

    let opcode: u8 = match h.opcode {
        Opcode::QUERY => 0,
        Opcode::IQUERY => 1,
        Opcode::STATUS => 2,
        Opcode::Reserved => 3, // Technically, this should encompass 3-15, but I don't care enough to implement the others
    };

    byte_three = (opcode << 3) | byte_three;

    if h.aa {
        byte_three = byte_three | 0b0000_0100;
    }

    if h.tc {
        byte_three = byte_three | 0b0000_0010;
    }

    if h.rd {
        byte_three = byte_three | 0b0000_0001;
    }

    result.push(byte_three);

    let mut byte_four: u8 = 0;

    if h.ra {
        byte_four = 0b1000_0000;
    }

    byte_four = (h.z << 4) | byte_four;

    let rcode: u8 = match h.rcode {
        RCODE::NoError => 0,
        RCODE::FormatError => 1,
        RCODE::ServerFailure => 2,
        RCODE::NameError => 3,
        RCODE::NotImplemented => 4,
        RCODE::Refused => 5,
        RCODE::Reserved => 6, // Technically, this should encompass 6-15, but I don't care enough to implement the others
    };

    byte_four = rcode | byte_four;

    result.push(byte_four);

    result.extend_from_slice(&h.qdcount.to_be_bytes());
    result.extend_from_slice(&h.ancount.to_be_bytes());
    result.extend_from_slice(&h.nscount.to_be_bytes());
    result.extend_from_slice(&h.arcount.to_be_bytes());

    result

}

fn build_questions(questions: Vec<DNSQuestion>) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    for q in questions.iter() {
        result.extend_from_slice(&q.qname);

        let qtype: u16 = match q.qtype {
            RecordType::A => 1,
            RecordType::Other => 0, // This is wrong, but I only care about A-type records
        };

        result.extend_from_slice(&qtype.to_be_bytes());

        let qclass: u16 = match q.qclass {
            ClassType::IN => 1,
            ClassType::Other => 0, // This is wrong, but I only care about the IN class
        };

        result.extend_from_slice(&qclass.to_be_bytes());
    }

    result
}

fn build_records(records: Vec<ResourceRecord>) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    for r in records.iter() {
        result.extend_from_slice(&r.name);

        let record_type: u16 = match r.record_type {
            RecordType::A => 1,
            RecordType::Other => 0,
        };

        result.extend_from_slice(&record_type.to_be_bytes());

        let class: u16 = match r.class {
            ClassType::IN => 1,
            ClassType::Other => 0,
        };

        result.extend_from_slice(&class.to_be_bytes());
        result.extend_from_slice(&r.ttl.to_be_bytes());
        result.extend_from_slice(&r.rdlength.to_be_bytes());
        result.extend_from_slice(&r.rdata);
    }

    result
}

mod tests {
    use super::*;

    // This is makes errors for non-matching byte arrays more helpful
    macro_rules! assert_bytes_eq {
        ($left:expr, $right:expr) => {
            if $left != $right {
                eprintln!("Byte arrays differ:");
                eprintln!("left           | right");
                eprintln!("---------------|--------------");
                let max_len = std::cmp::max($left.len(), $right.len());
                for i in 0..max_len {
                    let left_byte = $left.get(i).map(|b| format!("{:08b}", b)).unwrap_or("--".to_string());
                    let right_byte = $right.get(i).map(|b| format!("{:08b}", b)).unwrap_or("--".to_string());
                    let marker = if $left.get(i) != $right.get(i) { " <--" } else { "" };
                    eprintln!("   {:2}    |   {:2}   {}", left_byte, right_byte, marker);
                }
                panic!("Assertion failed: byte arrays not equal");
            }
        };
    }

    #[test]
    fn test_build_header() {
        struct Test {
            label: String,
            h: DNSHeader,
            want: Vec<u8>,
        }

        let tests: Vec<Test> = vec![
            Test {
                label: "basic test 1".to_string(),
                h: DNSHeader {
                    id:     0b0000_0100_1101_0010,
                    qr:     QR::Query,
                    opcode: Opcode::STATUS,
                    aa:     false,
                    tc:     true,
                    rd:     false,
                    ra:     true,
                    z:      0b101,
                    rcode:  RCODE::NotImplemented,
                    qdcount: 0, ancount: 0, nscount: 0, arcount: 0,
                },
                want: vec![
                    0b0000_0100,
                    0b1101_0010,
                    0b0001_0010,
                    0b1101_0100,
                    0b0000_0000,
                    0b0000_0000,
                    0b0000_0000,
                    0b0000_0000,
                    0b0000_0000,
                    0b0000_0000,
                    0b0000_0000,
                    0b0000_0000,
                ],
            },

            Test {
                label: "basic test 2".to_string(),
                h: DNSHeader {
                    id:     0b0000_0000_0100_0101,
                    qr:     QR::Query,
                    opcode: Opcode::IQUERY,
                    aa:     true,
                    tc:     false,
                    rd:     true,
                    ra:     false,
                    z:      0b110,
                    rcode:  RCODE::NameError,
                    qdcount: 0, ancount: 0, nscount: 0, arcount: 0,
                },

                want: vec![
                    0b0000_0000,
                    0b0100_0101,
                    0b0000_1101,
                    0b0110_0011,
                    0b0000_0000,
                    0b0000_0000,
                    0b0000_0000,
                    0b0000_0000,
                    0b0000_0000,
                    0b0000_0000,
                    0b0000_0000,
                    0b0000_0000,
                ],
            },
        ];

        for t in tests {
            println!("Running test \"{}\"", t.label);
            let got_data = build_header(t.h);
            assert_bytes_eq!(got_data, t.want);
        }
    }

    #[test]
    fn test_build_questions() {
        struct Test {
            label: String,
            questions: Vec<DNSQuestion>,
            want: Vec<u8>,
        }

        let tests: Vec<Test> = vec![
            Test {
                label: "basic test 1".to_string(),
                questions: vec![
                    DNSQuestion {
                        qname: vec![
                            0x07,
                            b'e', b'x', b'a', b'm', b'p', b'l', b'e',
                            0x03,
                            b'c', b'o', b'm',
                            0x00,
                        ],

                        qtype: RecordType::A,
                        qclass: ClassType::IN,
                    }
                ],

                want: vec![
                    0x07,
                    b'e', b'x', b'a', b'm', b'p', b'l', b'e',
                    0x03,
                    b'c', b'o', b'm',
                    0x00,
                    0x00, 0x01,
                    0x00, 0x01,
                ]
            }
        ];

        for t in tests {
            println!("Running test \"{}\"", t.label);
            let got_data = build_questions(t.questions);
            assert_bytes_eq!(got_data, t.want);
        }
    }

    #[test]
    fn test_build_records() {
        struct Test {
            label: String,
            records: Vec<ResourceRecord>,
            want: Vec<u8>,
        }

        let tests: Vec<Test> = vec![Test {
                label: "basic test".to_string(),
                records: vec![ResourceRecord{
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
                }],

                want: vec![
                    0x07,
                    b'e', b'x', b'a', b'm', b'p', b'l', b'e',
                    0x03,
                    b'c', b'o', b'm',
                    0x00,
                    0x00, 0x01,
                    0x00, 0x01,
                    0x00, 0x00, 0x00, 0x00,
                    0x00, 0b0000_1101,
                    b'h', b'e', b'l', b'l', b'o', b',', b' ', b'w', b'o', b'r', b'l', b'd', b'!',
                ],
        }];

        for t in tests {
            println!("Running test \"{}\"", t.label);
            let got_data = build_records(t.records);
            assert_bytes_eq!(got_data, t.want);
        }
    }
}