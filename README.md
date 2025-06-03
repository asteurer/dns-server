# Overview

In both Rust and Go, I built a (mostly) [RFC 1035](https://www.rfc-editor.org/rfc/pdfrfc/rfc1035.txt.pdf) compliant DNS server, that is cabable of both resolving and forwarding DNS messages.


This was a learning opportunity for me, and should definitely not be used in production.

# Usage

For the Go application: make sure you have Go version `1.24` or later installed, and be sure that you have navigated inside the `go` directory.

For the Rust application: make sure you have Rust version `1.86.0` or later installed, and be sure that you have navigated inside the `rust` directory.

To run the code, you have two options:
- You can run this as a DNS resolver, which will mean any DNS requests sent to the server will resolve to IP address `192.168.0.6`
    - For Go, run `go run .`
    - For Rust, run `cargo run`

- You can run this as a DNS forwarder, which means the server will forward DNS requests to a server you specify, and give you the response
    - For Go, run `go run . --resolver <DNS SERVER IP>:<DNS SERVER PORT>`
    - For Rust, run `cargo run -- --resolver <DNS SERVER IP>:<DNS SERVER PORT>`

To test that the server is working, we'll use the `dig` command: `dig @127.0.0.1 -p 2053 +noedns google.com`

Depending on whether you run the server as a resolver/forwarder, you'll get different IP addresses.