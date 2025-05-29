package main

type Header struct {
	// 2 bytes
	ID uint16

	// 1 byte
	QR     bool
	Opcode uint8 // 4 bits
	AA     bool
	TC     bool
	RD     bool

	// 1 byte
	RA    bool
	Z     uint8 // 3 bits
	RCODE uint8 // 4 bits

	// 2 bytes each...
	QDCOUNT uint16
	ANCOUNT uint16
	NSCOUNT uint16
	ARCOUNT uint16
}

type Question struct {
	QNAME  []byte
	QTYPE  uint16
	QCLASS uint16
}

type ResourceRecord struct {
	Name     []byte
	Type     uint16
	Class    uint16
	TTL      uint32
	RDLENGTH uint16 // Length of RDATA
	RDATA    []byte
}

type Message struct {
	Header    Header
	Questions []Question
	Answers   []ResourceRecord
	// Authority  ResourceRecord
	// Additional ResourceRecord
}

func (m1 *Message) Equals(m2 Message) bool {
	// Compare Header fields
	if !(m1.Header.ID == m2.Header.ID &&
		m1.Header.QR == m2.Header.QR &&
		m1.Header.Opcode == m2.Header.Opcode &&
		m1.Header.AA == m2.Header.AA &&
		m1.Header.TC == m2.Header.TC &&
		m1.Header.RD == m2.Header.RD &&
		m1.Header.RA == m2.Header.RA &&
		m1.Header.Z == m2.Header.Z &&
		m1.Header.RCODE == m2.Header.RCODE &&
		m1.Header.QDCOUNT == m2.Header.QDCOUNT &&
		m1.Header.ANCOUNT == m2.Header.ANCOUNT &&
		m1.Header.NSCOUNT == m2.Header.NSCOUNT &&
		m1.Header.ARCOUNT == m2.Header.ARCOUNT) {
		return false
	}

	if len(m1.Questions) != len(m2.Questions) {
		return false
	}

	for idx, m1Question := range m1.Questions {
		m2Question := m2.Questions[idx]

		if len(m1Question.QNAME) != len(m2Question.QNAME) {
			return false
		}

		for i := range m1Question.QNAME {
			if m1Question.QNAME[i] != m2Question.QNAME[i] {
				return false
			}
		}

		if m1Question.QTYPE != m2Question.QTYPE ||
			m1Question.QCLASS != m2Question.QCLASS {
			return false
		}
	}

	return true
}
