package main

import (
	"encoding/binary"
)

func parseMessage(data []byte) Message {
	header := parseHeader(data[0:12])
	questions, ansIdx := parseQuestions(data[12:], header.QDCOUNT)
	answers, _ := parseRecords(data[12:], header.ANCOUNT, ansIdx)

	// This server only handles standard queries, so we need to indicate that other request types aren't handled
	if header.Opcode != 0 {
		header.RCODE = 4
	}

	return Message{
		Header:    header,
		Questions: questions,
		Answers:   answers,
	}
}

func parseHeader(data []byte) Header {
	var header Header

	header.ID = binary.BigEndian.Uint16(data[:2])

	byteTwo := data[2] // QR, OPCODE, AA, TC, RD
	header.QR = (byteTwo & 0b1000_0000) != 0
	header.Opcode = (byteTwo & 0b0111_1000) >> 3
	header.AA = (byteTwo & 0b0000_0100) != 0
	header.TC = (byteTwo & 0b0000_0010) != 0
	header.RD = (byteTwo & 0b0000_0001) != 0

	byteThree := data[3] // RA, Z, RCODE
	header.RA = (byteThree & 0b1000_0000) != 0
	header.Z = (byteThree & 0b0111_0000) >> 4
	header.RCODE = (byteThree & 0b0000_1111)

	header.QDCOUNT = binary.BigEndian.Uint16(data[4:6])
	header.ANCOUNT = binary.BigEndian.Uint16(data[6:8])
	header.NSCOUNT = binary.BigEndian.Uint16(data[8:10])
	header.ARCOUNT = binary.BigEndian.Uint16(data[10:12])

	return header
}

func parseQuestions(data []byte, numQuestions uint16) ([]Question, uint16) {
	var questions []Question
	currentByte := 0

	for range numQuestions {
		var question Question

		qname, start := parseDomainName(data, uint16(currentByte))

		currentByte = start + 1 // Increment to start of QTYPE

		question.QNAME = qname

		question.QTYPE = binary.BigEndian.Uint16(data[currentByte : currentByte+2])

		currentByte += 2 // Increment to start of QCLASS

		question.QCLASS = binary.BigEndian.Uint16(data[currentByte : currentByte+2])

		currentByte += 2 // Final increment to byte after current question

		questions = append(questions, question)
	}

	return questions, uint16(currentByte)
}

func parseRecords(data []byte, numAnswers uint16, currentByte uint16) ([]ResourceRecord, uint16) {
	var records []ResourceRecord

	for range numAnswers {
		var record ResourceRecord
		name, start := parseDomainName(data, currentByte)

		record.Name = name

		currentByte = uint16(start + 1) // Increment to start of TYPE

		record.Type = binary.BigEndian.Uint16(data[currentByte : currentByte+2])
		currentByte += 2 // Increment to start of CLASS

		record.Class = binary.BigEndian.Uint16(data[currentByte : currentByte+2])
		currentByte += 2 // Increment to start of TTL

		record.TTL = binary.BigEndian.Uint32(data[currentByte : currentByte+4])
		currentByte += 4 // Increment to start of RDLENGTH

		record.RDLENGTH = binary.BigEndian.Uint16(data[currentByte : currentByte+2])
		currentByte += 2 // Increment to start of RDATA

		record.RDATA = data[currentByte : currentByte+record.RDLENGTH]
		currentByte += record.RDLENGTH // Increment to byte after current record

		records = append(records, record)
	}

	return records, currentByte
}

func parseDomainName(data []byte, startIdx uint16) ([]byte, int) {
	result := []byte{}
	byteIsContent := false
	for {
		var labelLen byte

		if !byteIsContent {
			if (data[startIdx] & 0b1100_0000) == 0b1100_0000 {
				pointer := (binary.BigEndian.Uint16([]byte{data[startIdx], data[startIdx+1]}) & 0b0011_1111_1111_1111)
				offset := byte(pointer) - 12 // Since we sliced off the header data, we need to adjust the pointerOffset so that it is pointing to the correct data
				decompressedData, _ := parseDomainName(data, uint16(offset))
				result = append(result, decompressedData...)
				startIdx++ // Increment to the 2nd (last) byte of the pointer
				break
			}

			labelLen = data[startIdx]

			result = append(result, labelLen)

			if labelLen == 0 { // Null termination byte
				break
			} else {
				startIdx++
			}
		}

		nextNonContentByte := startIdx + uint16(labelLen)
		for idx := startIdx; idx < nextNonContentByte; idx++ {
			result = append(result, data[idx])
			startIdx++
		}

		byteIsContent = false
	}

	return result, int(startIdx)
}
