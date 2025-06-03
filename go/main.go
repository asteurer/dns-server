package main

import (
	"fmt"
	"net"
	"os"
)

func main() {
	args := os.Args
	isForwardMode := len(args) >= 3 && args[1] == "--resolver"
	var forwardConn *net.UDPConn
	if isForwardMode {
		fmt.Println("Forwarding queries to", args[2])
		forwardAddress, err := net.ResolveUDPAddr("udp", args[2])
		if err != nil {
			fmt.Println("Failed to resolve UDP address:", err)
			return
		}

		conn, err := net.DialUDP("udp", nil, forwardAddress)
		if err != nil {
			fmt.Println("Failed to resolve forwarding address:", err)
			return
		}

		forwardConn = conn

	} else {
		fmt.Println("Running in resolve mode")
	}

	fmt.Println("Server listening on 127.0.0.1:2053")

	udpAddr, err := net.ResolveUDPAddr("udp", "127.0.0.1:2053")
	if err != nil {
		fmt.Println("Failed to resolve UDP address:", err)
		return
	}

	udpConn, err := net.ListenUDP("udp", udpAddr)
	if err != nil {
		fmt.Println("Failed to bind to address:", err)
		return
	}
	defer udpConn.Close()

	buf := make([]byte, 512)

	for {
		dataLen, source, err := udpConn.ReadFromUDP(buf)
		if err != nil {
			fmt.Println("Error receiving data:", err)
			break
		}

		data := buf[:dataLen]

		var response []byte

		if isForwardMode {
			resp, err := forwardRequest(data, forwardConn)
			if err != nil {
				fmt.Println("Failed to forward response:", err)
				continue
			}

			response = resp

		} else {
			response = resolveRequest(data)
		}
		_, err = udpConn.WriteToUDP(response, source)
		if err != nil {
			fmt.Println("Failed to send response:", err)
		}
	}
}

func forwardRequest(data []byte, forwardConn *net.UDPConn) ([]byte, error) {
	msg := parseMessage(data)

	var answers []ResourceRecord

	splitMsg := Message{
		Header: msg.Header,
	}
	splitMsg.Header.QDCOUNT = 1

	for _, question := range msg.Questions {
		splitMsg.Questions = []Question{question}

		msgData := buildMessage(splitMsg)

		if _, err := forwardConn.Write(msgData); err != nil {
			return []byte{}, fmt.Errorf("failed to forward message to resolver: %v", err)
		}

		buf := make([]byte, 512)
		dataLen, err := forwardConn.Read(buf)
		if err != nil {
			return []byte{}, fmt.Errorf("error reading response: %v", err)
		}

		response := parseMessage(buf[:dataLen])

		// If there's an empty response, return an empty answer
		if response.Header.ANCOUNT == 0 {
			response.Answers = []ResourceRecord{
				{
					Name:     question.QNAME,
					Type:     1, // A Record
					Class:    1,
					TTL:      0,
					RDLENGTH: 0,
					RDATA:    []byte{},
				},
			}
		}

		answers = append(answers, response.Answers...)
	}

	msg.Header.QR = true
	msg.Answers = answers
	msg.Header.ANCOUNT = uint16(len(answers))

	msgData := buildMessage(msg)

	return msgData, nil
}

func resolveRequest(data []byte) []byte {
	msg := parseMessage(data)
	msg.Header.QR = true
	msg.Header.AA = false
	msg.Header.TC = false
	msg.Header.RA = false
	msg.Header.Z = 0

	for idx := range msg.Questions {
		msg.Questions[idx].QTYPE = 1
		msg.Questions[idx].QCLASS = 1

		answer := ResourceRecord{
			Name:     msg.Questions[idx].QNAME,
			Type:     1, // A Record
			Class:    1, // IN => internet
			RDLENGTH: 4,
			RDATA:    []byte{192, 168, 0, 6},
		}

		msg.Answers = append(msg.Answers, answer)
	}

	msg.Header.ANCOUNT = uint16(len(msg.Answers))

	return buildMessage(msg)
}
