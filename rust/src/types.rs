#[derive(Clone)]
pub struct DNSMessage {
    pub header: DNSHeader,
    pub questions: Vec<DNSQuestion>,
    pub answers: Vec<ResourceRecord>,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct DNSHeader {
    pub id: u16,
    pub qr: QR, // 1 bit
    pub opcode: Opcode, // 4 bits
    pub aa: bool, // 1 bit
    pub tc: bool, // 1 bit
    pub rd: bool, // 1 bit
    pub ra: bool, // 1 bit
    pub z: u8, // 3 bits
    pub rcode: RCODE, // 4 bits
    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16,
    pub arcount: u16,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum QR {
    Query,
    Response,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Opcode {
    QUERY,
    IQUERY,
    STATUS,
    Reserved,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum RCODE {
    NoError,
    FormatError,
    ServerFailure,
    NameError,
    NotImplemented,
    Refused,
    Reserved,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct DNSQuestion {
    pub qname: Vec<u8>,
    pub qtype: RecordType,
    pub qclass: ClassType,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ResourceRecord {
    pub name: Vec<u8>,
    pub record_type: RecordType,
    pub class: ClassType,
    pub ttl: u32,
    pub rdlength: u16,
    pub rdata: Vec<u8>,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum RecordType {
    A,
    Other,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ClassType {
    IN,
    Other,
}