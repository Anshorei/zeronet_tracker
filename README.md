![Build](http://localhost:43110/1EtS177qrqicnGtGzktcMPEeL13FeLKJEG/img/build.svg)
![Tests](http://localhost:43110/1EtS177qrqicnGtGzktcMPEeL13FeLKJEG/img/tests.svg)
[![Coverage](http://localhost:43110/1EtS177qrqicnGtGzktcMPEeL13FeLKJEG/img/coverage.svg)](http://localhost:43110/1EtS177qrqicnGtGzktcMPEeL13FeLKJEG/coverage/html)
[![Documentation](http://localhost:43110/1EtS177qrqicnGtGzktcMPEeL13FeLKJEG/img/doc.svg)](http://localhost:43110/1EtS177qrqicnGtGzktcMPEeL13FeLKJEG/doc/zeronet_tracker)

# ZeroNet Tracker
This is a ZeroNet tracker written in Rust. It can keep track of a number of different peer types.

# Roadmap
Peertypes:
- [x] IPV4 & IPV6
- [ ] Onion v2 & v3
- [ ] I2P b32
- [ ] Loki

Features:
- [ ] Remove old peers
- [ ] Remove peerless hashes
- [ ] Server
  - [x] Overview showing: version, uptime, connections, peers, hashes
  - [x] Explore hashes
    - [ ] Option to crawl ZeroSites for hashes
  - [x] Explore peers
  - [ ] Show log (this should not be publicly accessible)
- [ ] Influx

# Compiling
Before compiling will need to clone the [ZeroNet Protocol repository](http://localhost:43110/1H3ct93gHL9BgtTnyrqJrkjn4NdociFFTn) to the same folder as the ZeroNet tracker repository. Then you can use `cargo` to run or build the program.

## Optional Features

### Server
The ZeroNet Tracker can optionally be compiled with the `server` flag. If enabled a server using Rocket and Maud will make useful information about the status of the tracker available on `localhost:8000`, or at the `ROCKET_PORT` environment variable.

It should be perfectly safe to make this available outside your network. Be aware that with low numbers of peers this information combined with a ZeroSites crawler could be used to deanonymize peers.

### Influx
The ZeroNet Tracker can be compiled with the `influx` flag. If enabled the tracker will send statistics to an influx database.
