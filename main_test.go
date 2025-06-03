package main

import (
	"fmt"
	"testing"
)

func TestBuildHeader(t *testing.T) {
	tests := []struct {
		label string // Helps differentiate the tests
		h     Header
		want  []byte
	}{
		{
			label: "basic test 1",
			h: Header{
				ID:     0b0000_0100_1101_0010,
				QR:     false,
				Opcode: 0b1111,
				AA:     false,
				TC:     true,
				RD:     false,
				RA:     true,
				Z:      0b101,
				RCODE:  0b1100,
			},

			want: []byte{
				0b0000_0100,
				0b1101_0010,
				0b0111_1010,
				0b1101_1100,
				0b0000_0000,
				0b0000_0000,
				0b0000_0000,
				0b0000_0000,
				0b0000_0000,
				0b0000_0000,
				0b0000_0000,
				0b0000_0000,
			},
		},
		{
			label: "basic test 2",
			h: Header{
				ID:     0b0000_0000_0100_0101,
				QR:     true,
				Opcode: 0b1010,
				AA:     true,
				TC:     false,
				RD:     true,
				RA:     false,
				Z:      0b110,
				RCODE:  0b0011,
			},

			want: []byte{
				0b0000_0000,
				0b0100_0101,
				0b1101_0101,
				0b0110_0011,
				0b0000_0000,
				0b0000_0000,
				0b0000_0000,
				0b0000_0000,
				0b0000_0000,
				0b0000_0000,
				0b0000_0000,
				0b0000_0000,
			},
		},
	}

	for _, test := range tests {
		fmt.Println("Running test:", test.label)
		got := buildHeader(test.h)
		for idx, wantEntry := range test.want {
			if got[idx] != wantEntry {
				t.Errorf("test %q failed:\nwant:\n%v\n\ngot:\n%v\n", test.label, test.want, got)
				break
			}
		}
	}
}

func TestBuildQuestions(t *testing.T) {
	tests := []struct {
		label     string // Helps differentiate the tests
		questions []Question
		want      []byte
	}{
		{
			label: "basic test",
			questions: []Question{
				{
					QNAME: []byte{
						0x07,
						'e', 'x', 'a', 'm', 'p', 'l', 'e',
						0x03,
						'c', 'o', 'm',
						0x00,
					},
					QTYPE:  1,
					QCLASS: 2,
				},
			},

			want: []byte{
				0x07,
				'e', 'x', 'a', 'm', 'p', 'l', 'e',
				0x03,
				'c', 'o', 'm',
				0x00,
				0x00,
				0x01,
				0x00,
				0x02,
			},
		},
	}

	for _, test := range tests {
		fmt.Println("Running test:", test.label)
		got := buildQuestions(test.questions)
		for idx, gotEntry := range got {
			if test.want[idx] != gotEntry {
				t.Errorf("test %q failed:\nwant:\n%v\n\ngot:\n%v\n", test.label, test.want, got)
				break
			}
		}
	}
}

func TestBuildRecords(t *testing.T) {
	tests := []struct {
		label   string // Helps differentiate the tests
		records []ResourceRecord
		want    []byte
	}{
		{
			label: "basic test",
			records: []ResourceRecord{
				{
					Name: []byte{
						0x07,
						'e', 'x', 'a', 'm', 'p', 'l', 'e',
						0x03,
						'c', 'o', 'm',
						0x00,
					},
					Type:     0b1,
					Class:    0b1,
					TTL:      0b0,
					RDLENGTH: 13,
					RDATA:    []byte("hello, world!"),
				},
			},
			want: []byte{
				// Name
				0x07,
				'e', 'x', 'a', 'm', 'p', 'l', 'e',
				0x03,
				'c', 'o', 'm',
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
				'h', 'e', 'l', 'l', 'o', ',', ' ', 'w', 'o', 'r', 'l', 'd', '!',
			},
		},
	}

	for _, test := range tests {
		fmt.Println("Running test:", test.label)
		got := buildRecords(test.records)
		for idx, gotEntry := range got {
			if gotEntry != test.want[idx] {
				t.Errorf("test %q failed:\nwant:\n%v\n\ngot:\n%v\n", test.label, test.want, got)
				break
			}
		}
	}
}

func TestParseHeader(t *testing.T) {
	tests := []struct {
		label string // Helps differentiate the tests
		data  []byte
		want  Header
	}{
		{
			label: "basic test",
			data: []byte{
				0b0000_0100,
				0b1101_0010,
				0b0111_1010,
				0b1101_1100,
				0b0000_0000,
				0b0000_0000,
				0b0000_0000,
				0b0000_0000,
				0b0000_0000,
				0b0000_0000,
				0b0000_0000,
				0b0000_0000,
			},

			want: Header{
				ID:     0b0000_0100_1101_0010,
				QR:     false,
				Opcode: 0b1111,
				AA:     false,
				TC:     true,
				RD:     false,
				RA:     true,
				Z:      0b101,
				RCODE:  0b1100,
			},
		},
	}

	for _, test := range tests {
		fmt.Println("Running test:", test.label)
		got := Message{Header: parseHeader(test.data)}
		if !got.Equals(Message{Header: test.want}) {
			t.Errorf("test %q failed:\nwant:\n%v\n\ngot:\n%v\n", test.label, test.want, got)
			break
		}
	}
}

