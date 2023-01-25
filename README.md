# chip-sys

[![CI](https://github.com/ivmarkov/chip-sys/actions/workflows/ci.yml/badge.svg)](https://github.com/ivmarkov/chip-sys/actions/workflows/ci.yml)
![crates.io](https://img.shields.io/crates/v/chip-sys.svg)

A SYS crate for the [Matter C++ SDK](https://github.com/project-chip/connectedhomeip), as well as a tiny set of type-safe Rust wrappers around it.

Currently buildable and runnable on Linux only, but should not be difficult to port to ESP32 and other MCUs which are already supported by the C++ SDK.

## Demo
* Install the [Linux build prerequisites](https://github.com/project-chip/connectedhomeip/blob/master/docs/guides/BUILDING.md#installing-prerequisites-on-linux). For Debian/Ubuntu:
  ```sh
  sudo apt-get install git gcc g++ pkg-config libssl-dev libdbus-1-dev libglib2.0-dev libavahi-client-dev ninja-build python3-venv python3-dev python3-pip unzip libgirepository1.0-dev libcairo2-dev libreadline-dev
  ```
* Build the [chip-tool](https://github.com/project-chip/connectedhomeip/tree/master/examples/chip-tool) utility from the Matter SDK
* Build and run the crate and the Light On-Off example: 
  ```sh
  cargo run --example on_off
  ```
* (During the build, the SYS crate will download and cache a private copy of the Matter SDK and its tooling; you only need to have the build prerequisites of your distro pre-installed, as per above.)
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
