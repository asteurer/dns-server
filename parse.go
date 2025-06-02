package main

import (
	"encoding/binary"
)

func parseHeader(data []byte) Header {
	var header Header

	header.ID = binary.BigEndian.Uint16(data[:2])

	byteThree := data[2] // QR, OPCODE, AA, TC, RD
	header.QR = (byteThree & 0b1000_0000) != 0
	header.Opcode = (byteThree & 0b0111_1000) >> 3
	header.AA = (byteThree & 0b0000_0100) != 0
	header.TC = (byteThree & 0b0000_0010) != 0
	header.RD = (byteThree & 0b0000_0001) != 0

	byteFour := data[3] // RA, Z, RCODE
	header.RA = (byteFour & 0b1000_0000) != 0
	header.Z = (byteFour & 0b0111_0000) >> 4
	header.RCODE = (byteFour & 0b0000_1111)

	header.QDCOUNT = binary.BigEndian.Uint16(data[4:6])
	header.ANCOUNT = binary.BigEndian.Uint16(data[6:8])
	header.NSCOUNT = binary.BigEndian.Uint16(data[8:10])
	header.ARCOUNT = binary.BigEndian.Uint16(data[10:12])

	return header
}

func parseQuestions(data []byte, numQuestions uint16) ([]Question, uint16) {
	var questions []Question
	// This is the byte number corresponding to the byte after the last Header byte.
	// This will be incremented to the byte after the end of the question section
	currentByte := 12

	// This helps ensure we iterate through all questions
	var startIdx uint16

	for range numQuestions {
		var question Question

		// QNAME
		qname, start, isPointer := parseDomainName(data, startIdx)
		if isPointer {
			start += 2 // Increment past both pointer octets
		}

		// Increment to start of QTYPE
		currentByte = start + 1

		question.QNAME = qname

		// QTYPE & QCLASS
		question.QTYPE = binary.BigEndian.Uint16(data[currentByte : currentByte+2])

		// Increment to start of QCLASS
		currentByte += 2

		question.QCLASS = binary.BigEndian.Uint16(data[currentByte : currentByte+2])

		// Final increment to byte after current question
		currentByte += 2

		// Updating new startIdx for multiple questions
		// Incrementing start by 2 to keep pace with currentByte
		startIdx = uint16(currentByte)

		questions = append(questions, question)
	}

	return questions, uint16(currentByte)
}

func parseRecords(data []byte, numAnswers uint16, startIdx uint16) ([]ResourceRecord, uint16) {
	var records []ResourceRecord
	var currentByte uint16

	for range numAnswers {
		var record ResourceRecord
		name, start, isPointer := parseDomainName(data, startIdx /*wholeMsgOffset*/)
		if isPointer {
			start += 1 // Increment past pointer octet
		}

		record.Name = name

		currentByte = uint16(start + 1)

		record.Type = binary.BigEndian.Uint16(data[currentByte : currentByte+2])
		currentByte += 2

		record.Class = binary.BigEndian.Uint16(data[currentByte : currentByte+2])
		currentByte += 2

		record.TTL = binary.BigEndian.Uint32(data[currentByte : currentByte+4])
		currentByte += 4

		record.RDLENGTH = binary.BigEndian.Uint16(data[currentByte : currentByte+2])
		currentByte += 2

		record.RDATA = data[currentByte : currentByte+record.RDLENGTH]
		currentByte += record.RDLENGTH

		startIdx = currentByte

		records = append(records, record)
	}

	return records, currentByte
}

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

func parseDomainName(data []byte, startIdx uint16) ([]byte, int, bool) {
	result := []byte{}
	byteIsContent := false
	alreadyAppendedNullByte := false
	// Resource records and questions handle null termination for pointers differently, so this will help
	// each respective function handle the pointers effectively
	isPointer := false
	for {
		var labelLen byte

		if !byteIsContent {
			if (data[startIdx] & 0b1100_0000) == 0b1100_0000 {
				pointer := (binary.BigEndian.Uint16([]byte{data[startIdx], data[startIdx+1]}) & 0b0011_1111_1111_1111)
				offset := byte(pointer) - 12 // Since we sliced off the header data, we need to adjust the pointerOffset so that it is pointing to the correct data
				decompressedData, _, _ := parseDomainName(data, uint16(offset))
				result = append(result, decompressedData...)
				alreadyAppendedNullByte = true
				isPointer = true
				break
			}

			labelLen = data[startIdx]

			// This helps make sure we don't add duplicate null termination bytes to the result array
			if !alreadyAppendedNullByte {
				result = append(result, labelLen)
				alreadyAppendedNullByte = false
			}

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

	return result, int(startIdx), isPointer
}