func TestParseQuestions(t *testing.T) {
	tests := []struct {
		label         string
		data          []byte
		numQuestions  uint16
		wantQuestions []Question
		wantIdx       uint16
	}{
		{
			label: "basic test",
			data: []byte{
				0x07,
				'e', 'x', 'a', 'm', 'p', 'l', 'e',
				0x03,
				'c', 'o', 'm',
				0x00,
				0x00, 0x01,
				0x00, 0x02,
			},

			numQuestions: 1,

			wantQuestions: []Question{
				{
					QNAME: []byte{
						0x07,
						'e', 'x', 'a', 'm', 'p', 'l', 'e',
						0x03,
						'c', 'o', 'm',
						0x00,
					},
					QTYPE:  1,
					QCLASS: 2,
				},
			},

			wantIdx: 17,
		},
		{
			label: "testing pointer",
			data: []byte{
				0x07,
				'e', 'x', 'a', 'm', 'p', 'l', 'e',
				0x03,
				'c', 'o', 'm',
				0x00,
				0x00, 0x00,
				0x00, 0x00,
				0x04,
				't', 'e', 's', 't',
				0b1100_0000, 0b0000_1100, // Offset relative to a fictional header
				0x00,
				0x00, 0x00,
				0x00, 0x00,
				0x02,
				'i', 'o',
				0x02,
				'i', 'o',
				0x00,
				0x00, 0x00,
				0x00, 0x00,
			},

			numQuestions: 3,

			wantQuestions: []Question{
				{
					QNAME: []byte{
						0x07,
						'e', 'x', 'a', 'm', 'p', 'l', 'e',
						0x03,
						'c', 'o', 'm',
						0x00,
					},
				},
				{
					QNAME: []byte{
						0x04,
						't', 'e', 's', 't',
						0x07,
						'e', 'x', 'a', 'm', 'p', 'l', 'e',
						0x03,
						'c', 'o', 'm',
						0x00,
					},
				},
				{
					QNAME: []byte{
						0x02,
						'i', 'o',
						0x02,
						'i', 'o',
						0x00,
					},
				},
			},

			wantIdx: 40,
		},
	}

	for _, test := range tests {
		fmt.Println("Running test:", test.label)
		gotQuestions, gotIdx := parseQuestions(test.data, test.numQuestions)
		msg := Message{Questions: gotQuestions}

		if !msg.Equals(Message{Questions: test.wantQuestions}) {
			t.Errorf("test %q failed:\nwantQuestion:\n%v\n\ngotQuestion:\n%v\n", test.label, test.wantQuestions, gotQuestions)
			break
		}

		if gotIdx != test.wantIdx {
			t.Errorf("test %q failed: \nwantIdx: %v\ngotIdx: %v\n", test.label, test.wantIdx, gotIdx)
			break
		}
	}
}

func TestParseRecords(t *testing.T) {
	tests := []struct {
		label       string // Helps differentiate the tests
		wantRecords []ResourceRecord
		numRecords  uint16
		startIdx    uint16
		wantIdx     uint16
		data        []byte
	}{
		{
			label: "basic test",
			wantRecords: []ResourceRecord{
				{
					Name: []byte{
						0x07,
						'e', 'x', 'a', 'm', 'p', 'l', 'e',
						0x03,
						'c', 'o', 'm',
						0x00,
					},
					Type:     0b1,
					Class:    0b1,
					TTL:      0b0,
					RDLENGTH: 13,
					RDATA:    []byte("hello, world!"),
				},
			},

			numRecords: 1,
			startIdx:   17,
			wantIdx:    53,

			data: []byte{
				// QUESTION
				0x07,
				'e', 'x', 'a', 'm', 'p', 'l', 'e',
				0x03,
				'c', 'o', 'm',
				0x00,
				0x00, 0x00,
				0x00, 0x00,
				// ANSWER
				// Name
				0x07,
				'e', 'x', 'a', 'm', 'p', 'l', 'e',
				0x03,
				'c', 'o', 'm',
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
				'h', 'e', 'l', 'l', 'o', ',', ' ', 'w', 'o', 'r', 'l', 'd', '!',
			},
		},
		{
			label: "pointer test",
			wantRecords: []ResourceRecord{
				{
					Name: []byte{
						0x07,
						'e', 'x', 'a', 'm', 'p', 'l', 'e',
						0x03,
						'c', 'o', 'm',
						0x00,
					},
					Type:     0b1,
					Class:    0b1,
					TTL:      0b0,
					RDLENGTH: 13,
					RDATA:    []byte("hello, world!"),
				},
			},
			numRecords: 1,
			startIdx:   17,
			wantIdx:    42,
			data: []byte{
				// QUESTION
				0x07,
				'e', 'x', 'a', 'm', 'p', 'l', 'e',
				0x03,
				'c', 'o', 'm',
				0x00,
				0x00, 0x00,
				0x00, 0x00,

				// ANSWER
				// Name
				0b1100_0000, 0b0000_1100, // Offset relative to a fictional header
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
				'h', 'e', 'l', 'l', 'o', ',', ' ', 'w', 'o', 'r', 'l', 'd', '!',
			},
		},
	}

	for _, test := range tests {
		fmt.Println("Running test:", test.label)
		gotAnswers, gotIdx := parseRecords(test.data, test.numRecords, test.startIdx)
		got := Message{Answers: gotAnswers}
		if !got.Equals(Message{Answers: test.wantRecords}) {
			t.Errorf("test %q failed:\nwantRecords:\n%v\n\ngotRecords:\n%v\n", test.label, test.wantRecords, gotAnswers)
			break
		}

		if gotIdx != test.wantIdx {
			t.Errorf("test %q failed:\nwantIdx:\n%v\n\ngotIdx:\n%v\n", test.label, test.wantIdx, gotIdx)
			break
		}
	}
}
