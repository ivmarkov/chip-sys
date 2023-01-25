# chip-sys

[![CI](https://github.com/ivmarkov/chip-sys/actions/workflows/ci.yml/badge.svg)](https://github.com/ivmarkov/chip-sys/actions/workflows/ci.yml)
![crates.io](https://img.shields.io/crates/v/chip-sys.svg)

A SYS crate for the [Matter C++ SDK](https://github.com/project-chip/connectedhomeip), as well as a tiny set of type-safe Rust wrappers around it.

Currently buildable and runnable on Linux only, but should not be difficult to port to ESP32 and other MCUs which are already supported by the C++ SDK.

## Demo
* Build the [chip-tool](https://github.com/project-chip/connectedhomeip/tree/master/examples/chip-tool) utility from the Matter SDK
* Build and run the crate and the Light On-Off example: 
  ```sh
  cargo run --example on_off
  ```
* (During the build, the SYS crate will download and cache a private copy of the Matter SDK and its tooling; you only need to have the `build-essentials` of your distro pre-installed.)
* Comission the Light example using chip-tool: 
  ```sh
  chip-tool pairing onnetwork-long 23 20202021 3840
  ```
* Turn the Light on using chip-tool: 
  ```sh
  chip-tool onoff on 23 3
  ```
* Turn if back off: 
  ```sh
  chip-tool onoff off 23 3
  ```
