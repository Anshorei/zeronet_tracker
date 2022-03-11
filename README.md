![Build](http://localhost:43110/1EtS177qrqicnGtGzktcMPEeL13FeLKJEG/img/build.svg)
![Tests](http://localhost:43110/1EtS177qrqicnGtGzktcMPEeL13FeLKJEG/img/tests.svg)
[![Coverage](http://localhost:43110/1EtS177qrqicnGtGzktcMPEeL13FeLKJEG/img/coverage.svg)](http://localhost:43110/1EtS177qrqicnGtGzktcMPEeL13FeLKJEG/coverage/html)
[![Documentation](http://localhost:43110/1EtS177qrqicnGtGzktcMPEeL13FeLKJEG/img/doc.svg)](http://localhost:43110/1EtS177qrqicnGtGzktcMPEeL13FeLKJEG/doc/zeronet_tracker)

# ZeroNet Tracker
This is a ZeroNet tracker written in Rust. It can keep track of a number of different peer types.

## Installation

### Prerequisites
- Copy of the source code
- Cargo + rustc nightly

### Steps
Run `cargo +nightly install --path .` in the local repository root

## Roadmap
Peertypes:
- [x] IPV4 & IPV6
- [x] Onion v2 & v3
- [x] I2P b32

Features:
- [x] Remove dead peers
- [x] Remove stale hashes
- [ ] Server
  - [x] Overview showing: version, uptime, connections, peers, hashes
  - [x] Explore hashes
    - [ ] Option to crawl ZeroSites for hashes
  - [x] Explore peers
  - [ ] Show log (this should not be publicly accessible)
- [x] Metrics
  - [x] Endpoint for Prometheus scraper

# Building
This ZeroNet Tracker depends on the ZeroNet Protocol library which is available from [the Cratez Rust Registry](/1CRAteZVBUYrnx8jj9x87A1zCnptrWFhPH), information on how to add Cratez to your cargo's registry configuration can be found on the site.

## Optional Features

### Server
The ZeroNet Tracker can optionally be compiled with the `server` flag. If enabled a server using Rocket and Maud will make useful information about the status of the tracker available on `localhost:8000`, or at the `ROCKET_PORT` environment variable.

It should be perfectly safe to make this available outside your network. Be aware that with low numbers of peers this information combined with a ZeroSites crawler could be used to deanonymize peers.
