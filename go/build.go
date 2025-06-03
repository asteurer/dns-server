package main

import (
	"encoding/binary"
)

func buildMessage(m Message) []byte {
	result := []byte{}
	result = append(result, buildHeader(m.Header)...)
	result = append(result, buildQuestions(m.Questions)...)
	result = append(result, buildRecords(m.Answers)...)

	return result
}

func buildHeader(h Header) []byte {
	result := []byte{}

	buf := make([]byte, 2)

	// Place ID (bytes 1 and 2)
	binary.BigEndian.PutUint16(buf, h.ID)
	result = append(result, buf...)

	// Place QR, Opcode, AA, TC, RD
	var byteThree uint8

	if h.QR {
		byteThree = 0b1000_0000
	}

	byteThree = (h.Opcode << 3) | byteThree

	if h.AA {
		byteThree = byteThree | 0b0000_0100
	}

	if h.TC {
		byteThree = byteThree | 0b0000_0010
	}

	if h.RD {
		byteThree = byteThree | 0b0000_0001
	}

	result = append(result, byteThree)

	// Place RA, Z, RCODE
	var byteFour uint8

	if h.RA {
		byteFour = 0b1000_0000
	}

	byteFour = (h.Z << 4) | byteFour

	byteFour = (h.RCODE) | byteFour

	result = append(result, byteFour)

	// Place remaining bytes
	binary.BigEndian.PutUint16(buf, h.QDCOUNT)
	result = append(result, buf...)

	binary.BigEndian.PutUint16(buf, h.ANCOUNT)
	result = append(result, buf...)

	binary.BigEndian.PutUint16(buf, h.NSCOUNT)
	result = append(result, buf...)

	binary.BigEndian.PutUint16(buf, h.ARCOUNT)
	result = append(result, buf...)

	return result
}

func buildQuestions(questions []Question) []byte {
	result := []byte{}
	buf := make([]byte, 2)

	for _, q := range questions {
		result = append(result, q.QNAME...)

		binary.BigEndian.PutUint16(buf, q.QTYPE)
		result = append(result, buf...)

		binary.BigEndian.PutUint16(buf, q.QCLASS)
		result = append(result, buf...)

	}

	return result
}

func buildRecords(records []ResourceRecord) []byte {
	result := []byte{}
	buf := make([]byte, 2)

	for _, r := range records {
		result = append(result, r.Name...)

		binary.BigEndian.PutUint16(buf, r.Type)
		result = append(result, buf...)

		binary.BigEndian.PutUint16(buf, r.Class)
		result = append(result, buf...)

		fourBuf := make([]byte, 4)
		binary.BigEndian.PutUint32(fourBuf, uint32(r.TTL))
		result = append(result, fourBuf...)

		binary.BigEndian.PutUint16(buf, r.RDLENGTH)
		result = append(result, buf...)

		result = append(result, r.RDATA...)
	}

	return result
}
